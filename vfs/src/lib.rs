use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::io;

#[cfg(feature = "real_fs")]
pub mod fs {
    pub use std::fs::*;
}

#[cfg(not(feature = "real_fs"))]
pub mod fs {
    use super::*;

    static IN_MEM_FILES: Lazy<Mutex<HashMap<PathBuf, Vec<u8>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

    pub fn create_dir_all<P: AsRef<Path>>(_path: P) -> io::Result<()> {
        // Directories are implicit in in-memory VFS.
        Ok(())
    }

    pub fn read_to_string<P: AsRef<Path>>(path: P) -> io::Result<String> {
        let map = IN_MEM_FILES.lock().unwrap();
        let p = path.as_ref().to_path_buf();
        match map.get(&p) {
            Some(bytes) => Ok(String::from_utf8_lossy(bytes).to_string()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
        }
    }

    pub fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> io::Result<()> {
        let mut map = IN_MEM_FILES.lock().unwrap();
        map.insert(path.as_ref().to_path_buf(), contents.as_ref().to_vec());
        Ok(())
    }

    pub fn read<P: AsRef<Path>>(path: P) -> io::Result<Vec<u8>> {
        let map = IN_MEM_FILES.lock().unwrap();
        let p = path.as_ref().to_path_buf();
        match map.get(&p) {
            Some(bytes) => Ok(bytes.clone()),
            None => Err(io::Error::new(io::ErrorKind::NotFound, "file not found")),
        }
    }

    pub fn remove_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
        let mut map = IN_MEM_FILES.lock().unwrap();
        map.remove(&path.as_ref().to_path_buf());
        Ok(())
    }

    pub fn copy<P: AsRef<Path>, Q: AsRef<Path>>(from: P, to: Q) -> io::Result<u64> {
        let mut map = IN_MEM_FILES.lock().unwrap();
        let fromp = from.as_ref().to_path_buf();
        if let Some(bytes) = map.get(&fromp) {
            map.insert(to.as_ref().to_path_buf(), bytes.clone());
            Ok(bytes.len() as u64)
        } else {
            Err(io::Error::new(io::ErrorKind::NotFound, "source file not found"))
        }
    }

    pub fn metadata<P: AsRef<Path>>(_path: P) -> io::Result<std::fs::Metadata> {
        // Minimal stub: returning error to indicate metadata unavailable
        Err(io::Error::new(io::ErrorKind::Other, "metadata not supported in in-memory VFS"))
    }

    pub fn exists<P: AsRef<Path>>(path: P) -> bool {
        let map = IN_MEM_FILES.lock().unwrap();
        map.contains_key(&path.as_ref().to_path_buf())
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
            let new = match pos {
                SeekFrom::Start(off) => off as i64,
                SeekFrom::End(off) => bytes.len() as i64 + off,
                SeekFrom::Current(off) => self.pos as i64 + off,
            };
            if new < 0 {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid seek"));
            }
            self.pos = new as usize;
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
mod tests {
    use super::fs;
    use std::path::PathBuf;

    #[test]
    fn basic_write_read() {
        let p = PathBuf::from("/tmp/testfile");
        fs::write(&p, b"hello").unwrap();
        let s = fs::read_to_string(&p).unwrap();
        assert_eq!(s, "hello");
    }
}
