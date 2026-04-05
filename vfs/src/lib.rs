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
#[allow(dead_code)]
use std::collections::HashMap;
use std::fs as stdfs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
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

#[cfg(not(feature = "real_fs"))]
static IN_MEM_FILES: Lazy<Mutex<HashMap<PathBuf, Vec<u8>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// File-backed VFS instance persisted in a single binary file.
#[derive(Debug, Clone)]
pub struct VfsInstance {
    file_path: PathBuf,
    map: Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>,
}

#[derive(Serialize, Deserialize)]
struct VfsPersist {
    entries: Vec<(PathBuf, Vec<u8>)>,
}

impl VfsInstance {
    /// Create a new file-backed VFS at the provided path. If the file exists it will be loaded.
    pub fn open_file_backed<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let p = path.as_ref().to_path_buf();
        if p.exists() {
            let bytes = stdfs::read(&p)?;
            let persist: VfsPersist = bincode::deserialize(&bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let mut map = HashMap::new();
            for (k, v) in persist.entries.into_iter() {
                map.insert(k, v);
            }
            Ok(Self {
                file_path: p,
                map: Arc::new(Mutex::new(map)),
            })
        } else {
            Ok(Self {
                file_path: p,
                map: Arc::new(Mutex::new(HashMap::new())),
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
        })
    }

    /// Persist current in-memory map to disk atomically.
    pub fn persist(&self) -> io::Result<()> {
        let map = self.map.lock().unwrap();
        let mut entries = Vec::new();
        for (k, v) in map.iter() {
            entries.push((k.clone(), v.clone()));
        }
        let persist = VfsPersist { entries };
        let bytes = bincode::serialize(&persist).map_err(io::Error::other)?;
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

    pub fn as_map(&self) -> Arc<Mutex<HashMap<PathBuf, Vec<u8>>>> {
        self.map.clone()
    }

    /// Return the backing file path for this VFS instance.
    pub fn backing_path(&self) -> PathBuf {
        self.file_path.clone()
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

            // Deserialize the VFS data
            let persist: VfsPersist = bincode::deserialize(&plaintext)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            let mut map = HashMap::new();
            for (k, v) in persist.entries.into_iter() {
                map.insert(k, v);
            }
            Ok(Self {
                file_path: p,
                map: Arc::new(Mutex::new(map)),
            })
        } else {
            Ok(Self {
                file_path: p,
                map: Arc::new(Mutex::new(HashMap::new())),
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
        let mut entries = Vec::new();
        for (k, v) in map.iter() {
            entries.push((k.clone(), v.clone()));
        }
        let persist = VfsPersist { entries };
        let plaintext = bincode::serialize(&persist).map_err(io::Error::other)?;

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

#[cfg(not(feature = "real_fs_acknowledged"))]
pub mod fs {
    use super::*;

    type FileMap = Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>;

    #[allow(dead_code)]
    fn map_for_current() -> Option<FileMap> {
        if let Some(vfs) = get_current_vfs() {
            return Some(vfs.as_map());
        }
        Some(Arc::new(Mutex::new(IN_MEM_FILES.lock().unwrap().clone())))
    }

    pub fn create_dir_all<P: AsRef<Path>>(path: P) -> io::Result<()> {
        // Validate path to prevent traversal attacks
        let _p = validate_path(path.as_ref())?;
        // Directories are implicit in in-memory VFS.
        Ok(())
    }

    pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
        // Validate and normalize path to prevent traversal attacks
        let p = validate_path(path.as_ref())?;
        if let Some(vfs) = get_current_vfs() {
            let map_arc = vfs.as_map();
            let map = map_arc.lock().unwrap();
            match map.get(&p) {
                Some(bytes) => Ok(String::from_utf8_lossy(bytes).to_string()),
                None => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
            }
        } else {
            let map = IN_MEM_FILES.lock().unwrap();
            match map.get(&p) {
                Some(bytes) => Ok(String::from_utf8_lossy(bytes).to_string()),
                None => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
            }
        }
    }

    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
        // Validate and normalize path to prevent traversal attacks
        let p = validate_path(path.as_ref())?;
        if let Some(vfs) = get_current_vfs() {
            let map_arc = vfs.as_map();
            let mut map = map_arc.lock().unwrap();
            map.insert(p, contents.as_ref().to_vec());
            Ok(())
        } else {
            let mut map = IN_MEM_FILES.lock().unwrap();
            map.insert(p, contents.as_ref().to_vec());
            Ok(())
        }
    }

    pub fn read<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
        // Validate and normalize path to prevent traversal attacks
        let p = validate_path(path.as_ref())?;
        if let Some(vfs) = get_current_vfs() {
            let map_arc = vfs.as_map();
            let map = map_arc.lock().unwrap();
            match map.get(&p) {
                Some(bytes) => Ok(bytes.clone()),
                None => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
            }
        } else {
            let map = IN_MEM_FILES.lock().unwrap();
            match map.get(&p) {
                Some(bytes) => Ok(bytes.clone()),
                None => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
            }
        }
    }

    /// A minimal metadata representation for in-memory VFS
    #[derive(Debug, Clone)]
    pub struct Metadata {
        is_dir: bool,
        len: u64,
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
    }

    pub fn remove_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
        // Validate and normalize path to prevent traversal attacks
        let p = validate_path(path.as_ref())?;
        if let Some(vfs) = get_current_vfs() {
            let map_arc = vfs.as_map();
            let mut map = map_arc.lock().unwrap();
            map.remove(&p);
            Ok(())
        } else {
            let mut map = IN_MEM_FILES.lock().unwrap();
            map.remove(&p);
            Ok(())
        }
    }

    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
        // Validate and normalize paths to prevent traversal attacks
        let fromp = validate_path(from.as_ref())?;
        let top = validate_path(to.as_ref())?;
        if let Some(vfs) = get_current_vfs() {
            let map_arc = vfs.as_map();
            let mut map = map_arc.lock().unwrap();
            if let Some(bytes) = map.get(&fromp).cloned() {
                let len = bytes.len() as u64;
                map.insert(top, bytes);
                Ok(len)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "source file not found",
                ))
            }
        } else {
            let mut map = IN_MEM_FILES.lock().unwrap();
            if let Some(bytes) = map.get(&fromp).cloned() {
                let len = bytes.len() as u64;
                map.insert(top, bytes);
                Ok(len)
            } else {
                Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "source file not found",
                ))
            }
        }
    }

    pub fn metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata> {
        // Validate and normalize path to prevent traversal attacks
        let p = validate_path(path.as_ref())?;
        if let Some(vfs) = get_current_vfs() {
            let map_arc = vfs.as_map();
            let map = map_arc.lock().unwrap();
            if let Some(bytes) = map.get(&p) {
                Ok(Metadata {
                    is_dir: false,
                    len: bytes.len() as u64,
                })
            } else {
                // Check if any file is a descendant -> treat as dir
                // SECURITY: Use normalized path for comparison
                let is_dir = map.keys().any(|k| {
                    // Normalize stored paths for safe comparison
                    if let Some(normalized_k) = normalize_path(k) {
                        normalized_k.starts_with(&p)
                    } else {
                        false
                    }
                });
                if is_dir {
                    Ok(Metadata {
                        is_dir: true,
                        len: 0,
                    })
                } else {
                    Err(io::Error::new(io::ErrorKind::NotFound, "path not found"))
                }
            }
        } else {
            let map = IN_MEM_FILES.lock().unwrap();
            if let Some(bytes) = map.get(&p) {
                Ok(Metadata {
                    is_dir: false,
                    len: bytes.len() as u64,
                })
            } else {
                // SECURITY: Use normalized path for comparison
                let is_dir = map.keys().any(|k| {
                    if let Some(normalized_k) = normalize_path(k) {
                        normalized_k.starts_with(&p)
                    } else {
                        false
                    }
                });
                if is_dir {
                    Ok(Metadata {
                        is_dir: true,
                        len: 0,
                    })
                } else {
                    Err(io::Error::new(io::ErrorKind::NotFound, "path not found"))
                }
            }
        }
    }

    /// Directory entry for in-memory VFS
    #[derive(Debug, Clone)]
    pub struct DirEntry {
        path: PathBuf,
        is_dir: bool,
        len: u64,
    }

    impl DirEntry {
        pub fn path(&self) -> PathBuf {
            self.path.clone()
        }

        pub fn metadata(&self) -> io::Result<Metadata> {
            Ok(Metadata {
                is_dir: self.is_dir,
                len: self.len,
            })
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

    /// List immediate children (files and directories) of `path` within the in-memory VFS
    pub fn read_dir<P: AsRef<Path>>(path: P) -> io::Result<ReadDir> {
        // Validate and normalize path to prevent traversal attacks
        let root = validate_path(path.as_ref())?;
        use std::collections::HashMap as StdMap;
        let mut children: StdMap<PathBuf, (bool, u64)> = StdMap::new();

        if let Some(vfs) = get_current_vfs() {
            let map_arc = vfs.as_map();
            let map = map_arc.lock().unwrap();
            for (file_path, bytes) in map.iter() {
                // SECURITY: Normalize file paths before comparison
                let normalized_path = match normalize_path(file_path) {
                    Some(p) => p,
                    None => continue, // Skip malformed paths
                };
                if let Ok(relative) = normalized_path.strip_prefix(&root) {
                    let mut comps = relative.components();
                    if let Some(first) = comps.next() {
                        let child = root.join(first.as_os_str());
                        if comps.next().is_some() {
                            // there are further components -> it's a directory
                            children.entry(child).or_insert((true, 0));
                        } else {
                            // immediate file
                            children.entry(child).or_insert((false, bytes.len() as u64));
                        }
                    }
                }
            }
        } else {
            let map = IN_MEM_FILES.lock().unwrap();
            for (file_path, bytes) in map.iter() {
                // SECURITY: Normalize file paths before comparison
                let normalized_path = match normalize_path(file_path) {
                    Some(p) => p,
                    None => continue, // Skip malformed paths
                };
                if let Ok(relative) = normalized_path.strip_prefix(&root) {
                    let mut comps = relative.components();
                    if let Some(first) = comps.next() {
                        let child = root.join(first.as_os_str());
                        if comps.next().is_some() {
                            // there are further components -> it's a directory
                            children.entry(child).or_insert((true, 0));
                        } else {
                            // immediate file
                            children.entry(child).or_insert((false, bytes.len() as u64));
                        }
                    }
                }
            }
        }

        let mut entries = Vec::new();
        for (path, (is_dir, len)) in children.into_iter() {
            entries.push(DirEntry { path, is_dir, len });
        }

        Ok(ReadDir { entries, idx: 0 })
    }

    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        // Validate and normalize path; return false for malicious paths
        let p = match validate_path(path.as_ref()) {
            Ok(p) => p,
            Err(_) => return false,
        };
        if let Some(vfs) = get_current_vfs() {
            let map_arc = vfs.as_map();
            let map = map_arc.lock().unwrap();
            map.contains_key(&p)
        } else {
            let map = IN_MEM_FILES.lock().unwrap();
            map.contains_key(&p)
        }
    }
}

