// ============================================================================
// SECURITY WARNING: real_fs FEATURE
// ============================================================================
#[cfg(feature = "real_fs")]
compile_error!(
    "\n\n\
    ============================================================================\n\
    SECURITY ERROR: The 'real_fs' feature is enabled!\n\
    ============================================================================\n\
    \n\
    This feature bypasses the VFS sandbox and grants FULL FILESYSTEM ACCESS.\n\
    It is intended ONLY for testing and development purposes.\n\
    \n\
    DO NOT use this feature in production builds!\n\
    \n\
    If you truly need real filesystem access for testing:\n\
    1. Acknowledge you understand the security implications\n\
    2. Use the 'real_fs_acknowledged' feature instead\n\
    3. Never ship builds with this feature enabled\n\
    \n\
    To proceed with real_fs for testing only:\n\
    - Use --features real_fs_acknowledged instead of --features real_fs\n\
    ============================================================================\n\n"
);

// When real_fs_acknowledged is enabled, print a warning at runtime on first use
#[cfg(feature = "real_fs_acknowledged")]
mod real_fs_warning {
    use std::sync::Once;
    static WARN_ONCE: Once = Once::new();

    pub fn warn() {
        WARN_ONCE.call_once(|| {
            eprintln!(
                "\n[SECURITY WARNING] VFS real_fs_acknowledged feature is enabled!\n\
                 The VFS sandbox is BYPASSED. Full filesystem access is granted.\n\
                 This should NEVER be used in production.\n"
            );
        });
    }
}

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs as stdfs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use uuid::Uuid;

// Encryption imports for session data at rest
use chacha20poly1305::aead::rand_core::RngCore;
use chacha20poly1305::{
    ChaCha20Poly1305, Nonce,
    aead::{Aead, KeyInit, OsRng},
};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

/// Secret management for session encryption.
///
/// # Security
///
/// This module manages the session secret used for encrypting VFS data.
/// The secret is obtained from (in order of priority):
///
/// 1. `THALORA_SESSION_SECRET` environment variable (recommended for production)
/// 2. Auto-generated secret stored in a secure location (fallback)
///
/// The hardcoded fallback has been REMOVED for security reasons.
mod secret {
    use super::*;

    /// Path to store the auto-generated secret
    fn secret_file_path() -> PathBuf {
        // Try to use a secure location
        if let Some(data_dir) = dirs_next::data_local_dir() {
            let thalora_dir = data_dir.join("thalora");
            return thalora_dir.join(".session_secret");
        }

        // Fallback to temp directory (less secure but always available)
        std::env::temp_dir().join(".thalora_session_secret")
    }

    /// Generate a cryptographically secure random secret
    fn generate_secret() -> String {
        let mut bytes = [0u8; 32];
        OsRng.fill_bytes(&mut bytes);
        // Use hex encoding for readability and easy storage
        bytes.iter().map(|b| format!("{:02x}", b)).collect()
    }

    /// Load or generate the session secret.
    ///
    /// # Security
    ///
    /// - In production, set THALORA_SESSION_SECRET environment variable
    /// - If not set, generates a random secret and stores it on first use
    /// - The stored secret is created with restricted permissions
    pub fn get_session_secret() -> String {
        // Priority 1: Environment variable
        if let Ok(secret) = std::env::var("THALORA_SESSION_SECRET") {
            if secret.len() >= 32 {
                return secret;
            }
            // Secret too short, log warning and continue to fallback
            #[cfg(debug_assertions)]
            eprintln!(
                "[SECURITY WARNING] THALORA_SESSION_SECRET is too short ({} chars). \
                 Minimum 32 characters required. Using auto-generated secret.",
                secret.len()
            );
        }

        // Priority 2: Stored secret file
        let secret_path = secret_file_path();
        if let Ok(secret) = stdfs::read_to_string(&secret_path) {
            let secret = secret.trim().to_string();
            if secret.len() >= 32 {
                return secret;
            }
        }

        // Priority 3: Generate new secret and store it
        let new_secret = generate_secret();

        // Try to create parent directory
        if let Some(parent) = secret_path.parent() {
            let _ = stdfs::create_dir_all(parent);
        }

        // Write the secret with restricted permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            if let Ok(mut file) = stdfs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600) // Owner read/write only
                .open(&secret_path)
            {
                use std::io::Write;
                let _ = file.write_all(new_secret.as_bytes());
            }
        }

        #[cfg(not(unix))]
        {
            // On Windows, just write the file
            let _ = stdfs::write(&secret_path, &new_secret);
        }

        #[cfg(debug_assertions)]
        eprintln!(
            "[SECURITY WARNING] Using auto-generated session secret. \
             For production, set THALORA_SESSION_SECRET environment variable \
             to a cryptographically random 32+ byte value."
        );

        new_secret
    }
}

