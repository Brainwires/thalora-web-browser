use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
#[allow(dead_code)]
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::io;
use uuid::Uuid;
use std::fs as stdfs;
use bincode;

// Encryption imports for session data at rest
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    ChaCha20Poly1305, Nonce,
};
use chacha20poly1305::aead::rand_core::RngCore;
use zeroize::Zeroizing;
use sha2::{Sha256, Digest};

/// Derive a 256-bit encryption key from a session ID and optional secret.
///
/// # Security
/// Uses SHA-256 to derive a key from:
/// - THALORA_SESSION_SECRET environment variable (if set)
/// - A hardcoded fallback secret (for development/testing only)
/// - The session_id
///
/// In production, THALORA_SESSION_SECRET should be set to a cryptographically
/// random 32+ byte value.
pub fn derive_session_key(session_id: &str) -> Zeroizing<[u8; 32]> {
    // Get secret from environment or use fallback
    let secret = std::env::var("THALORA_SESSION_SECRET")
        .unwrap_or_else(|_| "thalora-dev-session-secret-do-not-use-in-production".to_string());

    // Derive key using SHA-256(secret || session_id)
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(session_id.as_bytes());
    let result = hasher.finalize();

    let mut key = Zeroizing::new([0u8; 32]);
    key.copy_from_slice(&result);
    key
}

#[cfg(not(feature = "real_fs"))]
static IN_MEM_FILES: Lazy<Mutex<HashMap<PathBuf, Vec<u8>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

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
            let persist: VfsPersist = bincode::deserialize(&bytes).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
            let mut map = HashMap::new();
            for (k, v) in persist.entries.into_iter() {
                map.insert(k, v);
            }
            Ok(Self { file_path: p, map: Arc::new(Mutex::new(map)) })
        } else {
            Ok(Self { file_path: p, map: Arc::new(Mutex::new(HashMap::new())) })
        }
    }

    /// Create a new temporary file-backed VFS with a unique filename in `dir`.
    pub fn new_temp_in_dir<P: AsRef<Path>>(dir: P) -> io::Result<Self> {
        let id = Uuid::new_v4().to_string();
        let file = dir.as_ref().join(format!("vfs-{}.bin", id));
        Ok(Self { file_path: file, map: Arc::new(Mutex::new(HashMap::new())) })
    }

    /// Persist current in-memory map to disk atomically.
    pub fn persist(&self) -> io::Result<()> {
        let map = self.map.lock().unwrap();
        let mut entries = Vec::new();
        for (k, v) in map.iter() {
            entries.push((k.clone(), v.clone()));
        }
        let persist = VfsPersist { entries };
        let bytes = bincode::serialize(&persist).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
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
                return Err(io::Error::new(io::ErrorKind::InvalidData, "encrypted file too short"));
            }

            let (nonce_bytes, ciphertext) = encrypted_bytes.split_at(12);
            let nonce = Nonce::from_slice(nonce_bytes);

            // Create cipher with zeroizing key wrapper
            let key_array = Zeroizing::new(*key);
            let cipher = ChaCha20Poly1305::new_from_slice(&*key_array)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

            // Decrypt the data
            let plaintext = cipher.decrypt(nonce, ciphertext)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData,
                    "decryption failed: invalid key or corrupted data"))?;

            // Deserialize the VFS data
            let persist: VfsPersist = bincode::deserialize(&plaintext)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

            let mut map = HashMap::new();
            for (k, v) in persist.entries.into_iter() {
                map.insert(k, v);
            }
            Ok(Self { file_path: p, map: Arc::new(Mutex::new(map)) })
        } else {
            Ok(Self { file_path: p, map: Arc::new(Mutex::new(HashMap::new())) })
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
        let plaintext = bincode::serialize(&persist)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        // Generate a random 96-bit nonce
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Create cipher with zeroizing key wrapper
        let key_array = Zeroizing::new(*key);
        let cipher = ChaCha20Poly1305::new_from_slice(&*key_array)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e.to_string()))?;

        // Encrypt the data
        let ciphertext = cipher.encrypt(nonce, plaintext.as_slice())
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;

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

#[cfg(feature = "real_fs")]
pub mod fs {
    pub use std::fs::*;
    use std::path::Path;

    /// Check if a path exists (wrapper for Path::exists for API compatibility)
    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().exists()
    }
}

#[cfg(not(feature = "real_fs"))]
pub mod fs {
    use super::*;

    #[allow(dead_code)]
    fn map_for_current() -> Option<Arc<Mutex<HashMap<PathBuf, Vec<u8>>>>> {
        if let Some(vfs) = get_current_vfs() {
            return Some(vfs.as_map());
        }
        Some(Arc::new(Mutex::new(IN_MEM_FILES.lock().unwrap().clone())))
    }

