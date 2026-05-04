//! File System API implementation.
//!
//! This module implements the WHATWG File System API and the Origin Private
//! File System (OPFS) reachable from `navigator.storage.getDirectory()`. The
//! existing file-picker entry points (`showOpenFilePicker` etc.) and the
//! legacy permission model are preserved for backwards compatibility.
//!
//! ## OPFS architecture
//!
//! Each origin gets an isolated directory under
//! `<data_local_dir>/thalora/opfs/<slug>` managed by [`opfs_backend::OpfsBackend`].
//! Handles obtained from `navigator.storage.getDirectory()` carry an
//! `is_opfs: true` flag plus an `Arc<OpfsBackend>` reference and use real disk
//! I/O via that backend. Permission checks are short-circuited to `"granted"`
//! for OPFS handles since access is implicitly authorised by same-origin policy.
//!
//! Non-OPFS handles (returned from the picker functions) retain the legacy
//! `vfs`-backed code path.
//!
//! More information:
//!  - [WHATWG File System Specification](https://fs.spec.whatwg.org/)
//!  - [MDN File System API](https://developer.mozilla.org/en-US/docs/Web/API/File_System_API)

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::Intrinsics,
    js_string,
    object::JsPromise,
    property::PropertyDescriptor,
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
};
use boa_gc::{Finalize, Trace, Tracer};

pub mod errors;
pub mod iterators;
pub mod opfs_backend;
pub mod sync_access;
pub mod writable_stream;

use errors::{names, reject_with};
use iterators::IteratorKind;
use opfs_backend::OpfsBackend;
use sync_access::FileSystemSyncAccessHandle;
use writable_stream::FileSystemWritableFileStream;

// =============================================================================
// PERMISSION MANAGEMENT SYSTEM
// =============================================================================

/// Permission state for file system access.
/// Follows the W3C Permissions API states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PermissionState {
    /// Permission has been explicitly granted
    Granted,
    /// Permission has been explicitly denied
    Denied,
    /// Permission needs to be requested (user prompt needed)
    Prompt,
}

impl PermissionState {
    /// Convert to JavaScript string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            PermissionState::Granted => "granted",
            PermissionState::Denied => "denied",
            PermissionState::Prompt => "prompt",
        }
    }
}

/// Permission mode for file system access
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PermissionMode {
    /// Read-only access
    Read,
    /// Read and write access
    ReadWrite,
}

impl PermissionMode {
    /// Parse from JavaScript descriptor object
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "readwrite" | "read-write" => PermissionMode::ReadWrite,
            _ => PermissionMode::Read,
        }
    }
}

/// A permission entry tracking access to a specific path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEntry {
    /// The origin (e.g., "https://example.com")
    pub origin: String,
    /// The path being accessed
    pub path: PathBuf,
    /// Permission mode (read or readwrite)
    pub mode: PermissionMode,
    /// Current permission state
    pub state: PermissionState,
    /// Whether this permission was granted by MCP client
    pub mcp_granted: bool,
}

#[derive(Debug)]
pub struct FileSystemPermissions {
    permissions: Arc<RwLock<HashMap<(String, PathBuf, PermissionMode), PermissionEntry>>>,
}

impl Default for FileSystemPermissions {
    fn default() -> Self {
        Self::new()
    }
}

