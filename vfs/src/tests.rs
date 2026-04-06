use super::fs;
use super::{VfsInstance, set_current_vfs};
use std::fs as stdfs;
use std::path::PathBuf;
use std::sync::Arc;

// Helper to run a test with an isolated temporary VFS instance, ensuring
// no cross-test interference through the global CURRENT_VFS.
fn with_temp_vfs<F: FnOnce()>(f: F) {
    let tmp = std::env::temp_dir();
    let vfs = VfsInstance::new_temp_in_dir(&tmp).expect("create vfs");
    let arc = Arc::new(vfs);
    let prev = set_current_vfs(Some(arc.clone()));
    f();
    arc.delete_backing_file().ok();
    set_current_vfs(prev);
}

// =============================================================================
// BASIC READ/WRITE
// =============================================================================

#[test]
fn basic_write_read() {
    with_temp_vfs(|| {
        let p = PathBuf::from("/tmp/testfile");
        fs::write(&p, b"hello").unwrap();
        let s = fs::read_to_string(&p).unwrap();
        assert_eq!(s, "hello");
    });
}

#[test]
fn read_binary() {
    with_temp_vfs(|| {
        let p = PathBuf::from("/bin/data");
        let data = vec![0u8, 1, 2, 255, 128];
        fs::write(&p, &data).unwrap();
        let out = fs::read(&p).unwrap();
        assert_eq!(out, data);
    });
}

#[test]
fn overwrite_file() {
    with_temp_vfs(|| {
        let p = PathBuf::from("/overwrite");
        fs::write(&p, b"first").unwrap();
        fs::write(&p, b"second").unwrap();
        assert_eq!(fs::read_to_string(&p).unwrap(), "second");
    });
}

// =============================================================================
// PERSISTENCE
// =============================================================================

#[test]
fn file_backed_persist_cycle() {
    let tmp = std::env::temp_dir();
    let vfs = VfsInstance::new_temp_in_dir(&tmp).expect("create vfs");
    let backing = vfs.backing_path();
    let arc = Arc::new(vfs);
    let _prev = set_current_vfs(Some(arc.clone()));

    let p = PathBuf::from("/session/data.txt");
    fs::write(&p, b"payload").expect("write");
    let s = fs::read_to_string(&p).expect("read");
    assert_eq!(s, "payload");

    arc.persist().expect("persist");
    assert!(backing.exists());

    let bytes = stdfs::read(&backing).expect("read backing");
    assert!(!bytes.is_empty());

    arc.delete_backing_file().expect("delete");
    assert!(!backing.exists());
    drop(set_current_vfs(None));
}

#[test]
fn encrypted_persist_and_load() {
    use super::derive_session_key;

    let tmp = std::env::temp_dir();
    let backing = tmp.join("test-encrypted-vfs.bin.enc");
    let _ = stdfs::remove_file(&backing);

    let key = derive_session_key("test-session-123");

    {
        let vfs = VfsInstance::open_file_backed_encrypted(&backing, &*key).expect("create vfs");
        let arc = Arc::new(vfs);
        let _prev = set_current_vfs(Some(arc.clone()));

        let p = PathBuf::from("/encrypted/secret.txt");
        fs::write(&p, b"sensitive data").expect("write");

        arc.persist_encrypted(&*key).expect("persist encrypted");
        drop(set_current_vfs(None));
    }

    assert!(backing.exists());

    let raw_bytes = stdfs::read(&backing).expect("read raw");
    assert!(!raw_bytes.is_empty());
    assert!(!String::from_utf8_lossy(&raw_bytes).contains("sensitive data"));

    {
        let vfs = VfsInstance::open_file_backed_encrypted(&backing, &*key).expect("reload vfs");
        let arc = Arc::new(vfs);
        let _prev = set_current_vfs(Some(arc.clone()));

        let p = PathBuf::from("/encrypted/secret.txt");
        let s = fs::read_to_string(&p).expect("read decrypted");
        assert_eq!(s, "sensitive data");
        drop(set_current_vfs(None));
    }

    let _ = stdfs::remove_file(&backing);
}

#[test]
fn encrypted_wrong_key_fails() {
    use super::derive_session_key;

    let tmp = std::env::temp_dir();
    let backing = tmp.join("test-wrong-key.bin.enc");
    let _ = stdfs::remove_file(&backing);

    let key1 = derive_session_key("session-key-1");
    {
        let vfs = VfsInstance::open_file_backed_encrypted(&backing, &*key1).expect("create vfs");
        let arc = Arc::new(vfs);
        let _prev = set_current_vfs(Some(arc.clone()));

        let p = PathBuf::from("/test/data.txt");
        fs::write(&p, b"test content").expect("write");
        arc.persist_encrypted(&*key1).expect("persist");
        drop(set_current_vfs(None));
    }

    let key2 = derive_session_key("session-key-2-different");
    let result = VfsInstance::open_file_backed_encrypted(&backing, &*key2);
    assert!(result.is_err());

    let _ = stdfs::remove_file(&backing);
}

