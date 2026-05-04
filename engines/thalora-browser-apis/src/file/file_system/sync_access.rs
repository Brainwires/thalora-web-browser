//! `FileSystemSyncAccessHandle` — synchronous, exclusive-lock file I/O.
//!
//! Per WHATWG File System spec this is OPFS-only and worker-thread-only. We
//! enforce both gates in `FileSystemFileHandle::create_sync_access_handle`.
//! At most one live handle per file path is allowed; a second concurrent
//! `createSyncAccessHandle` call rejects with `NoModificationAllowedError`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Weak};

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::Intrinsics,
    js_string,
    object::{JsPromise, builtins::JsArrayBuffer, builtins::JsTypedArray},
    realm::Realm,
    string::StaticJsStrings,
};
use boa_gc::{Finalize, Trace};
use once_cell::sync::Lazy;
use parking_lot::Mutex;

use super::errors::{names, reject_with};
use super::opfs_backend::OpfsBackend;

/// Per-canonical-path exclusive lock map. The map stores `Weak<()>` so dropped
/// handles release automatically without explicit cleanup.
static FILE_LOCKS: Lazy<Mutex<HashMap<PathBuf, Weak<()>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Holds an exclusive lock on a path while alive.
pub struct OpfsExclusiveLock {
    _marker: Arc<()>,
    path: PathBuf,
}

impl OpfsExclusiveLock {
    pub fn acquire(canonical_path: PathBuf) -> Option<Self> {
        let mut map = FILE_LOCKS.lock();
        map.retain(|_, w| w.strong_count() > 0);
        if map.contains_key(&canonical_path) {
            return None;
        }
        let marker = Arc::new(());
        map.insert(canonical_path.clone(), Arc::downgrade(&marker));
        Some(Self {
            _marker: marker,
            path: canonical_path,
        })
    }
}

impl Drop for OpfsExclusiveLock {
    fn drop(&mut self) {
        let mut map = FILE_LOCKS.lock();
        map.remove(&self.path);
    }
}

#[derive(Trace, Finalize, JsData)]
pub struct FileSystemSyncAccessHandle {
    #[unsafe_ignore_trace]
    inner: Mutex<SyncInner>,
}

struct SyncInner {
    backend: Arc<OpfsBackend>,
    virtual_path: PathBuf,
    file: Option<std::fs::File>,
    closed: bool,
    _lock: OpfsExclusiveLock,
}

impl std::fmt::Debug for FileSystemSyncAccessHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FileSystemSyncAccessHandle").finish()
    }
}

impl FileSystemSyncAccessHandle {
    pub fn open(
        backend: Arc<OpfsBackend>,
        virtual_path: PathBuf,
    ) -> Result<Self, SyncOpenError> {
        let canonical = backend
            .resolve(&virtual_path)
            .map_err(|_| SyncOpenError::PathInvalid)?;
        let lock = OpfsExclusiveLock::acquire(canonical.clone())
            .ok_or(SyncOpenError::AlreadyLocked)?;
        let file = backend
            .open_file_rw(&virtual_path, true)
            .map_err(SyncOpenError::Io)?;
        Ok(Self {
            inner: Mutex::new(SyncInner {
                backend,
                virtual_path,
                file: Some(file),
                closed: false,
                _lock: lock,
            }),
        })
    }
}

#[derive(Debug)]
pub enum SyncOpenError {
    PathInvalid,
    AlreadyLocked,
    Io(std::io::Error),
}

impl BuiltInObject for FileSystemSyncAccessHandle {
    const NAME: JsString = StaticJsStrings::EMPTY_STRING;
}

