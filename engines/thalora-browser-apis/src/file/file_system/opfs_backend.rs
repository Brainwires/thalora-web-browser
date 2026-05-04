//! Per-origin disk-backed Origin Private File System backend.
//!
//! Each origin gets an isolated directory under `<data_local_dir>/thalora/opfs/<slug>`.
//! All paths inside the OPFS tree are *virtual* (rooted at `/`) and resolved through
//! [`OpfsBackend::resolve`], which canonicalises and checks containment to prevent
//! traversal escapes.
//!
//! The backend goes direct to `std::fs` rather than the workspace `vfs` crate, which
//! is security-gated and not intended for real-disk per-origin isolation.

use once_cell::sync::Lazy;
use parking_lot::Mutex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

/// Snapshot of one entry returned by [`OpfsBackend::read_dir`].
#[derive(Debug, Clone)]
pub struct DirEntrySnapshot {
    pub name: String,
    pub is_dir: bool,
}

/// Result-style errors surfaced from path resolution. Disk I/O failures are
/// returned as `io::Error`; this enum is for path-level violations that the
/// caller maps to specific DOMException names.
#[derive(Debug)]
pub enum FsError {
    /// Path component contains `/`, `\`, `.`, `..`, or is empty.
    InvalidName,
    /// Resolved path escaped the OPFS root.
    PathEscape,
}

/// Per-origin OPFS root.
#[derive(Debug)]
pub struct OpfsBackend {
    root: PathBuf,
}

static OPFS_ROOTS: Lazy<Mutex<HashMap<String, Arc<OpfsBackend>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

impl OpfsBackend {
    /// Get (or create) the backend for an origin. Cached process-wide.
    pub fn for_origin(origin: &str) -> Arc<OpfsBackend> {
        let mut roots = OPFS_ROOTS.lock();
        if let Some(existing) = roots.get(origin) {
            return existing.clone();
        }
        let root = Self::compute_origin_root(origin);
        let _ = fs::create_dir_all(&root);
        let backend = Arc::new(OpfsBackend { root });
        roots.insert(origin.to_string(), backend.clone());
        backend
    }

    fn compute_origin_root(origin: &str) -> PathBuf {
        let base = dirs::data_local_dir()
            .unwrap_or_else(std::env::temp_dir)
            .join("thalora")
            .join("opfs");
        let slug = origin_slug(origin);
        base.join(slug)
    }

    pub fn root_path(&self) -> &Path {
        &self.root
    }

    /// Resolve a virtual path (rooted at `/`) to a real path inside the OPFS root.
    /// Rejects empty components, `.`, `..`, drive-letter prefixes, etc.
    pub fn resolve(&self, virt: &Path) -> Result<PathBuf, FsError> {
        let mut out = self.root.clone();
        for comp in virt.components() {
            match comp {
                Component::RootDir | Component::CurDir => continue,
                Component::Normal(seg) => {
                    let s = seg.to_str().ok_or(FsError::InvalidName)?;
                    if s.is_empty() || s == "." || s == ".." || s.contains('/') || s.contains('\\')
                    {
                        return Err(FsError::InvalidName);
                    }
                    out.push(seg);
                }
                Component::ParentDir | Component::Prefix(_) => return Err(FsError::PathEscape),
            }
        }
        // Belt-and-braces: refuse anything that doesn't start_with the root after pushing.
        if !out.starts_with(&self.root) {
            return Err(FsError::PathEscape);
        }
        Ok(out)
    }

    pub fn read_bytes(&self, virt: &Path) -> io::Result<Vec<u8>> {
        let path = self.resolve(virt).map_err(invalid_name_to_io)?;
        fs::read(path)
    }