impl FileSystemPermissions {
    pub fn new() -> Self {
        Self {
            permissions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn query(&self, origin: &str, path: &Path, mode: PermissionMode) -> PermissionState {
        let normalized_path = Self::normalize_path(path);
        let key = (origin.to_string(), normalized_path, mode);
        let permissions = self.permissions.read().unwrap();
        permissions
            .get(&key)
            .map(|entry| entry.state)
            .unwrap_or(PermissionState::Denied)
    }

    pub fn request(&self, origin: &str, path: &Path, mode: PermissionMode) -> PermissionState {
        self.query(origin, path, mode)
    }

    pub fn grant_permission(&self, origin: &str, path: &Path, mode: PermissionMode) {
        let normalized_path = Self::normalize_path(path);
        let key = (origin.to_string(), normalized_path.clone(), mode);
        let mut permissions = self.permissions.write().unwrap();
        permissions.insert(
            key,
            PermissionEntry {
                origin: origin.to_string(),
                path: normalized_path,
                mode,
                state: PermissionState::Granted,
                mcp_granted: true,
            },
        );
    }

    pub fn revoke_permission(&self, origin: &str, path: &Path, mode: PermissionMode) {
        let normalized_path = Self::normalize_path(path);
        let key = (origin.to_string(), normalized_path, mode);
        let mut permissions = self.permissions.write().unwrap();
        permissions.remove(&key);
    }

    pub fn clear_all(&self) {
        let mut permissions = self.permissions.write().unwrap();
        permissions.clear();
    }

    pub fn clear_origin(&self, origin: &str) {
        let mut permissions = self.permissions.write().unwrap();
        permissions.retain(|key, _| key.0 != origin);
    }

    pub fn list_granted(&self) -> Vec<PermissionEntry> {
        let permissions = self.permissions.read().unwrap();
        permissions
            .values()
            .filter(|entry| entry.state == PermissionState::Granted)
            .cloned()
            .collect()
    }

    fn normalize_path(path: &Path) -> PathBuf {
        let mut normalized = PathBuf::new();
        for component in path.components() {
            match component {
                std::path::Component::Normal(c) => normalized.push(c),
                std::path::Component::RootDir => normalized.push("/"),
                std::path::Component::ParentDir => {
                    normalized.pop();
                }
                _ => {}
            }
        }
        normalized
    }
}

/// Global permission store instance
pub static PERMISSIONS: Lazy<FileSystemPermissions> = Lazy::new(FileSystemPermissions::new);

/// Get the current origin for permission checks. Reads from the realm context
/// installed by `crate::realm_ext::install` (see `lib.rs` and the worker boot
/// path); falls back to `"thalora://local"` for tests that don't initialise it.
fn get_current_origin(context: &Context) -> String {
    crate::realm_ext::current_origin(context)
}

// =============================================================================
// FILE SYSTEM HANDLE DATA STRUCTURES
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileData {
    pub name: String,
    pub kind: String,
    pub size: u64,
    pub last_modified: u64,
}

/// Common state for all file system handles. Carries an OPFS backend reference
/// and a virtual path when the handle was minted by `navigator.storage.getDirectory()`.
#[derive(Debug, Trace, Finalize, JsData)]
pub struct FileSystemHandle {
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) path: PathBuf,
    pub(crate) is_opfs: bool,
    #[unsafe_ignore_trace]
    pub(crate) backend: Option<Arc<OpfsBackend>>,
    pub(crate) virtual_path: PathBuf,
}

impl FileSystemHandle {
    /// Legacy constructor used by the picker functions (non-OPFS).
    pub fn new(name: String, kind: String, path: PathBuf) -> Self {
        Self {
            name,
            kind,
            path,
            is_opfs: false,
            backend: None,
            virtual_path: PathBuf::new(),
        }
    }

    /// OPFS constructor: handle is rooted in an origin's OPFS tree.
    pub fn new_opfs(
        name: String,
        kind: String,
        backend: Arc<OpfsBackend>,
        virtual_path: PathBuf,
    ) -> Self {
        let path = backend
            .resolve(&virtual_path)
            .unwrap_or_else(|_| backend.root_path().to_path_buf());
        Self {
            name,
            kind,
            path,
            is_opfs: true,
            backend: Some(backend),
            virtual_path,
        }
    }
}

impl BuiltInObject for FileSystemHandle {
    const NAME: JsString = StaticJsStrings::EMPTY_STRING;
}

impl IntrinsicObject for FileSystemHandle {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::is_same_entry, js_string!("isSameEntry"), 1)
            .method(Self::query_permission, js_string!("queryPermission"), 0)
            .method(
                Self::request_permission,
                js_string!("requestPermission"),
                0,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for FileSystemHandle {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;
    const STANDARD_CONSTRUCTOR: fn(
        &boa_engine::context::intrinsics::StandardConstructors,
    ) -> &boa_engine::context::intrinsics::StandardConstructor =
        boa_engine::context::intrinsics::StandardConstructors::file_system_handle;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("FileSystemHandle constructor cannot be called directly")
            .into())
    }
}