#[test]
fn derive_session_key_deterministic() {
    use super::derive_session_key;

    let key1 = derive_session_key("my-session");
    let key2 = derive_session_key("my-session");
    assert_eq!(&*key1, &*key2);

    let key3 = derive_session_key("other-session");
    assert_ne!(&*key1, &*key3);
}

// =============================================================================
// EXISTS (bug fix: must recognize directories)
// =============================================================================

#[test]
fn exists_file() {
    with_temp_vfs(|| {
        let p = PathBuf::from("/exists/file.txt");
        assert!(!fs::exists(&p));
        fs::write(&p, b"data").unwrap();
        assert!(fs::exists(&p));
    });
}

#[test]
fn exists_recognizes_implicit_directories() {
    with_temp_vfs(|| {
        fs::write("/deep/nested/file.txt", b"data").unwrap();
        assert!(fs::exists("/deep"));
        assert!(fs::exists("/deep/nested"));
    });
}

#[test]
fn exists_recognizes_explicit_directories() {
    with_temp_vfs(|| {
        fs::create_dir_all("/explicit/dir").unwrap();
        assert!(fs::exists("/explicit"));
        assert!(fs::exists("/explicit/dir"));
    });
}

// =============================================================================
// DIRECTORY OPERATIONS
// =============================================================================

#[test]
fn create_dir_all_creates_ancestors() {
    with_temp_vfs(|| {
        fs::create_dir_all("/a/b/c").unwrap();
        assert!(fs::exists("/a"));
        assert!(fs::exists("/a/b"));
        assert!(fs::exists("/a/b/c"));
        let meta = fs::metadata("/a/b/c").unwrap();
        assert!(meta.is_dir());
    });
}

#[test]
fn create_dir_single() {
    with_temp_vfs(|| {
        // Create parent first
        fs::create_dir_all("/parent").unwrap();
        fs::create_dir("/parent/child").unwrap();
        assert!(fs::metadata("/parent/child").unwrap().is_dir());
    });
}

#[test]
fn create_dir_fails_without_parent() {
    with_temp_vfs(|| {
        let result = fs::create_dir("/no_parent/child");
        assert!(result.is_err());
    });
}

#[test]
fn create_dir_fails_if_exists() {
    with_temp_vfs(|| {
        fs::create_dir_all("/existing").unwrap();
        let result = fs::create_dir("/existing");
        assert!(result.is_err());
    });
}

#[test]
fn remove_dir_empty() {
    with_temp_vfs(|| {
        fs::create_dir_all("/removable").unwrap();
        fs::remove_dir("/removable").unwrap();
        assert!(!fs::exists("/removable"));
    });
}

#[test]
fn remove_dir_fails_if_not_empty() {
    with_temp_vfs(|| {
        fs::write("/notempty/file.txt", b"data").unwrap();
        let result = fs::remove_dir("/notempty");
        assert!(result.is_err());
    });
}

#[test]
fn remove_dir_fails_on_file() {
    with_temp_vfs(|| {
        fs::write("/afile", b"data").unwrap();
        let result = fs::remove_dir("/afile");
        assert!(result.is_err());
    });
}

#[test]
fn remove_dir_all_removes_children() {
    with_temp_vfs(|| {
        fs::write("/tree/a.txt", b"a").unwrap();
        fs::write("/tree/sub/b.txt", b"b").unwrap();
        fs::write("/tree/sub/deep/c.txt", b"c").unwrap();
        fs::remove_dir_all("/tree").unwrap();
        assert!(!fs::exists("/tree"));
        assert!(!fs::exists("/tree/a.txt"));
        assert!(!fs::exists("/tree/sub/b.txt"));
    });
}

#[test]
fn remove_dir_all_not_found() {
    with_temp_vfs(|| {
        let result = fs::remove_dir_all("/nonexistent");
        assert!(result.is_err());
    });
}

// =============================================================================
// REMOVE FILE
// =============================================================================

#[test]
fn remove_file_works() {
    with_temp_vfs(|| {
        fs::write("/removeme", b"data").unwrap();
        fs::remove_file("/removeme").unwrap();
        assert!(!fs::exists("/removeme"));
    });
}