/// Derive a 256-bit encryption key from a session ID and the session secret.
///
/// # Security
///
/// Uses SHA-256 to derive a key from:
/// - The session secret (from environment variable or auto-generated)
/// - The session_id (for session-specific key derivation)
///
/// The session secret is obtained from:
/// 1. THALORA_SESSION_SECRET environment variable (recommended for production)
/// 2. Auto-generated and persisted secret (fallback)
///
/// **IMPORTANT**: In production deployments, always set THALORA_SESSION_SECRET
/// to a cryptographically random 32+ byte value.
pub fn derive_session_key(session_id: &str) -> Zeroizing<[u8; 32]> {
    // Get secret from secure source (no more hardcoded fallback!)
    let secret = secret::get_session_secret();

    // Derive key using SHA-256(secret || session_id)
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(session_id.as_bytes());
    let result = hasher.finalize();

    let mut key = Zeroizing::new([0u8; 32]);
    key.copy_from_slice(&result);
    key
}

/// Check if the session secret is properly configured.
///
/// Returns `true` if THALORA_SESSION_SECRET is set and has sufficient length.
/// This can be used to warn users in production environments.
pub fn is_session_secret_configured() -> bool {
    std::env::var("THALORA_SESSION_SECRET")
        .map(|s| s.len() >= 32)
        .unwrap_or(false)
}

// =============================================================================
// PATH SECURITY
// =============================================================================

/// Normalize a path by removing `.` and resolving `..` components.
///
/// # Security
///
/// This function prevents path traversal attacks by:
/// - Removing `.` (current directory) components
/// - Resolving `..` (parent directory) components without going above root
/// - Rejecting paths with null bytes
///
/// This provides defense-in-depth for VFS operations.
///
/// # Returns
///
/// Returns `Some(normalized_path)` if the path is safe, `None` if the path
/// is malicious (e.g., contains null bytes or attempts to escape root).
pub fn normalize_path(path: &Path) -> Option<PathBuf> {
    // Check for null bytes (path injection attack)
    if path.to_string_lossy().contains('\0') {
        return None;
    }

    let mut normalized = PathBuf::new();
    let mut depth: i32 = 0;

    for component in path.components() {
        match component {
            std::path::Component::Normal(c) => {
                // Check for null bytes in component
                if c.to_string_lossy().contains('\0') {
                    return None;
                }
                normalized.push(c);
                depth += 1;
            }
            std::path::Component::RootDir => {
                normalized.push("/");
            }
            std::path::Component::CurDir => {
                // Skip `.` - no effect on path
            }
            std::path::Component::ParentDir => {
                // Only go up if we have depth to spare (prevent escaping root)
                if depth > 0 {
                    normalized.pop();
                    depth -= 1;
                }
                // If depth is 0, silently ignore (can't go above root)
            }
            std::path::Component::Prefix(p) => {
                normalized.push(p.as_os_str());
            }
        }
    }

    // Ensure we have at least root or the path isn't empty
    if normalized.as_os_str().is_empty() {
        normalized.push("/");
    }

    Some(normalized)
}

/// Validate and normalize a path, returning an error if the path is malicious.
///
/// # Security
///
/// This function should be called on all user-provided paths before using them
/// in VFS operations.
pub fn validate_path(path: &Path) -> io::Result<PathBuf> {
    normalize_path(path).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid path: contains forbidden characters or traversal sequences",
        )
    })
}

// =============================================================================
// VFS ENTRY TYPE
// =============================================================================

/// Current Unix timestamp in milliseconds.
fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

/// A single entry in the virtual filesystem.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VfsEntry {
    pub data: Vec<u8>,
    pub is_dir: bool,
    pub created: u64,
    pub modified: u64,
    pub accessed: u64,
}

impl VfsEntry {
    /// Create a new file entry with the given data and current timestamps.
    fn new_file(data: Vec<u8>) -> Self {
        let now = now_millis();
        Self {
            data,
            is_dir: false,
            created: now,
            modified: now,
            accessed: now,
        }
    }

    /// Create a new directory entry with current timestamps.
    fn new_dir() -> Self {
        let now = now_millis();
        Self {
            data: Vec::new(),
            is_dir: true,
            created: now,
            modified: now,
            accessed: now,
        }
    }
}

/// Type alias for the underlying VFS map.
type VfsMap = HashMap<PathBuf, VfsEntry>;

// =============================================================================
// VFS STORAGE
// =============================================================================

#[cfg(not(feature = "real_fs"))]
static IN_MEM_FILES: Lazy<Arc<Mutex<VfsMap>>> = Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

// =============================================================================
// VFS PERSISTENCE
// =============================================================================

/// Serialization format version 1 (legacy: raw bytes).
#[derive(Serialize, Deserialize)]
struct VfsPersistV1 {
    entries: Vec<(PathBuf, Vec<u8>)>,
}

/// Serialization format version 2 (VfsEntry with metadata).
#[derive(Serialize, Deserialize)]
struct VfsPersistV2 {
    version: u8,
    entries: Vec<(PathBuf, VfsEntry)>,
}