impl FileSystemHandle {
    pub(crate) fn is_same_entry(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let this_path = handle_path(this).ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a FileSystemHandle")
        })?;
        let other = args.get_or_undefined(0);
        let other_obj = other.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Argument is not a FileSystemHandle")
        })?;
        let other_path = handle_path(&JsValue::from(other_obj.clone())).ok_or_else(|| {
            JsNativeError::typ().with_message("Argument is not a FileSystemHandle")
        })?;

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(
            &JsValue::undefined(),
            &[JsValue::from(this_path == other_path)],
            context,
        )?;
        Ok(JsValue::from(promise))
    }

    pub(crate) fn query_permission(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let (is_opfs, path, _) = handle_meta(this).ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a FileSystemHandle")
        })?;

        let mode = parse_permission_mode(args.first(), context);

        let state = if is_opfs {
            PermissionState::Granted
        } else {
            let origin = get_current_origin(context);
            PERMISSIONS.query(&origin, &path, mode)
        };

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(
            &JsValue::undefined(),
            &[JsValue::from(JsString::from(state.as_str()))],
            context,
        )?;
        Ok(JsValue::from(promise))
    }

    pub(crate) fn request_permission(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let (is_opfs, path, _) = handle_meta(this).ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a FileSystemHandle")
        })?;
        let mode = parse_permission_mode(args.first(), context);

        let state = if is_opfs {
            PermissionState::Granted
        } else {
            let origin = get_current_origin(context);
            PERMISSIONS.request(&origin, &path, mode)
        };

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers.resolve.call(
            &JsValue::undefined(),
            &[JsValue::from(JsString::from(state.as_str()))],
            context,
        )?;
        Ok(JsValue::from(promise))
    }
}

fn parse_permission_mode(descriptor: Option<&JsValue>, context: &mut Context) -> PermissionMode {
    let Some(descriptor) = descriptor else {
        return PermissionMode::Read;
    };
    let Some(obj) = descriptor.as_object() else {
        return PermissionMode::Read;
    };
    let Ok(mode_val) = obj.get(js_string!("mode"), context) else {
        return PermissionMode::Read;
    };
    let Ok(mode_str) = mode_val.to_string(context) else {
        return PermissionMode::Read;
    };
    PermissionMode::from_str(&mode_str.to_std_string_escaped())
}

/// Pull (`is_opfs`, `path`, `kind`) from any handle subtype.
fn handle_meta(this: &JsValue) -> Option<(bool, PathBuf, String)> {
    let obj = this.as_object()?;
    if let Some(file) = obj.downcast_ref::<FileSystemFileHandle>() {
        return Some((
            file.handle.is_opfs,
            file.handle.path.clone(),
            file.handle.kind.clone(),
        ));
    }
    if let Some(dir) = obj.downcast_ref::<FileSystemDirectoryHandle>() {
        return Some((
            dir.handle.is_opfs,
            dir.handle.path.clone(),
            dir.handle.kind.clone(),
        ));
    }
    if let Some(base) = obj.downcast_ref::<FileSystemHandle>() {
        return Some((base.is_opfs, base.path.clone(), base.kind.clone()));
    }
    None
}

fn handle_path(this: &JsValue) -> Option<PathBuf> {
    handle_meta(this).map(|(_, p, _)| p)
}

// =============================================================================
// FILE HANDLE
// =============================================================================

#[derive(Debug, Trace, Finalize, JsData)]
pub struct FileSystemFileHandle {
    pub(crate) handle: FileSystemHandle,
}

impl FileSystemFileHandle {
    /// Legacy non-OPFS constructor (used by pickers).
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            handle: FileSystemHandle::new(name, "file".to_string(), path),
        }
    }

    /// OPFS-rooted constructor.
    pub fn new_opfs(
        name: String,
        virtual_path: PathBuf,
        backend: Arc<OpfsBackend>,
        is_opfs: bool,
    ) -> Self {
        let mut handle =
            FileSystemHandle::new_opfs(name, "file".to_string(), backend, virtual_path);
        handle.is_opfs = is_opfs;
        Self { handle }
    }
}

impl BuiltInObject for FileSystemFileHandle {
    const NAME: JsString = StaticJsStrings::EMPTY_STRING;
}

impl IntrinsicObject for FileSystemFileHandle {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().file_system_handle().prototype()))
            .method(Self::is_same_entry_proxy, js_string!("isSameEntry"), 1)
            .method(Self::query_permission_proxy, js_string!("queryPermission"), 0)
            .method(
                Self::request_permission_proxy,
                js_string!("requestPermission"),
                0,
            )
            .method(Self::get_file, js_string!("getFile"), 0)
            .method(Self::create_writable, js_string!("createWritable"), 0)
            .method(
                Self::create_sync_access_handle,
                js_string!("createSyncAccessHandle"),
                0,
            )
            .property(js_string!("kind"), JsString::from("file"), boa_engine::property::Attribute::CONFIGURABLE)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for FileSystemFileHandle {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;
    const STANDARD_CONSTRUCTOR: fn(
        &boa_engine::context::intrinsics::StandardConstructors,
    ) -> &boa_engine::context::intrinsics::StandardConstructor =
        boa_engine::context::intrinsics::StandardConstructors::file_system_file_handle;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("FileSystemFileHandle constructor cannot be called directly")
            .into())
    }
}