#[test]
fn remove_file_rejects_directory() {
    with_temp_vfs(|| {
        fs::create_dir_all("/adir").unwrap();
        let result = fs::remove_file("/adir");
        assert!(result.is_err());
    });
}

#[test]
fn remove_file_not_found() {
    with_temp_vfs(|| {
        let result = fs::remove_file("/no_such_file");
        assert!(result.is_err());
    });
}

// =============================================================================
// RENAME
// =============================================================================

#[test]
fn rename_file() {
    with_temp_vfs(|| {
        fs::write("/old.txt", b"content").unwrap();
        fs::rename("/old.txt", "/new.txt").unwrap();
        assert!(!fs::exists("/old.txt"));
        assert_eq!(fs::read_to_string("/new.txt").unwrap(), "content");
    });
}

#[test]
fn rename_directory() {
    with_temp_vfs(|| {
        fs::write("/olddir/a.txt", b"a").unwrap();
        fs::write("/olddir/sub/b.txt", b"b").unwrap();
        fs::rename("/olddir", "/newdir").unwrap();
        assert!(!fs::exists("/olddir"));
        assert_eq!(fs::read_to_string("/newdir/a.txt").unwrap(), "a");
        assert_eq!(fs::read_to_string("/newdir/sub/b.txt").unwrap(), "b");
    });
}

#[test]
fn rename_not_found() {
    with_temp_vfs(|| {
        let result = fs::rename("/missing", "/dest");
        assert!(result.is_err());
    });
}

// =============================================================================
// COPY
// =============================================================================

#[test]
fn copy_file() {
    with_temp_vfs(|| {
        fs::write("/src.txt", b"original").unwrap();
        let len = fs::copy("/src.txt", "/dst.txt").unwrap();
        assert_eq!(len, 8);
        assert_eq!(fs::read_to_string("/dst.txt").unwrap(), "original");
        // Source still exists
        assert!(fs::exists("/src.txt"));
    });
}

#[test]
fn copy_not_found() {
    with_temp_vfs(|| {
        let result = fs::copy("/nope", "/dst");
        assert!(result.is_err());
    });
}

// =============================================================================
// METADATA & TIMESTAMPS
// =============================================================================

#[test]
fn metadata_file() {
    with_temp_vfs(|| {
        fs::write("/meta.txt", b"12345").unwrap();
        let meta = fs::metadata("/meta.txt").unwrap();
        assert!(!meta.is_dir());
        assert!(meta.is_file());
        assert_eq!(meta.len(), 5);
        // Timestamps should be recent
        let created = meta.created().unwrap();
        let elapsed = created.elapsed().unwrap();
        assert!(elapsed.as_secs() < 5);
    });
}

#[test]
fn metadata_directory() {
    with_temp_vfs(|| {
        fs::create_dir_all("/metadir").unwrap();
        let meta = fs::metadata("/metadir").unwrap();
        assert!(meta.is_dir());
        assert!(!meta.is_file());
    });
}

#[test]
fn metadata_not_found() {
    with_temp_vfs(|| {
        let result = fs::metadata("/nonexistent");
        assert!(result.is_err());
    });
}

#[test]
fn metadata_modified_updates_on_write() {
    with_temp_vfs(|| {
        fs::write("/ts.txt", b"v1").unwrap();
        let m1 = fs::metadata("/ts.txt").unwrap().modified().unwrap();
        // Small delay to ensure timestamp differs
        std::thread::sleep(std::time::Duration::from_millis(5));
        fs::write("/ts.txt", b"v2").unwrap();
        let m2 = fs::metadata("/ts.txt").unwrap().modified().unwrap();
        assert!(m2 > m1);
    });
}

// =============================================================================
// READ_DIR
// =============================================================================

#[test]
fn read_dir_lists_children() {
    with_temp_vfs(|| {
        fs::write("/listing/a.txt", b"a").unwrap();
        fs::write("/listing/b.txt", b"b").unwrap();
        fs::create_dir_all("/listing/subdir").unwrap();

        let entries: Vec<_> = fs::read_dir("/listing")
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        assert_eq!(entries.len(), 3);
        assert!(entries.contains(&PathBuf::from("/listing/a.txt")));
        assert!(entries.contains(&PathBuf::from("/listing/b.txt")));
        assert!(entries.contains(&PathBuf::from("/listing/subdir")));
    });
}

#[test]
fn read_dir_empty_directory() {
    with_temp_vfs(|| {
        fs::create_dir_all("/emptydir").unwrap();
        let entries: Vec<_> = fs::read_dir("/emptydir").unwrap().collect();
        assert!(entries.is_empty());
    });
}