    pub fn write_bytes(&self, virt: &Path, data: &[u8]) -> io::Result<()> {
        let path = self.resolve(virt).map_err(invalid_name_to_io)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, data)
    }

    pub fn copy_file(&self, src_virt: &Path, dst_virt: &Path) -> io::Result<()> {
        let src = self.resolve(src_virt).map_err(invalid_name_to_io)?;
        let dst = self.resolve(dst_virt).map_err(invalid_name_to_io)?;
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(src, dst).map(|_| ())
    }

    pub fn rename(&self, src_virt: &Path, dst_virt: &Path) -> io::Result<()> {
        let src = self.resolve(src_virt).map_err(invalid_name_to_io)?;
        let dst = self.resolve(dst_virt).map_err(invalid_name_to_io)?;
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::rename(src, dst)
    }

    pub fn open_file_rw(&self, virt: &Path, create: bool) -> io::Result<File> {
        let path = self.resolve(virt).map_err(invalid_name_to_io)?;
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(create)
            .open(path)
    }

    /// Create a single directory (fails if it already exists or parent missing).
    pub fn mkdir(&self, virt: &Path) -> io::Result<()> {
        let path = self.resolve(virt).map_err(invalid_name_to_io)?;
        fs::create_dir(path)
    }

    pub fn mkdir_p(&self, virt: &Path) -> io::Result<()> {
        let path = self.resolve(virt).map_err(invalid_name_to_io)?;
        fs::create_dir_all(path)
    }

    pub fn remove_file(&self, virt: &Path) -> io::Result<()> {
        let path = self.resolve(virt).map_err(invalid_name_to_io)?;
        fs::remove_file(path)
    }

    pub fn remove_dir(&self, virt: &Path) -> io::Result<()> {
        let path = self.resolve(virt).map_err(invalid_name_to_io)?;
        fs::remove_dir(path)
    }

    pub fn remove_dir_recursive(&self, virt: &Path) -> io::Result<()> {
        let path = self.resolve(virt).map_err(invalid_name_to_io)?;
        fs::remove_dir_all(path)
    }

    pub fn read_dir(&self, virt: &Path) -> io::Result<Vec<DirEntrySnapshot>> {
        let path = self.resolve(virt).map_err(invalid_name_to_io)?;
        let mut out = Vec::new();
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let name = match entry.file_name().into_string() {
                Ok(s) => s,
                Err(_) => continue,
            };
            let ft = entry.file_type()?;
            out.push(DirEntrySnapshot {
                name,
                is_dir: ft.is_dir(),
            });
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }

    pub fn metadata(&self, virt: &Path) -> io::Result<fs::Metadata> {
        let path = self.resolve(virt).map_err(invalid_name_to_io)?;
        fs::metadata(path)
    }

    pub fn exists(&self, virt: &Path) -> bool {
        self.resolve(virt)
            .map(|p| p.exists())
            .unwrap_or(false)
    }

    pub fn is_dir(&self, virt: &Path) -> bool {
        self.resolve(virt)
            .map(|p| p.is_dir())
            .unwrap_or(false)
    }

    pub fn is_file(&self, virt: &Path) -> bool {
        self.resolve(virt)
            .map(|p| p.is_file())
            .unwrap_or(false)
    }

    pub fn is_dir_empty(&self, virt: &Path) -> io::Result<bool> {
        let path = self.resolve(virt).map_err(invalid_name_to_io)?;
        Ok(fs::read_dir(path)?.next().is_none())
    }
}

fn invalid_name_to_io(e: FsError) -> io::Error {
    match e {
        FsError::InvalidName => {
            io::Error::new(io::ErrorKind::InvalidInput, "invalid OPFS name")
        }
        FsError::PathEscape => {
            io::Error::new(io::ErrorKind::PermissionDenied, "OPFS path escape")
        }
    }
}

/// Convert an origin into a filesystem-safe slug.
///
/// `lowercase(origin)` with `[^a-z0-9._-]` → `_`, then append `_<8 hex chars>`
/// of the SHA-256 of the original origin. The hash suffix prevents collisions
/// between origins that differ only in characters lost to the sanitisation.
fn origin_slug(origin: &str) -> String {
    let lower = origin.to_lowercase();
    let mut sanitised = String::with_capacity(lower.len());
    for ch in lower.chars() {
        if ch.is_ascii_alphanumeric() || ch == '.' || ch == '_' || ch == '-' {
            sanitised.push(ch);
        } else {
            sanitised.push('_');
        }
    }
    let hash = Sha256::digest(origin.as_bytes());
    let hex: String = hash
        .iter()
        .take(4)
        .map(|b| format!("{:02x}", b))
        .collect();
    if sanitised.is_empty() {
        format!("origin_{}", hex)
    } else {
        format!("{}_{}", sanitised, hex)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_distinguishes_similar_origins() {
        assert_ne!(
            origin_slug("https://example.com"),
            origin_slug("https://example.com:8080")
        );
        assert_ne!(
            origin_slug("https://example.com/"),
            origin_slug("https://example.com")
        );
    }

    #[test]
    fn resolve_rejects_traversal() {
        let backend = OpfsBackend {
            root: std::env::temp_dir().join("opfs_test_root"),
        };
        assert!(backend.resolve(Path::new("../etc/passwd")).is_err());
        assert!(backend.resolve(Path::new("/foo/../../etc")).is_err());
        assert!(backend.resolve(Path::new("/")).is_ok());
        assert!(backend.resolve(Path::new("/a/b/c")).is_ok());
    }

    #[test]
    fn for_origin_caches() {
        let a = OpfsBackend::for_origin("https://test-cache.example");
        let b = OpfsBackend::for_origin("https://test-cache.example");
        assert!(Arc::ptr_eq(&a, &b));
    }
}