impl FileSystemFileHandle {
    fn is_same_entry_proxy(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        FileSystemHandle::is_same_entry(this, args, context)
    }
    fn query_permission_proxy(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        FileSystemHandle::query_permission(this, args, context)
    }
    fn request_permission_proxy(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        FileSystemHandle::request_permission(this, args, context)
    }

    pub(crate) fn get_file(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a FileSystemFileHandle")
        })?;
        let (name, bytes, last_modified) = {
            let file_handle = obj.downcast_ref::<Self>().ok_or_else(|| {
                JsNativeError::typ().with_message("'this' is not a FileSystemFileHandle")
            })?;

            let name = file_handle.handle.name.clone();
            if let Some(backend) = &file_handle.handle.backend {
                match backend.read_bytes(&file_handle.handle.virtual_path) {
                    Ok(bytes) => {
                        let modified = backend
                            .metadata(&file_handle.handle.virtual_path)
                            .ok()
                            .and_then(|m| m.modified().ok())
                            .and_then(|t| {
                                t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| d.as_millis() as u64)
                            })
                            .unwrap_or(0);
                        (name, bytes, modified)
                    }
                    Err(e) => {
                        return reject_with(
                            errors::map_io_error(&e),
                            &format!("OPFS read failed: {e}"),
                            context,
                        );
                    }
                }
            } else {
                let bytes = vfs::fs::read(&file_handle.handle.path).unwrap_or_default();
                (name, bytes, 0)
            }
        };

        let blob = crate::file::blob::BlobData::new(bytes, "application/octet-stream".to_string());
        let file_data = crate::file::file::FileData::new(blob, name, Some(last_modified));
        let file_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().file().prototype(),
            file_data,
        );

        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[file_obj.into()], context)?;
        Ok(JsValue::from(promise))
    }

    pub(crate) fn create_writable(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let keep_existing = args
            .get_or_undefined(0)
            .as_object()
            .and_then(|o| o.get(js_string!("keepExistingData"), context).ok())
            .map(|v| v.to_boolean())
            .unwrap_or(false);

        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a FileSystemFileHandle")
        })?;
        let (backend, virtual_path) = {
            let file_handle = obj.downcast_ref::<Self>().ok_or_else(|| {
                JsNativeError::typ().with_message("'this' is not a FileSystemFileHandle")
            })?;
            match (&file_handle.handle.backend, &file_handle.handle.virtual_path) {
                (Some(b), p) => (b.clone(), p.clone()),
                _ => {
                    return reject_with(
                        names::INVALID_STATE,
                        "createWritable is only supported on OPFS handles in this build",
                        context,
                    );
                }
            }
        };

        match FileSystemWritableFileStream::new(backend, virtual_path, keep_existing) {
            Ok(stream) => {
                let stream_obj = JsObject::from_proto_and_data_with_shared_shape(
                    context.root_shape(),
                    context
                        .intrinsics()
                        .constructors()
                        .file_system_writable_file_stream()
                        .prototype(),
                    stream,
                );
                let (promise, resolvers) = JsPromise::new_pending(context);
                resolvers
                    .resolve
                    .call(&JsValue::undefined(), &[stream_obj.into()], context)?;
                Ok(JsValue::from(promise))
            }
            Err(e) => reject_with(errors::map_io_error(&e), &format!("{e}"), context),
        }
    }

    pub(crate) fn create_sync_access_handle(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a FileSystemFileHandle")
        })?;
        let (backend, virtual_path, is_opfs) = {
            let file_handle = obj.downcast_ref::<Self>().ok_or_else(|| {
                JsNativeError::typ().with_message("'this' is not a FileSystemFileHandle")
            })?;
            (
                file_handle.handle.backend.clone(),
                file_handle.handle.virtual_path.clone(),
                file_handle.handle.is_opfs,
            )
        };
        if !is_opfs || backend.is_none() {
            return reject_with(
                names::INVALID_STATE,
                "createSyncAccessHandle is only available on OPFS handles",
                context,
            );
        }
        if !crate::realm_ext::is_worker(context) {
            return reject_with(
                names::INVALID_STATE,
                "createSyncAccessHandle may only be called from a Worker",
                context,
            );
        }
        match sync_access::FileSystemSyncAccessHandle::open(backend.unwrap(), virtual_path) {
            Ok(handle) => {
                let handle_obj = JsObject::from_proto_and_data_with_shared_shape(
                    context.root_shape(),
                    context
                        .intrinsics()
                        .constructors()
                        .file_system_sync_access_handle()
                        .prototype(),
                    handle,
                );
                let (promise, resolvers) = JsPromise::new_pending(context);
                resolvers
                    .resolve
                    .call(&JsValue::undefined(), &[handle_obj.into()], context)?;
                Ok(JsValue::from(promise))
            }
            Err(sync_access::SyncOpenError::AlreadyLocked) => reject_with(
                names::NO_MODIFICATION_ALLOWED,
                "another sync access handle is already open for this file",
                context,
            ),
            Err(sync_access::SyncOpenError::PathInvalid) => {
                reject_with(names::SYNTAX, "invalid OPFS path", context)
            }
            Err(sync_access::SyncOpenError::Io(e)) => {
                reject_with(errors::map_io_error(&e), &format!("{e}"), context)
            }
        }
    }
}