/// Serialize a VfsMap to bytes using the v2 format.
fn serialize_vfs_map(map: &VfsMap) -> io::Result<Vec<u8>> {
    let entries: Vec<(PathBuf, VfsEntry)> =
        map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let persist = VfsPersistV2 {
        version: 2,
        entries,
    };
    bincode::serialize(&persist).map_err(io::Error::other)
}

/// Deserialize bytes into a VfsMap, supporting both v1 and v2 formats.
fn deserialize_vfs_map(bytes: &[u8]) -> io::Result<VfsMap> {
    // Try v2 first
    if let Ok(v2) = bincode::deserialize::<VfsPersistV2>(bytes)
        && v2.version == 2
    {
        let mut map = HashMap::new();
        for (k, v) in v2.entries {
            map.insert(k, v);
        }
        return Ok(map);
    }
    // Fall back to v1
    let v1: VfsPersistV1 =
        bincode::deserialize(bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    let mut map = HashMap::new();
    for (k, v) in v1.entries {
        map.insert(k, VfsEntry::new_file(v));
    }
    Ok(map)
}

// =============================================================================
// VFS INSTANCE
// =============================================================================

/// File-backed VFS instance persisted in a single binary file.
#[derive(Debug, Clone)]
pub struct VfsInstance {
    file_path: PathBuf,
    map: Arc<Mutex<VfsMap>>,
    /// Maximum total bytes stored across all entries (None = unlimited).
    quota: Option<u64>,
    /// Maximum size of a single file in bytes (None = unlimited).
    max_file_size: Option<u64>,
    /// Maximum number of entries (files + directories) (None = unlimited).
    max_files: Option<u64>,
}

impl VfsInstance {
    /// Create a new file-backed VFS at the provided path. If the file exists it will be loaded.
    pub fn open_file_backed<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let p = path.as_ref().to_path_buf();
        if p.exists() {
            let bytes = stdfs::read(&p)?;
            let map = deserialize_vfs_map(&bytes)?;
            Ok(Self {
                file_path: p,
                map: Arc::new(Mutex::new(map)),
                quota: None,
                max_file_size: None,
                max_files: None,
            })
        } else {
            Ok(Self {
                file_path: p,
                map: Arc::new(Mutex::new(HashMap::new())),
                quota: None,
                max_file_size: None,
                max_files: None,
            })
        }
    }

    /// Create a new temporary file-backed VFS with a unique filename in `dir`.
    pub fn new_temp_in_dir<P: AsRef<Path>>(dir: P) -> io::Result<Self> {
        let id = Uuid::new_v4().to_string();
        let file = dir.as_ref().join(format!("vfs-{}.bin", id));
        Ok(Self {
            file_path: file,
            map: Arc::new(Mutex::new(HashMap::new())),
            quota: None,
            max_file_size: None,
            max_files: None,
        })
    }

    /// Persist current in-memory map to disk atomically.
    pub fn persist(&self) -> io::Result<()> {
        let map = self.map.lock().unwrap();
        let bytes = serialize_vfs_map(&map)?;
        let tmp = self.file_path.with_extension("tmp");
        stdfs::write(&tmp, &bytes)?;
        stdfs::rename(&tmp, &self.file_path)?;
        Ok(())
    }

    /// Delete the backing file if present.
    pub fn delete_backing_file(&self) -> io::Result<()> {
        if self.file_path.exists() {
            stdfs::remove_file(&self.file_path)?;
        }
        Ok(())
    }

    pub fn as_map(&self) -> Arc<Mutex<VfsMap>> {
        self.map.clone()
    }

    /// Return the backing file path for this VFS instance.
    pub fn backing_path(&self) -> PathBuf {
        self.file_path.clone()
    }

    // --- Quota API ---

    /// Get current total data usage in bytes.
    pub fn usage(&self) -> u64 {
        let map = self.map.lock().unwrap();
        map.values().map(|e| e.data.len() as u64).sum()
    }

    /// Get the configured quota (None = unlimited).
    pub fn quota(&self) -> Option<u64> {
        self.quota
    }

    /// Set the total storage quota in bytes. Pass `None` for unlimited.
    pub fn set_quota(&mut self, bytes: Option<u64>) {
        self.quota = bytes;
    }

    /// Set the maximum single file size in bytes. Pass `None` for unlimited.
    pub fn set_max_file_size(&mut self, bytes: Option<u64>) {
        self.max_file_size = bytes;
    }

    /// Set the maximum number of entries. Pass `None` for unlimited.
    pub fn set_max_files(&mut self, count: Option<u64>) {
        self.max_files = count;
    }

    // === ENCRYPTED PERSISTENCE METHODS ===

    /// Open an encrypted file-backed VFS. If the file exists it will be decrypted and loaded.
    /// The key should be a 256-bit (32-byte) key derived from a session secret.
    ///
    /// # Security
    /// Uses ChaCha20-Poly1305 AEAD cipher for authenticated encryption.
    /// The nonce is stored as the first 12 bytes of the encrypted file.
    pub fn open_file_backed_encrypted<P: AsRef<Path>>(path: P, key: &[u8; 32]) -> io::Result<Self> {
        let p = path.as_ref().to_path_buf();
        if p.exists() {
            let encrypted_bytes = stdfs::read(&p)?;

            // File format: [12-byte nonce][ciphertext with auth tag]
            if encrypted_bytes.len() < 12 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "encrypted file too short",
                ));
            }

            let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
            let nonce = Nonce::from_slice(nonce_bytes);

            // Create cipher with zeroizing key wrapper
            let key_array = Zeroizing::new(*key);
            let cipher = ChaCha20Poly1305::new_from_slice(&*key_array)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

            // Decrypt the data
            let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "decryption failed: invalid key or corrupted data",
                )
            })?;

            // Deserialize the VFS data (supports both v1 and v2)
            let map = deserialize_vfs_map(&plaintext)?;
            Ok(Self {
                file_path: p,
                map: Arc::new(Mutex::new(map)),
                quota: None,
                max_file_size: None,
                max_files: None,
            })
        } else {
            Ok(Self {
                file_path: p,
                map: Arc::new(Mutex::new(HashMap::new())),
                quota: None,
                max_file_size: None,
                max_files: None,
            })
        }
    }

    /// Persist current in-memory map to disk with encryption.
    /// The key should be a 256-bit (32-byte) key derived from a session secret.
    ///
    /// # Security
    /// Uses ChaCha20-Poly1305 AEAD cipher for authenticated encryption.
    /// A random 96-bit nonce is generated for each persist operation.
    pub fn persist_encrypted(&self, key: &[u8; 32]) -> io::Result<()> {
        let map = self.map.lock().unwrap();
        let plaintext = serialize_vfs_map(&map)?;

        // Generate a random 96-bit nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher with zeroizing key wrapper
        let key_array = Zeroizing::new(*key);
        let cipher = ChaCha20Poly1305::new_from_slice(&*key_array)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

        // Encrypt the data
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_slice())
            .map_err(|e| io::Error::other(e.to_string()))?;

        // Write file format: [12-byte nonce][ciphertext with auth tag]
        let mut encrypted_bytes = Vec::with_capacity(12 + ciphertext.len());
        encrypted_bytes.extend_from_slice(&nonce_bytes);
        encrypted_bytes.extend_from_slice(&ciphertext);

        // Atomic write via temp file
        let tmp = self.file_path.with_extension("tmp.enc");
        stdfs::write(&tmp, &encrypted_bytes)?;
        stdfs::rename(&tmp, &self.file_path)?;
        Ok(())
    }
}