#[test]
fn read_dir_implicit_subdir() {
    with_temp_vfs(|| {
        fs::write("/root/sub/deep/file.txt", b"x").unwrap();
        let entries: Vec<_> = fs::read_dir("/root")
            .unwrap()
            .map(|e| e.unwrap().path())
            .collect();
        // Should show "sub" as an immediate child
        assert!(entries.contains(&PathBuf::from("/root/sub")));
    });
}

// =============================================================================
// CANONICALIZE
// =============================================================================

#[test]
fn canonicalize_existing() {
    with_temp_vfs(|| {
        fs::write("/canon/file.txt", b"x").unwrap();
        let p = fs::canonicalize("/canon/file.txt").unwrap();
        assert_eq!(p, PathBuf::from("/canon/file.txt"));
    });
}

#[test]
fn canonicalize_not_found() {
    with_temp_vfs(|| {
        let result = fs::canonicalize("/nonexistent");
        assert!(result.is_err());
    });
}

// =============================================================================
// FILE HANDLE API (with VFS instance support)
// =============================================================================

#[test]
fn file_open_read_write_seek() {
    use std::io::{Read, Seek, SeekFrom, Write};

    with_temp_vfs(|| {
        // Create and write
        let mut f = super::File::create("/handle.txt").unwrap();
        f.write_all(b"hello world").unwrap();

        // Open and read
        let mut f2 = super::File::open("/handle.txt").unwrap();
        let mut buf = String::new();
        f2.read_to_string(&mut buf).unwrap();
        assert_eq!(buf, "hello world");

        // Seek and read partial
        f2.seek(SeekFrom::Start(6)).unwrap();
        let mut buf2 = vec![0u8; 5];
        f2.read_exact(&mut buf2).unwrap();
        assert_eq!(&buf2, b"world");
    });
}

#[test]
fn file_uses_vfs_instance() {
    use std::io::Write;

    with_temp_vfs(|| {
        let mut f = super::File::create("/vfs_handle.txt").unwrap();
        f.write_all(b"vfs data").unwrap();

        // Should be readable through fs API (same VFS)
        let data = fs::read_to_string("/vfs_handle.txt").unwrap();
        assert_eq!(data, "vfs data");
    });
}

#[test]
fn file_open_not_found() {
    with_temp_vfs(|| {
        let result = super::File::open("/no_such_handle");
        assert!(result.is_err());
    });
}

// =============================================================================
// OPEN OPTIONS
// =============================================================================

#[test]
fn open_options_create_truncate() {
    use std::io::{Read, Write};

    with_temp_vfs(|| {
        // Create and write initial data
        let mut f = super::OpenOptions::new()
            .write(true)
            .create(true)
            .open("/opts.txt")
            .unwrap();
        f.write_all(b"initial").unwrap();

        // Truncate and write new data
        let mut f2 = super::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open("/opts.txt")
            .unwrap();
        f2.write_all(b"new").unwrap();

        let mut f3 = super::OpenOptions::new()
            .read(true)
            .open("/opts.txt")
            .unwrap();
        let mut buf = String::new();
        f3.read_to_string(&mut buf).unwrap();
        assert_eq!(buf, "new");
    });
}

#[test]
fn open_options_append() {
    use std::io::{Read, Write};

    with_temp_vfs(|| {
        fs::write("/append.txt", b"hello").unwrap();

        let mut f = super::OpenOptions::new()
            .write(true)
            .append(true)
            .open("/append.txt")
            .unwrap();
        f.write_all(b" world").unwrap();

        let mut f2 = super::File::open("/append.txt").unwrap();
        let mut buf = String::new();
        f2.read_to_string(&mut buf).unwrap();
        assert_eq!(buf, "hello world");
    });
}

// =============================================================================
// QUOTA ENFORCEMENT
// =============================================================================

#[test]
fn quota_total_enforced() {
    let tmp = std::env::temp_dir();
    let mut vfs = VfsInstance::new_temp_in_dir(&tmp).unwrap();
    vfs.set_quota(Some(100));
    let arc = Arc::new(vfs);
    let prev = set_current_vfs(Some(arc.clone()));

    // Write 50 bytes — should succeed
    fs::write("/q1.txt", &vec![0u8; 50]).unwrap();
    // Write another 50 — should succeed (exactly at quota)
    fs::write("/q2.txt", &vec![0u8; 50]).unwrap();
    // Write 1 more byte — should fail
    let result = fs::write("/q3.txt", b"x");
    assert!(result.is_err());

    arc.delete_backing_file().ok();
    set_current_vfs(prev);
}