// =============================================================================
// DIRECTORY HANDLE
// =============================================================================

#[derive(Debug, Trace, Finalize, JsData)]
pub struct FileSystemDirectoryHandle {
    pub(crate) handle: FileSystemHandle,
}

impl FileSystemDirectoryHandle {
    pub fn new(name: String, path: PathBuf) -> Self {
        Self {
            handle: FileSystemHandle::new(name, "directory".to_string(), path),
        }
    }

    pub fn new_opfs(
        name: String,
        virtual_path: PathBuf,
        backend: Arc<OpfsBackend>,
        is_opfs: bool,
    ) -> Self {
        let mut handle =
            FileSystemHandle::new_opfs(name, "directory".to_string(), backend, virtual_path);
        handle.is_opfs = is_opfs;
        Self { handle }
    }

    pub fn new_opfs_root(backend: Arc<OpfsBackend>) -> Self {
        Self::new_opfs(String::new(), PathBuf::from("/"), backend, true)
    }

    /// Backend reference, if any (None for picker-style handles).
    pub fn backend(&self) -> Option<&Arc<OpfsBackend>> {
        self.handle.backend.as_ref()
    }
}

impl BuiltInObject for FileSystemDirectoryHandle {
    const NAME: JsString = StaticJsStrings::EMPTY_STRING;
}

impl IntrinsicObject for FileSystemDirectoryHandle {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().file_system_handle().prototype()))
            .method(Self::is_same_entry_proxy, js_string!("isSameEntry"), 1)
            .method(Self::query_permission_proxy, js_string!("queryPermission"), 0)
            .method(
                Self::request_permission_proxy,
                js_string!("requestPermission"),
                0,
            )
            .method(Self::get_file_handle, js_string!("getFileHandle"), 1)
            .method(Self::get_directory_handle, js_string!("getDirectoryHandle"), 1)
            .method(Self::remove_entry, js_string!("removeEntry"), 1)
            .method(Self::resolve, js_string!("resolve"), 1)
            .method(Self::keys, js_string!("keys"), 0)
            .method(Self::values, js_string!("values"), 0)
            .method(Self::entries, js_string!("entries"), 0)
            .method(Self::async_iterator, JsSymbol::async_iterator(), 0)
            .property(js_string!("kind"), JsString::from("directory"), boa_engine::property::Attribute::CONFIGURABLE)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for FileSystemDirectoryHandle {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;
    const STANDARD_CONSTRUCTOR: fn(
        &boa_engine::context::intrinsics::StandardConstructors,
    ) -> &boa_engine::context::intrinsics::StandardConstructor =
        boa_engine::context::intrinsics::StandardConstructors::file_system_directory_handle;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("FileSystemDirectoryHandle constructor cannot be called directly")
            .into())
    }
}

impl FileSystemDirectoryHandle {
    fn is_same_entry_proxy(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        FileSystemHandle::is_same_entry(this, args, context)
    }
    fn query_permission_proxy(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        FileSystemHandle::query_permission(this, args, context)
    }
    fn request_permission_proxy(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        FileSystemHandle::request_permission(this, args, context)
    }