// =============================================================================
// GLOBAL VFS INSTANCE
// =============================================================================

// Global current VFS instance (optional)
static CURRENT_VFS: Lazy<Mutex<Option<Arc<VfsInstance>>>> = Lazy::new(|| Mutex::new(None));

/// Set the current VFS instance for this process. Returns the previous instance.
pub fn set_current_vfs(v: Option<Arc<VfsInstance>>) -> Option<Arc<VfsInstance>> {
    let mut cur = CURRENT_VFS.lock().unwrap();
    let prev = cur.clone();
    *cur = v;
    prev
}

pub fn get_current_vfs() -> Option<Arc<VfsInstance>> {
    let cur = CURRENT_VFS.lock().unwrap();
    cur.clone()
}

// =============================================================================
// QUOTA ENFORCEMENT HELPERS
// =============================================================================

#[cfg(not(feature = "real_fs"))]
fn quota_error(msg: &str) -> io::Error {
    io::Error::other(msg)
}

/// Check quota constraints before writing `new_data_len` bytes to `path` in `map`.
/// `vfs` is the current VfsInstance (if any) providing quota settings.
#[cfg(not(feature = "real_fs"))]
fn check_quota(
    map: &VfsMap,
    path: &Path,
    new_data_len: u64,
    vfs: Option<&VfsInstance>,
) -> io::Result<()> {
    let vfs = match vfs {
        Some(v) => v,
        None => return Ok(()), // No VFS instance = no quota (global fallback)
    };

    // Check max_file_size
    if let Some(max) = vfs.max_file_size
        && new_data_len > max
    {
        return Err(quota_error("file size exceeds maximum allowed"));
    }

    // Check max_files (only if this is a new entry)
    if let Some(max) = vfs.max_files
        && !map.contains_key(path)
        && map.len() as u64 >= max
    {
        return Err(quota_error("maximum number of files exceeded"));
    }

    // Check total quota
    if let Some(quota) = vfs.quota {
        let current_usage: u64 = map.values().map(|e| e.data.len() as u64).sum();
        // Subtract the existing entry size if overwriting
        let existing = map.get(path).map(|e| e.data.len() as u64).unwrap_or(0);
        let new_total = current_usage - existing + new_data_len;
        if new_total > quota {
            return Err(quota_error("storage quota exceeded"));
        }
    }

    Ok(())
}