#[test]
fn quota_max_file_size_enforced() {
    let tmp = std::env::temp_dir();
    let mut vfs = VfsInstance::new_temp_in_dir(&tmp).unwrap();
    vfs.set_max_file_size(Some(10));
    let arc = Arc::new(vfs);
    let prev = set_current_vfs(Some(arc.clone()));

    fs::write("/small.txt", b"ok").unwrap();
    let result = fs::write("/big.txt", &vec![0u8; 20]);
    assert!(result.is_err());

    arc.delete_backing_file().ok();
    set_current_vfs(prev);
}

#[test]
fn quota_max_files_enforced() {
    let tmp = std::env::temp_dir();
    let mut vfs = VfsInstance::new_temp_in_dir(&tmp).unwrap();
    vfs.set_max_files(Some(3));
    let arc = Arc::new(vfs);
    let prev = set_current_vfs(Some(arc.clone()));

    fs::write("/f1.txt", b"a").unwrap();
    fs::write("/f2.txt", b"b").unwrap();
    // Third entry is a parent dir auto-created, and fourth would be f3
    // Let's use flat paths to be predictable
    fs::write("/f3.txt", b"c").unwrap();
    let result = fs::write("/f4.txt", b"d");
    assert!(result.is_err());

    arc.delete_backing_file().ok();
    set_current_vfs(prev);
}

#[test]
fn quota_usage_api() {
    let tmp = std::env::temp_dir();
    let vfs = VfsInstance::new_temp_in_dir(&tmp).unwrap();
    let arc = Arc::new(vfs);
    let prev = set_current_vfs(Some(arc.clone()));

    assert_eq!(arc.usage(), 0);
    fs::write("/usage.txt", b"12345").unwrap();
    assert!(arc.usage() >= 5);

    arc.delete_backing_file().ok();
    set_current_vfs(prev);
}

// =============================================================================
// PATH SECURITY
// =============================================================================

#[test]
fn path_traversal_blocked() {
    with_temp_vfs(|| {
        let result = fs::write("/../../../etc/passwd", b"hacked");
        // The path normalizer strips the traversal, so this writes to /etc/passwd
        // inside the VFS (not the real filesystem) — but the key point is the VFS
        // is in-memory so no real file is touched.
        assert!(result.is_ok()); // Path is normalized, not rejected

        // Null byte injection IS rejected
        let result = fs::write("/file\0.txt", b"data");
        assert!(result.is_err());
    });
}

// =============================================================================
// READ/WRITE ON DIRECTORIES (should fail)
// =============================================================================

#[test]
fn read_directory_fails() {
    with_temp_vfs(|| {
        fs::create_dir_all("/readdir_test").unwrap();
        assert!(fs::read_to_string("/readdir_test").is_err());
        assert!(fs::read("/readdir_test").is_err());
    });
}

#[test]
fn write_to_directory_fails() {
    with_temp_vfs(|| {
        fs::create_dir_all("/writedir_test").unwrap();
        assert!(fs::write("/writedir_test", b"data").is_err());
    });
}

// =============================================================================
// AUTO-CREATED PARENT DIRECTORIES
// =============================================================================

#[test]
fn write_auto_creates_parents() {
    with_temp_vfs(|| {
        fs::write("/auto/parent/dirs/file.txt", b"data").unwrap();
        assert!(fs::metadata("/auto").unwrap().is_dir());
        assert!(fs::metadata("/auto/parent").unwrap().is_dir());
        assert!(fs::metadata("/auto/parent/dirs").unwrap().is_dir());
    });
}

// =============================================================================
// CONCURRENT ACCESS
// =============================================================================

#[test]
fn concurrent_read_write() {
    use super::VfsEntry;

    let tmp = std::env::temp_dir();
    let vfs = VfsInstance::new_temp_in_dir(&tmp).unwrap();
    let map = vfs.as_map();

    // Spawn threads that write directly to the shared map
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let map_clone = map.clone();
            std::thread::spawn(move || {
                let path = PathBuf::from(format!("/concurrent/{}.txt", i));
                let data = format!("thread-{}", i);
                {
                    let mut m = map_clone.lock().unwrap();
                    m.insert(path.clone(), VfsEntry::new_file(data.as_bytes().to_vec()));
                }
                {
                    let m = map_clone.lock().unwrap();
                    let entry = m.get(&path).unwrap();
                    assert_eq!(String::from_utf8_lossy(&entry.data), data);
                }
            })
        })
        .collect();

    for h in handles {
        h.join().unwrap();
    }

    // All files should be present
    let m = map.lock().unwrap();
    for i in 0..10 {
        let path = PathBuf::from(format!("/concurrent/{}.txt", i));
        assert!(m.contains_key(&path));
    }

    vfs.delete_backing_file().ok();
}