    pub(crate) fn get_file_handle(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let name = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        if !is_valid_name(&name) {
            return Err(JsNativeError::typ()
                .with_message("Invalid file name")
                .into());
        }
        let create = args
            .get_or_undefined(1)
            .as_object()
            .and_then(|o| o.get(js_string!("create"), context).ok())
            .map(|v| v.to_boolean())
            .unwrap_or(false);

        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle")
        })?;
        let (backend, virtual_path, is_opfs) = {
            let dir = obj.downcast_ref::<Self>().ok_or_else(|| {
                JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle")
            })?;
            match &dir.handle.backend {
                Some(b) => (b.clone(), dir.handle.virtual_path.clone(), dir.handle.is_opfs),
                None => {
                    return legacy_get_file_handle(&dir.handle.path, &name, context);
                }
            }
        };

        let child_path = virtual_path.join(&name);
        if backend.exists(&child_path) {
            if backend.is_dir(&child_path) {
                return reject_with(
                    names::TYPE_MISMATCH,
                    "expected file but found directory",
                    context,
                );
            }
        } else if !create {
            return reject_with(
                names::NOT_FOUND,
                &format!("file '{name}' not found"),
                context,
            );
        } else if let Err(e) = backend.write_bytes(&child_path, &[]) {
            return reject_with(errors::map_io_error(&e), &format!("{e}"), context);
        }

        let handle = FileSystemFileHandle::new_opfs(name, child_path, backend, is_opfs);
        let handle_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .constructors()
                .file_system_file_handle()
                .prototype(),
            handle,
        );
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[handle_obj.into()], context)?;
        Ok(JsValue::from(promise))
    }

    pub(crate) fn get_directory_handle(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let name = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        if !is_valid_name(&name) {
            return Err(JsNativeError::typ()
                .with_message("Invalid directory name")
                .into());
        }
        let create = args
            .get_or_undefined(1)
            .as_object()
            .and_then(|o| o.get(js_string!("create"), context).ok())
            .map(|v| v.to_boolean())
            .unwrap_or(false);

        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle")
        })?;
        let (backend, virtual_path, is_opfs) = {
            let dir = obj.downcast_ref::<Self>().ok_or_else(|| {
                JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle")
            })?;
            match &dir.handle.backend {
                Some(b) => (b.clone(), dir.handle.virtual_path.clone(), dir.handle.is_opfs),
                None => {
                    let subdir = dir.handle.path.join(&name);
                    let _ = vfs::fs::write(subdir.join(".keep"), b"");
                    let new_handle = Self::new(name.clone(), subdir);
                    let handle_obj = JsObject::from_proto_and_data_with_shared_shape(
                        context.root_shape(),
                        context
                            .intrinsics()
                            .constructors()
                            .file_system_directory_handle()
                            .prototype(),
                        new_handle,
                    );
                    let (promise, resolvers) = JsPromise::new_pending(context);
                    resolvers
                        .resolve
                        .call(&JsValue::undefined(), &[handle_obj.into()], context)?;
                    return Ok(JsValue::from(promise));
                }
            }
        };

        let child_path = virtual_path.join(&name);
        if backend.exists(&child_path) {
            if !backend.is_dir(&child_path) {
                return reject_with(
                    names::TYPE_MISMATCH,
                    "expected directory but found file",
                    context,
                );
            }
        } else if !create {
            return reject_with(
                names::NOT_FOUND,
                &format!("directory '{name}' not found"),
                context,
            );
        } else if let Err(e) = backend.mkdir(&child_path) {
            return reject_with(errors::map_io_error(&e), &format!("{e}"), context);
        }

        let handle = FileSystemDirectoryHandle::new_opfs(name, child_path, backend, is_opfs);
        let handle_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context
                .intrinsics()
                .constructors()
                .file_system_directory_handle()
                .prototype(),
            handle,
        );
        let (promise, resolvers) = JsPromise::new_pending(context);
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[handle_obj.into()], context)?;
        Ok(JsValue::from(promise))
    }

    pub(crate) fn remove_entry(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let name = args
            .get_or_undefined(0)
            .to_string(context)?
            .to_std_string_escaped();
        if !is_valid_name(&name) {
            return Err(JsNativeError::typ()
                .with_message("Invalid entry name")
                .into());
        }
        let recursive = args
            .get_or_undefined(1)
            .as_object()
            .and_then(|o| o.get(js_string!("recursive"), context).ok())
            .map(|v| v.to_boolean())
            .unwrap_or(false);

        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle")
        })?;
        let (backend, virtual_path) = {
            let dir = obj.downcast_ref::<Self>().ok_or_else(|| {
                JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle")
            })?;
            match &dir.handle.backend {
                Some(b) => (b.clone(), dir.handle.virtual_path.clone()),
                None => {
                    let (promise, resolvers) = JsPromise::new_pending(context);
                    resolvers
                        .resolve
                        .call(&JsValue::undefined(), &[JsValue::undefined()], context)?;
                    return Ok(JsValue::from(promise));
                }
            }
        };

        let child = virtual_path.join(&name);
        if !backend.exists(&child) {
            return reject_with(
                names::NOT_FOUND,
                &format!("entry '{name}' not found"),
                context,
            );
        }
        let result = if backend.is_dir(&child) {
            if recursive {
                backend.remove_dir_recursive(&child)
            } else {
                match backend.is_dir_empty(&child) {
                    Ok(true) => backend.remove_dir(&child),
                    Ok(false) => {
                        return reject_with(
                            names::INVALID_MODIFICATION,
                            "directory is not empty",
                            context,
                        );
                    }
                    Err(e) => Err(e),
                }
            }
        } else {
            backend.remove_file(&child)
        };
        match result {
            Ok(()) => {
                let (promise, resolvers) = JsPromise::new_pending(context);
                resolvers
                    .resolve
                    .call(&JsValue::undefined(), &[JsValue::undefined()], context)?;
                Ok(JsValue::from(promise))
            }
            Err(e) => reject_with(errors::map_io_error(&e), &format!("{e}"), context),
        }
    }

    pub(crate) fn resolve(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle")
        })?;
        let parent_virtual = {
            let dir = obj.downcast_ref::<Self>().ok_or_else(|| {
                JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle")
            })?;
            dir.handle.virtual_path.clone()
        };

        let other_val = args.get_or_undefined(0);
        let other_path = handle_meta(other_val).map(|(_, _, _)| ()).and_then(|_| {
            let other_obj = other_val.as_object()?;
            if let Some(file) = other_obj.downcast_ref::<FileSystemFileHandle>() {
                Some(file.handle.virtual_path.clone())
            } else if let Some(dir) = other_obj.downcast_ref::<Self>() {
                Some(dir.handle.virtual_path.clone())
            } else {
                None
            }
        });

        let (promise, resolvers) = JsPromise::new_pending(context);
        let result = match other_path {
            Some(other) => match other.strip_prefix(&parent_virtual) {
                Ok(rel) => {
                    let arr =
                        boa_engine::builtins::array::Array::array_create(0, None, context)?;
                    let mut idx = 0u32;
                    for comp in rel.components() {
                        if let std::path::Component::Normal(s) = comp {
                            arr.set(
                                idx,
                                JsValue::from(JsString::from(s.to_string_lossy().into_owned())),
                                true,
                                context,
                            )?;
                            idx += 1;
                        }
                    }
                    JsValue::from(arr)
                }
                Err(_) => JsValue::null(),
            },
            None => JsValue::null(),
        };
        resolvers
            .resolve
            .call(&JsValue::undefined(), &[result], context)?;
        Ok(JsValue::from(promise))
    }

    pub(crate) fn keys(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Self::iterator_for_kind(this, IteratorKind::Keys, context)
    }
    pub(crate) fn values(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Self::iterator_for_kind(this, IteratorKind::Values, context)
    }
    pub(crate) fn entries(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Self::iterator_for_kind(this, IteratorKind::Entries, context)
    }
    pub(crate) fn async_iterator(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        Self::iterator_for_kind(this, IteratorKind::Entries, context)
    }

    fn iterator_for_kind(
        this: &JsValue,
        kind: IteratorKind,
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle")
        })?;
        let dir = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a FileSystemDirectoryHandle")
        })?;
        // Clone enough to release the borrow before calling back into iterators::build_iterator,
        // which takes &mut Context.
        let snapshot_dir = FileSystemDirectoryHandle {
            handle: FileSystemHandle {
                name: dir.handle.name.clone(),
                kind: dir.handle.kind.clone(),
                path: dir.handle.path.clone(),
                is_opfs: dir.handle.is_opfs,
                backend: dir.handle.backend.clone(),
                virtual_path: dir.handle.virtual_path.clone(),
            },
        };
        drop(dir);
        iterators::build_iterator(&snapshot_dir, kind, context)
    }
}