// =============================================================================
// REAL FS MODULE (acknowledged variant for testing)
// =============================================================================

// real_fs feature intentionally causes compile error (see top of file)
// real_fs_acknowledged is the acknowledged variant for testing
#[cfg(feature = "real_fs_acknowledged")]
pub mod fs {
    pub use std::fs::*;
    use std::path::Path;

    /// Initialize the real_fs module (prints security warning)
    fn init_warning() {
        #[cfg(feature = "real_fs_acknowledged")]
        super::real_fs_warning::warn();
    }

    /// Check if a path exists (wrapper for Path::exists for API compatibility)
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        init_warning();
        path.as_ref().exists()
    }

    // Re-export with warning on first use
    pub fn read_to_string<P: AsRef<Path>>(path: P) -> std::io::Result<String> {
        init_warning();
        std::fs::read_to_string(path)
    }

    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> std::io::Result<()> {
        init_warning();
        std::fs::write(path, contents)
    }

    pub fn read<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
        init_warning();
        std::fs::read(path)
    }

    pub fn create_dir_all<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        init_warning();
        std::fs::create_dir_all(path)
    }

    pub fn metadata<P: AsRef<Path>>(path: P) -> std::io::Result<std::fs::Metadata> {
        init_warning();
        std::fs::metadata(path)
    }

    pub fn remove_file<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        init_warning();
        std::fs::remove_file(path)
    }

    pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
        init_warning();
        std::fs::remove_dir_all(path)
    }

    pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> std::io::Result<()> {
        init_warning();
        std::fs::rename(from, to)
    }

    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> std::io::Result<u64> {
        init_warning();
        std::fs::copy(from, to)
    }

    pub fn read_dir<P: AsRef<Path>>(path: P) -> std::io::Result<std::fs::ReadDir> {
        init_warning();
        std::fs::read_dir(path)
    }
}

// =============================================================================
// IN-MEMORY FS MODULE
// =============================================================================

#[cfg(not(feature = "real_fs_acknowledged"))]
pub mod fs {
    use super::*;

    /// Return the active VFS map: from the current VfsInstance if set, otherwise the global fallback.
    pub(crate) fn active_map() -> Arc<Mutex<VfsMap>> {
        if let Some(vfs) = get_current_vfs() {
            vfs.as_map()
        } else {
            Arc::clone(&IN_MEM_FILES)
        }
    }

    /// Ensure all ancestor directories of `path` exist as explicit directory entries.
    pub(crate) fn ensure_parent_dirs(map: &mut VfsMap, path: &Path) {
        if let Some(parent) = path.parent() {
            let mut ancestors = Vec::new();
            let mut cur = parent;
            loop {
                // Stop at root or if directory already exists
                if cur.as_os_str().is_empty() || cur.as_os_str() == "/" {
                    break;
                }
                if map.get(cur).is_some_and(|e| e.is_dir) {
                    break;
                }
                ancestors.push(cur.to_path_buf());
                match cur.parent() {
                    Some(p) => cur = p,
                    None => break,
                }
            }
            for dir_path in ancestors.into_iter().rev() {
                map.entry(dir_path).or_insert_with(VfsEntry::new_dir);
            }
        }
    }

    /// Check if `path` is a directory — either an explicit dir entry, or has descendants.
    fn is_dir_in_map(map: &VfsMap, p: &Path) -> bool {
        if let Some(entry) = map.get(p) {
            return entry.is_dir;
        }
        // Check for implicit directory (has descendants)
        map.keys().any(|k| {
            if let Some(normalized_k) = normalize_path(k) {
                normalized_k.starts_with(p) && normalized_k != p
            } else {
                false
            }
        })
    }

    // --- Directory operations ---

