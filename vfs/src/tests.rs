use super::fs;
use std::path::PathBuf;
use super::{VfsInstance, set_current_vfs};
use std::sync::Arc;
use std::fs as stdfs;

#[test]
fn basic_write_read() {
    let p = PathBuf::from("/tmp/testfile");
    fs::write(&p, b"hello").unwrap();
    let s = fs::read_to_string(&p).unwrap();
    assert_eq!(s, "hello");
}

#[test]
fn file_backed_persist_cycle() {
    let tmp = std::env::temp_dir();
    let vfs = VfsInstance::new_temp_in_dir(&tmp).expect("create vfs");
    let backing = vfs.file_path.clone();
    let arc = Arc::new(vfs);
    let _prev = set_current_vfs(Some(arc.clone()));

    // write a file via fs API (delegates to current vfs)
    let p = PathBuf::from("/session/data.txt");
    fs::write(&p, b"payload").expect("write");
    let s = fs::read_to_string(&p).expect("read");
    assert_eq!(s, "payload");

    // persist backing
    arc.persist().expect("persist");
    assert!(backing.exists());

    // backing file should contain serialized data; read and ensure it's non-empty
    let bytes = stdfs::read(&backing).expect("read backing");
    assert!(!bytes.is_empty());

    // cleanup
    arc.delete_backing_file().expect("delete");
    assert!(!backing.exists());
    drop(set_current_vfs(None));
}