    pub fn create_dir_all<P: AsRef<Path>>(_path: P) -> io::Result<()> {
        // Directories are implicit in in-memory VFS.
        Ok(())
    }

    pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
        let p = path.as_ref().to_path_buf();
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
        let p = path.as_ref().to_path_buf();
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
        let p = path.as_ref().to_path_buf();
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

        pub fn is_dir(&self) -> bool {
            self.is_dir
        }
    }

    pub fn remove_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let p = path.as_ref().to_path_buf();
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
        let fromp = from.as_ref().to_path_buf();
        let top = to.as_ref().to_path_buf();
        if let Some(vfs) = get_current_vfs() {
            let map_arc = vfs.as_map();
            let mut map = map_arc.lock().unwrap();
            if let Some(bytes) = map.get(&fromp).cloned() {
                let len = bytes.len() as u64;
                map.insert(top, bytes);
                Ok(len)
            } else {
                Err(io::Error::new(io::ErrorKind::NotFound, "source file not found"))
            }
        } else {
            let mut map = IN_MEM_FILES.lock().unwrap();
            if let Some(bytes) = map.get(&fromp).cloned() {
                let len = bytes.len() as u64;
                map.insert(top, bytes);
                Ok(len)
            } else {
                Err(io::Error::new(io::ErrorKind::NotFound, "source file not found"))
            }
        }
    }

    pub fn metadata<P: AsRef<Path>>(path: P) -> io::Result<Metadata> {
        let p = path.as_ref().to_path_buf();
        if let Some(vfs) = get_current_vfs() {
            let map_arc = vfs.as_map();
            let map = map_arc.lock().unwrap();
            if let Some(bytes) = map.get(&p) {
                Ok(Metadata {
                    is_dir: false,
                    len: bytes.len() as u64,
                })
            } else {
                // check if any file is a descendant -> treat as dir
                let is_dir = map.keys().any(|k| k.starts_with(&p));
                if is_dir {
                    Ok(Metadata { is_dir: true, len: 0 })
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
                let is_dir = map.keys().any(|k| k.starts_with(&p));
                if is_dir {
                    Ok(Metadata { is_dir: true, len: 0 })
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
        let root = path.as_ref().to_path_buf();
        use std::collections::HashMap as StdMap;
        let mut children: StdMap<PathBuf, (bool, u64)> = StdMap::new();

        if let Some(vfs) = get_current_vfs() {
            let map_arc = vfs.as_map();
            let map = map_arc.lock().unwrap();
            for (file_path, bytes) in map.iter() {
                if let Ok(relative) = file_path.strip_prefix(&root) {
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
                if let Ok(relative) = file_path.strip_prefix(&root) {
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
        let p = path.as_ref().to_path_buf();
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
                SeekFrom::Start(off) => i64::try_from(off)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "seek offset too large"))?,
                SeekFrom::End(off) => {
                    let len = i64::try_from(bytes.len())
                        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "file too large"))?;
                    len.checked_add(off)
                        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek overflow"))?
                },
                SeekFrom::Current(off) => {
                    let pos = i64::try_from(self.pos)
                        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "position too large"))?;
                    pos.checked_add(off)
                        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "seek overflow"))?
                },
            };
            if new < 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid seek"));
            }
            // SECURITY: Safe conversion since we've verified new >= 0
            self.pos = usize::try_from(new)
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "seek result too large"))?;
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
        let entry = map.get_mut(&self.path).ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "file not found"))?;
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

    fn flush(&mut self) -> io::Result<()> { Ok(()) }
}

#[cfg(not(feature = "real_fs"))]
pub struct OpenOptions {
    read: bool,
    write: bool,
    create: bool,
    truncate: bool,
}

#[cfg(not(feature = "real_fs"))]
impl OpenOptions {
    pub fn new() -> Self {
        Self { read: false, write: false, create: false, truncate: false }
    }

    pub fn read(&mut self, v: bool) -> &mut Self { self.read = v; self }
    pub fn write(&mut self, v: bool) -> &mut Self { self.write = v; self }
    pub fn create(&mut self, v: bool) -> &mut Self { self.create = v; self }
    pub fn truncate(&mut self, v: bool) -> &mut Self { self.truncate = v; self }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        let p = path.as_ref().to_path_buf();
        let mut map = IN_MEM_FILES.lock().unwrap();
        if self.create && !map.contains_key(&p) {
            map.insert(p.clone(), Vec::new());
        }
        if self.truncate {
            if let Some(entry) = map.get_mut(&p) { entry.clear(); }
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