    pub fn create_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let p = validate_path(path.as_ref())?;
        let map_arc = active_map();
        let mut map = map_arc.lock().unwrap();
        // Create the target and all ancestors
        ensure_parent_dirs(&mut map, &p.join("_placeholder"));
        map.entry(p).or_insert_with(VfsEntry::new_dir);
        Ok(())
    }

    pub fn create_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let p = validate_path(path.as_ref())?;
        let map_arc = active_map();
        let mut map = map_arc.lock().unwrap();
        // Check parent exists
        if let Some(parent) = p.parent()
            && !parent.as_os_str().is_empty()
            && parent.as_os_str() != "/"
            && !is_dir_in_map(&map, parent)
        {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                "parent directory does not exist",
            ));
        }
        if map.contains_key(&p) {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "path already exists",
            ));
        }
        map.insert(p, VfsEntry::new_dir());
        Ok(())
    }

    pub fn remove_dir<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let p = validate_path(path.as_ref())?;
        let map_arc = active_map();
        let mut map = map_arc.lock().unwrap();
        match map.get(&p) {
            Some(entry) if entry.is_dir => {}
            Some(_) => {
                return Err(io::Error::other("not a directory"));
            }
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "directory not found",
                ));
            }
        }
        // Check if directory has children
        let has_children = map.keys().any(|k| k != &p && k.starts_with(&p));
        if has_children {
            return Err(io::Error::other("directory not empty"));
        }
        map.remove(&p);
        Ok(())
    }

    pub fn remove_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let p = validate_path(path.as_ref())?;
        let map_arc = active_map();
        let mut map = map_arc.lock().unwrap();
        // Collect all keys that are the path itself or descendants
        let to_remove: Vec<PathBuf> = map
            .keys()
            .filter(|k| *k == &p || k.starts_with(&p))
            .cloned()
            .collect();
        if to_remove.is_empty() {
            return Err(io::Error::new(io::ErrorKind::NotFound, "path not found"));
        }
        for k in to_remove {
            map.remove(&k);
        }
        Ok(())
    }

    // --- File read/write operations ---

    pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
        let p = validate_path(path.as_ref())?;
        let map_arc = active_map();
        let mut map = map_arc.lock().unwrap();
        match map.get_mut(&p) {
            Some(entry) if !entry.is_dir => {
                entry.accessed = now_millis();
                Ok(String::from_utf8_lossy(&entry.data).to_string())
            }
            Some(_) => Err(io::Error::other("is a directory")),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
        }
    }

    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
        let p = validate_path(path.as_ref())?;
        let data = contents.as_ref().to_vec();
        let map_arc = active_map();
        let mut map = map_arc.lock().unwrap();

        // Quota check
        let vfs = get_current_vfs();
        check_quota(&map, &p, data.len() as u64, vfs.as_deref())?;

        // Auto-create parent directories
        ensure_parent_dirs(&mut map, &p);

        let now = now_millis();
        if let Some(entry) = map.get_mut(&p) {
            if entry.is_dir {
                return Err(io::Error::other("is a directory"));
            }
            entry.data = data;
            entry.modified = now;
            entry.accessed = now;
        } else {
            map.insert(p, VfsEntry::new_file(data));
        }
        Ok(())
    }

    pub fn read<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
        let p = validate_path(path.as_ref())?;
        let map_arc = active_map();
        let mut map = map_arc.lock().unwrap();
        match map.get_mut(&p) {
            Some(entry) if !entry.is_dir => {
                entry.accessed = now_millis();
                Ok(entry.data.clone())
            }
            Some(_) => Err(io::Error::other("is a directory")),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
        }
    }

    // --- File management operations ---

    pub fn remove_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let p = validate_path(path.as_ref())?;
        let map_arc = active_map();
        let mut map = map_arc.lock().unwrap();
        match map.get(&p) {
            Some(entry) if entry.is_dir => {
                return Err(io::Error::other("is a directory"));
            }
            None => {
                return Err(io::Error::new(io::ErrorKind::NotFound, "file not found"));
            }
            _ => {}
        }
        map.remove(&p);
        Ok(())
    }

    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
        let fromp = validate_path(from.as_ref())?;
        let top = validate_path(to.as_ref())?;
        let map_arc = active_map();
        let mut map = map_arc.lock().unwrap();
        let source = map
            .get(&fromp)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "source file not found"))?;
        if source.is_dir {
            return Err(io::Error::other("cannot copy a directory"));
        }
        let data = source.data.clone();
        let len = data.len() as u64;

        // Quota check
        let vfs = get_current_vfs();
        check_quota(&map, &top, len, vfs.as_deref())?;

        ensure_parent_dirs(&mut map, &top);
        map.insert(top, VfsEntry::new_file(data));
        Ok(len)
    }

    pub fn rename<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<()> {
        let fromp = validate_path(from.as_ref())?;
        let top = validate_path(to.as_ref())?;
        let map_arc = active_map();
        let mut map = map_arc.lock().unwrap();

        if !map.contains_key(&fromp) && !is_dir_in_map(&map, &fromp) {
            return Err(io::Error::new(io::ErrorKind::NotFound, "source not found"));
        }

        // Collect all entries to rename (the entry itself + descendants for directories)
        let to_rename: Vec<(PathBuf, VfsEntry)> = map
            .iter()
            .filter(|(k, _)| *k == &fromp || k.starts_with(&fromp))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();

        ensure_parent_dirs(&mut map, &top);

        for (old_path, entry) in &to_rename {
            map.remove(old_path);
            let relative = old_path.strip_prefix(&fromp).unwrap_or(Path::new(""));
            let new_path = if relative.as_os_str().is_empty() {
                top.clone()
            } else {
                top.join(relative)
            };
            map.insert(new_path, entry.clone());
        }
        Ok(())
    }

    // --- Metadata ---

    /// A metadata representation for in-memory VFS with timestamps.
    #[derive(Debug, Clone)]
    pub struct Metadata {
        is_dir: bool,
        len: u64,
        created_ms: u64,
        modified_ms: u64,
        accessed_ms: u64,
    }

    impl Metadata {
        pub fn len(&self) -> u64 {
            self.len
        }

        pub fn is_empty(&self) -> bool {
            self.len == 0
        }

        pub fn is_dir(&self) -> bool {
            self.is_dir
        }

        pub fn is_file(&self) -> bool {
            !self.is_dir
        }

        pub fn created(&self) -> io::Result<SystemTime> {
            Ok(UNIX_EPOCH + Duration::from_millis(self.created_ms))
        }

        pub fn modified(&self) -> io::Result<SystemTime> {
            Ok(UNIX_EPOCH + Duration::from_millis(self.modified_ms))
        }

        pub fn accessed(&self) -> io::Result<SystemTime> {
            Ok(UNIX_EPOCH + Duration::from_millis(self.accessed_ms))
        }
    }

    pub fn metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata> {
        let p = validate_path(path.as_ref())?;
        let map_arc = active_map();
        let map = map_arc.lock().unwrap();
        if let Some(entry) = map.get(&p) {
            Ok(Metadata {
                is_dir: entry.is_dir,
                len: entry.data.len() as u64,
                created_ms: entry.created,
                modified_ms: entry.modified,
                accessed_ms: entry.accessed,
            })
        } else {
            // Check if any file is a descendant -> treat as implicit dir
            // SECURITY: Use normalized path for comparison
            let is_dir = map.keys().any(|k| {
                if let Some(normalized_k) = normalize_path(k) {
                    normalized_k.starts_with(&p) && normalized_k != p
                } else {
                    false
                }
            });
            if is_dir {
                let now = now_millis();
                Ok(Metadata {
                    is_dir: true,
                    len: 0,
                    created_ms: now,
                    modified_ms: now,
                    accessed_ms: now,
                })
            } else {
                Err(io::Error::new(io::ErrorKind::NotFound, "path not found"))
            }
        }
    }

    // --- Directory listing ---

    /// Directory entry for in-memory VFS
    #[derive(Debug, Clone)]
    pub struct DirEntry {
        path: PathBuf,
    }

    impl DirEntry {
        pub fn path(&self) -> PathBuf {
            self.path.clone()
        }

        pub fn metadata(&self) -> io::Result<Metadata> {
            super::fs::metadata(&self.path)
        }

        pub fn file_name(&self) -> std::ffi::OsString {
            self.path
                .file_name()
                .unwrap_or(self.path.as_os_str())
                .to_os_string()
        }
    }

    /// ReadDir iterator for in-memory VFS
    pub struct ReadDir {
        entries: Vec<DirEntry>,
        idx: usize,
    }

    impl Iterator for ReadDir {
        type Item = io::Result<DirEntry>;

        fn next(&mut self) -> Option<Self::Item> {
            if self.idx >= self.entries.len() {
                None
            } else {
                let e = self.entries[self.idx].clone();
                self.idx += 1;
                Some(Ok(e))
            }
        }
    }

    /// List immediate children (files and directories) of `path` within the in-memory VFS.
    pub fn read_dir<P: AsRef<Path>>(path: P) -> io::Result<ReadDir> {
        let root = validate_path(path.as_ref())?;
        let map_arc = active_map();
        let map = map_arc.lock().unwrap();

        use std::collections::HashSet;
        let mut children: HashSet<PathBuf> = HashSet::new();

        for file_path in map.keys() {
            // SECURITY: Normalize file paths before comparison
            let normalized_path = match normalize_path(file_path) {
                Some(p) => p,
                None => continue, // Skip malformed paths
            };
            if let Ok(relative) = normalized_path.strip_prefix(&root) {
                let mut comps = relative.components();
                if let Some(first) = comps.next() {
                    children.insert(root.join(first.as_os_str()));
                }
            }
        }

        let mut entries = Vec::new();
        for path in children {
            entries.push(DirEntry { path });
        }

        Ok(ReadDir { entries, idx: 0 })
    }

    // --- Existence check ---

    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        // Validate and normalize path; return false for malicious paths
        let p = match validate_path(path.as_ref()) {
            Ok(p) => p,
            Err(_) => return false,
        };
        let map_arc = active_map();
        let map = map_arc.lock().unwrap();
        // Check for exact match (file or directory entry)
        if map.contains_key(&p) {
            return true;
        }
        // Check for implicit directory (has descendants)
        is_dir_in_map(&map, &p)
    }

    /// Canonicalize a path within the VFS (normalize + verify existence).
    pub fn canonicalize<P: AsRef<Path>>(path: P) -> io::Result<PathBuf> {
        let p = validate_path(path.as_ref())?;
        if exists(&p) {
            Ok(p)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "path not found"))
        }
    }
}