#[cfg(not(feature = "real_fs"))]
use std::io::{Read, Seek, SeekFrom, Write};

#[cfg(not(feature = "real_fs"))]
pub struct File {
    path: PathBuf,
    pos: usize,
}

#[cfg(not(feature = "real_fs"))]
impl File {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let p = path.as_ref().to_path_buf();
        let map = IN_MEM_FILES.lock().unwrap();
        if map.contains_key(&p) {
            Ok(Self { path: p, pos: 0 })
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "file not found"))
        }
    }

    pub fn create<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let p = path.as_ref().to_path_buf();
        let mut map = IN_MEM_FILES.lock().unwrap();
        map.insert(p.clone(), Vec::new());
        Ok(Self { path: p, pos: 0 })
    }
}

#[cfg(not(feature = "real_fs"))]
impl Read for File {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let map = IN_MEM_FILES.lock().unwrap();
        if let Some(bytes) = map.get(&self.path) {
            let remaining = &bytes[self.pos..];
            let n = std::cmp::min(remaining.len(), buf.len());
            buf[..n].copy_from_slice(&remaining[..n]);
            self.pos += n;
            Ok(n)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "file not found"))
        }
    }
}

#[cfg(not(feature = "real_fs"))]
impl Seek for File {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let map = IN_MEM_FILES.lock().unwrap();
        if let Some(bytes) = map.get(&self.path) {
            // SECURITY: Use checked arithmetic to prevent integer overflow (CWE-190)
            let new = match pos {
                SeekFrom::Start(off) => i64::try_from(off).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidInput, "seek offset too large")
                })?,
                SeekFrom::End(off) => {
                    let len = i64::try_from(bytes.len()).map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidInput, "file too large")
                    })?;
                    len.checked_add(off).ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "seek overflow")
                    })?
                }
                SeekFrom::Current(off) => {
                    let pos = i64::try_from(self.pos).map_err(|_| {
                        io::Error::new(io::ErrorKind::InvalidInput, "position too large")
                    })?;
                    pos.checked_add(off).ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "seek overflow")
                    })?
                }
            };
            if new < 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid seek"));
            }
            // SECURITY: Safe conversion since we've verified new >= 0
            self.pos = usize::try_from(new).map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidInput, "seek result too large")
            })?;
            Ok(self.pos as u64)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "file not found"))
        }
    }
}

#[cfg(not(feature = "real_fs"))]
impl Write for File {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut map = IN_MEM_FILES.lock().unwrap();
        let entry = map
            .get_mut(&self.path)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))?;
        if self.pos >= entry.len() {
            entry.extend_from_slice(buf);
            self.pos = entry.len();
            Ok(buf.len())
        } else {
            let end = self.pos + buf.len();
            if end > entry.len() {
                entry.resize(end, 0);
            }
            entry[self.pos..end].copy_from_slice(buf);
            self.pos = end;
            Ok(buf.len())
        }
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

    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        let p = path.as_ref().to_path_buf();
        let mut map = IN_MEM_FILES.lock().unwrap();
        if self.create && !map.contains_key(&p) {
            map.insert(p.clone(), Vec::new());
        }
        if self.truncate
            && let Some(entry) = map.get_mut(&p)
        {
            entry.clear();
        }
        if map.contains_key(&p) {
            Ok(File { path: p, pos: 0 })
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "file not found"))
        }
    }
}

#[cfg(test)]
mod tests;