fn is_valid_name(name: &str) -> bool {
    !name.is_empty()
        && name != "."
        && name != ".."
        && !name.contains('/')
        && !name.contains('\\')
        && !name.contains('\0')
}

impl FileSystemDirectoryHandle {
    /// Expose the virtual path for iterator construction.
    pub(crate) fn virtual_path_ref(&self) -> &Path {
        &self.handle.virtual_path
    }
}

// Backwards-compat: expose `backend` and `virtual_path` directly through `handle` so iterators
// can read them with a single `.backend` / `.virtual_path` field access.
impl std::ops::Deref for FileSystemDirectoryHandle {
    type Target = FileSystemHandle;
    fn deref(&self) -> &FileSystemHandle {
        &self.handle
    }
}

// Legacy non-OPFS getFileHandle (called when `backend` is None — picker pathway).
fn legacy_get_file_handle(
    parent: &Path,
    name: &str,
    context: &mut Context,
) -> JsResult<JsValue> {
    let file_path = parent.join(name);
    let new_handle = FileSystemFileHandle::new(name.to_string(), file_path);
    let handle_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context
            .intrinsics()
            .constructors()
            .file_system_file_handle()
            .prototype(),
        new_handle,
    );
    let (promise, resolvers) = JsPromise::new_pending(context);
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[handle_obj.into()], context)?;
    Ok(JsValue::from(promise))
}