// =============================================================================
// FILE HANDLE API
// =============================================================================

#[cfg(not(feature = "real_fs"))]
use std::io::{Read, Seek, SeekFrom, Write};

#[cfg(not(feature = "real_fs"))]
pub struct File {
    path: PathBuf,
    pos: usize,
    map: Arc<Mutex<VfsMap>>,
}

#[cfg(not(feature = "real_fs"))]
impl File {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let p = validate_path(path.as_ref())?;
        let map = fs::active_map();
        {
            let m = map.lock().unwrap();
            match m.get(&p) {
                Some(entry) if !entry.is_dir => {}
                Some(_) => {
                    return Err(io::Error::other("is a directory"));
                }
                None => {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "file not found"));
                }
            }
        }
        Ok(Self {
            path: p,
            pos: 0,
            map,
        })
    }

    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let p = validate_path(path.as_ref())?;
        let map = fs::active_map();
        {
            let mut m = map.lock().unwrap();
            // Quota check for new empty file
            let vfs = get_current_vfs();
            check_quota(&m, &p, 0, vfs.as_deref())?;
            fs::ensure_parent_dirs(&mut m, &p);
            m.insert(p.clone(), VfsEntry::new_file(Vec::new()));
        }
        Ok(Self {
            path: p,
            pos: 0,
            map,
        })
    }
}