impl IntrinsicObject for FileSystemSyncAccessHandle {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::read_js, js_string!("read"), 2)
            .method(Self::write_js, js_string!("write"), 2)
            .method(Self::get_size_js, js_string!("getSize"), 0)
            .method(Self::truncate_js, js_string!("truncate"), 1)
            .method(Self::flush_js, js_string!("flush"), 0)
            .method(Self::close_js, js_string!("close"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInConstructor for FileSystemSyncAccessHandle {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;
    const STANDARD_CONSTRUCTOR: fn(
        &boa_engine::context::intrinsics::StandardConstructors,
    ) -> &boa_engine::context::intrinsics::StandardConstructor =
        boa_engine::context::intrinsics::StandardConstructors::file_system_sync_access_handle;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        Err(JsNativeError::typ()
            .with_message("FileSystemSyncAccessHandle is not directly constructible")
            .into())
    }
}

impl FileSystemSyncAccessHandle {
    fn get_size_js(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("getSize() on non-object"))?;
        let handle = obj
            .downcast_ref::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("getSize() on incompatible object"))?;
        let inner = handle.inner.lock();
        if inner.closed {
            return reject_with(names::INVALID_STATE, "handle is closed", context);
        }
        let file = inner
            .file
            .as_ref()
            .ok_or_else(|| JsNativeError::typ().with_message("file is not open"))?;
        match file.metadata() {
            Ok(m) => resolve_with(JsValue::from(m.len() as f64), context),
            Err(e) => reject_with(super::errors::map_io_error(&e), &format!("{e}"), context),
        }
    }

    fn flush_js(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("flush() on non-object"))?;
        let handle = obj
            .downcast_ref::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("flush() on incompatible object"))?;
        let mut inner = handle.inner.lock();
        if inner.closed {
            return reject_with(names::INVALID_STATE, "handle is closed", context);
        }
        let file = inner
            .file
            .as_mut()
            .ok_or_else(|| JsNativeError::typ().with_message("file is not open"))?;
        match file.sync_all() {
            Ok(_) => resolve_with(JsValue::undefined(), context),
            Err(e) => reject_with(super::errors::map_io_error(&e), &format!("{e}"), context),
        }
    }

    fn close_js(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("close() on non-object"))?;
        let handle = obj
            .downcast_ref::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("close() on incompatible object"))?;
        let mut inner = handle.inner.lock();
        inner.closed = true;
        inner.file = None;
        resolve_with(JsValue::undefined(), context)
    }

    fn truncate_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let new_size = match args.get_or_undefined(0).to_number(context) {
            Ok(n) if n >= 0.0 && n.is_finite() => n as u64,
            _ => return reject_with(names::INVALID_STATE, "invalid truncate size", context),
        };
        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("truncate() on non-object"))?;
        let handle = obj.downcast_ref::<Self>().ok_or_else(|| {
            JsNativeError::typ().with_message("truncate() on incompatible object")
        })?;
        let inner = handle.inner.lock();
        if inner.closed {
            return reject_with(names::INVALID_STATE, "handle is closed", context);
        }
        let file = inner
            .file
            .as_ref()
            .ok_or_else(|| JsNativeError::typ().with_message("file is not open"))?;
        match file.set_len(new_size) {
            Ok(_) => resolve_with(JsValue::undefined(), context),
            Err(e) => reject_with(super::errors::map_io_error(&e), &format!("{e}"), context),
        }
    }

    fn read_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let buffer_arg = args.get_or_undefined(0).clone();
        let at = read_at_option(args.get_or_undefined(1), context)?;
        let buffer_obj = buffer_arg
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("buffer must be a BufferSource"))?
            .clone();

        let (buf_handle, off, max_len) = describe_buffer(buffer_obj, context)?;

        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("read() on non-object"))?;
        let handle = obj
            .downcast_ref::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("read() on incompatible object"))?;
        let mut inner = handle.inner.lock();
        if inner.closed {
            return reject_with(names::INVALID_STATE, "handle is closed", context);
        }
        let file = inner
            .file
            .as_mut()
            .ok_or_else(|| JsNativeError::typ().with_message("file is not open"))?;

        use std::io::{Read, Seek, SeekFrom};
        if let Some(pos) = at {
            if let Err(e) = file.seek(SeekFrom::Start(pos)) {
                return reject_with(super::errors::map_io_error(&e), &format!("{e}"), context);
            }
        }
        let mut tmp = vec![0u8; max_len];
        let bytes_read = match file.read(&mut tmp) {
            Ok(n) => n,
            Err(e) => return reject_with(super::errors::map_io_error(&e), &format!("{e}"), context),
        };
        drop(inner);
        if let Some(mut data) = buf_handle.data_mut() {
            data[off..off + bytes_read].copy_from_slice(&tmp[..bytes_read]);
        }
        resolve_with(JsValue::from(bytes_read as f64), context)
    }

    fn write_js(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let buffer_arg = args.get_or_undefined(0).clone();
        let at = read_at_option(args.get_or_undefined(1), context)?;
        let buffer_obj = buffer_arg
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("buffer must be a BufferSource"))?
            .clone();
        let (buf_handle, off, len) = describe_buffer(buffer_obj, context)?;
        let bytes: Vec<u8> = buf_handle
            .data()
            .map(|d| d[off..off + len].to_vec())
            .unwrap_or_default();

        let obj = this
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("write() on non-object"))?;
        let handle = obj
            .downcast_ref::<Self>()
            .ok_or_else(|| JsNativeError::typ().with_message("write() on incompatible object"))?;
        let mut inner = handle.inner.lock();
        if inner.closed {
            return reject_with(names::INVALID_STATE, "handle is closed", context);
        }
        let file = inner
            .file
            .as_mut()
            .ok_or_else(|| JsNativeError::typ().with_message("file is not open"))?;

        use std::io::{Seek, SeekFrom, Write};
        if let Some(pos) = at {
            if let Err(e) = file.seek(SeekFrom::Start(pos)) {
                return reject_with(super::errors::map_io_error(&e), &format!("{e}"), context);
            }
        }
        let written = match file.write(&bytes) {
            Ok(n) => n,
            Err(e) => return reject_with(super::errors::map_io_error(&e), &format!("{e}"), context),
        };
        // Mark variables used to silence dead-code warnings.
        let _ = &inner.backend;
        let _ = &inner.virtual_path;
        resolve_with(JsValue::from(written as f64), context)
    }
}