// =============================================================================
// PICKER FUNCTIONS (legacy)
// =============================================================================

pub fn show_open_file_picker(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let file_path = PathBuf::from("/documents/example_document.txt");
    let sample_content = b"This is a sample document file created by showOpenFilePicker.\nYou can read and modify this content through the File System API.";
    let _ = vfs::fs::write(&file_path, sample_content);

    let file_handle = FileSystemFileHandle::new("example_document.txt".to_string(), file_path);
    let file_handle_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context
            .intrinsics()
            .constructors()
            .file_system_file_handle()
            .prototype(),
        file_handle,
    );

    let array = boa_engine::builtins::array::Array::array_create(1, None, context)?;
    array.set(0, file_handle_obj, true, context)?;
    let (promise, resolvers) = JsPromise::new_pending(context);
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[array.into()], context)?;
    Ok(JsValue::from(promise))
}

pub fn show_save_file_picker(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let file_path = PathBuf::from("/documents/new_save_file.txt");
    let _ = vfs::fs::write(&file_path, b"");
    let file_handle = FileSystemFileHandle::new("new_save_file.txt".to_string(), file_path);
    let file_handle_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context
            .intrinsics()
            .constructors()
            .file_system_file_handle()
            .prototype(),
        file_handle,
    );
    let (promise, resolvers) = JsPromise::new_pending(context);
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[file_handle_obj.into()], context)?;
    Ok(JsValue::from(promise))
}

pub fn show_directory_picker(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let dir_path = PathBuf::from("/documents");
    let _ = vfs::fs::write(dir_path.join("readme.txt"), b"Welcome to the documents directory!");
    let _ = vfs::fs::write(dir_path.join("notes.txt"), b"Sample notes file.");
    let dir_handle = FileSystemDirectoryHandle::new("documents".to_string(), dir_path);
    let dir_handle_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        context
            .intrinsics()
            .constructors()
            .file_system_directory_handle()
            .prototype(),
        dir_handle,
    );
    let (promise, resolvers) = JsPromise::new_pending(context);
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[dir_handle_obj.into()], context)?;
    Ok(JsValue::from(promise))
}

#[cfg(test)]
mod tests;

// Suppress unused-import warnings for items conditionally used by submodules.
#[allow(dead_code)]
fn _silence_unused() {
    let _ = PropertyDescriptor::default();
    let _ = std::sync::Arc::new(0u8);
    let _ = HashMap::<String, FileData>::new();
    let _ = RwLock::new(0u8);
    let _: Option<&FileSystemSyncAccessHandle> = None;
}

// Placate unused-warning on Tracer since the manual Trace impls were dropped.
#[allow(dead_code)]
fn _tracer_unused(_: &mut Tracer) {}