#[cfg(not(feature = "real_fs"))]
impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut map = self.map.lock().unwrap();
        let entry = map
            .get_mut(&self.path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))?;
        let remaining = &entry.data[self.pos..];
        let n = std::cmp::min(remaining.len(), buf.len());
        buf[..n].copy_from_slice(&remaining[..n]);
        self.pos += n;
        entry.accessed = now_millis();
        Ok(n)
    }
}

#[cfg(not(feature = "real_fs"))]
impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let map = self.map.lock().unwrap();
        let entry = map
            .get(&self.path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))?;
        // SECURITY: Use checked arithmetic to prevent integer overflow (CWE-190)
        let new = match pos {
            SeekFrom::Start(off) => i64::try_from(off).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "seek offset too large")
            })?,
            SeekFrom::End(off) => {
                let len = i64::try_from(entry.data.len())
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "file too large"))?;
                len.checked_add(off)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek overflow"))?
            }
            SeekFrom::Current(off) => {
                let pos = i64::try_from(self.pos).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "position too large")
                })?;
                pos.checked_add(off)
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek overflow"))?
            }
        };
        if new < 0 {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid seek"));
        }
        // SECURITY: Safe conversion since we've verified new >= 0
        self.pos = usize::try_from(new)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "seek result too large"))?;
        Ok(self.pos as u64)
    }
}

#[cfg(not(feature = "real_fs"))]
impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut map = self.map.lock().unwrap();

        // Quota check: compute what the new file size would be
        {
            let entry = map
                .get(&self.path)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))?;
            let end = self.pos + buf.len();
            let new_len = std::cmp::max(entry.data.len(), end) as u64;
            let vfs = get_current_vfs();
            check_quota(&map, &self.path, new_len, vfs.as_deref())?;
        }

        let entry = map
            .get_mut(&self.path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))?;
        if self.pos >= entry.data.len() {
            entry.data.extend_from_slice(buf);
            self.pos = entry.data.len();
        } else {
            let end = self.pos + buf.len();
            if end > entry.data.len() {
                entry.data.resize(end, 0);
            }
            entry.data[self.pos..end].copy_from_slice(buf);
            self.pos = end;
        }
        entry.modified = now_millis();
        entry.accessed = now_millis();
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(not(feature = "real_fs"))]
#[derive(Default)]
pub struct OpenOptions {
    read: bool,
    write: bool,
    create: bool,
    truncate: bool,
    append: bool,
}

#[cfg(not(feature = "real_fs"))]
impl OpenOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn read(&mut self, v: bool) -> &mut Self {
        self.read = v;
        self
    }
    pub fn write(&mut self, v: bool) -> &mut Self {
        self.write = v;
        self
    }
    pub fn create(&mut self, v: bool) -> &mut Self {
        self.create = v;
        self
    }
    pub fn truncate(&mut self, v: bool) -> &mut Self {
        self.truncate = v;
        self
    }
    pub fn append(&mut self, v: bool) -> &mut Self {
        self.append = v;
        self
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        let p = validate_path(path.as_ref())?;
        let map_arc = fs::active_map();
        let initial_pos;
        {
            let mut map = map_arc.lock().unwrap();
            if self.create && !map.contains_key(&p) {
                let vfs = get_current_vfs();
                check_quota(&map, &p, 0, vfs.as_deref())?;
                fs::ensure_parent_dirs(&mut map, &p);
                map.insert(p.clone(), VfsEntry::new_file(Vec::new()));
            }
            if self.truncate
                && let Some(entry) = map.get_mut(&p)
            {
                entry.data.clear();
                entry.modified = now_millis();
            }
            match map.get(&p) {
                Some(entry) if entry.is_dir => {
                    return Err(io::Error::other("is a directory"));
                }
                Some(entry) => {
                    initial_pos = if self.append { entry.data.len() } else { 0 };
                }
                None => {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "file not found"));
                }
            }
        }
        Ok(File {
            path: p,
            pos: initial_pos,
            map: map_arc,
        })
    }
}

#[cfg(test)]
mod tests;