fn read_at_option(arg: &JsValue, context: &mut Context) -> JsResult<Option<u64>> {
    if arg.is_undefined() || arg.is_null() {
        return Ok(None);
    }
    if let Some(obj) = arg.as_object() {
        let at = obj.get(js_string!("at"), context)?;
        if at.is_undefined() || at.is_null() {
            return Ok(None);
        }
        let n = at.to_number(context)?;
        if n < 0.0 || !n.is_finite() {
            return Err(JsNativeError::typ()
                .with_message("`at` must be a non-negative finite number")
                .into());
        }
        return Ok(Some(n as u64));
    }
    Ok(None)
}

fn describe_buffer(
    buffer: JsObject,
    context: &mut Context,
) -> JsResult<(JsArrayBuffer, usize, usize)> {
    if let Ok(arr_buf) = JsArrayBuffer::from_object(buffer.clone()) {
        let len = arr_buf.data().map(|d| d.len()).unwrap_or(0);
        return Ok((arr_buf, 0, len));
    }
    if let Ok(ta) = JsTypedArray::from_object(buffer.clone()) {
        let buf_val = ta.buffer(context)?;
        let buf_obj = buf_val
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("typed array has no buffer"))?
            .clone();
        let arr_buf = JsArrayBuffer::from_object(buf_obj)?;
        let off = ta.byte_offset(context)?;
        let len = ta.byte_length(context)?;
        return Ok((arr_buf, off, len));
    }
    Err(JsNativeError::typ()
        .with_message("buffer must be an ArrayBuffer or typed-array view")
        .into())
}

fn resolve_with(value: JsValue, context: &mut Context) -> JsResult<JsValue> {
    let (promise, resolvers) = JsPromise::new_pending(context);
    resolvers
        .resolve
        .call(&JsValue::undefined(), &[value], context)?;
    Ok(JsValue::from(promise))
}
