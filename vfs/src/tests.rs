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

#[test]
fn encrypted_persist_and_load() {
    use super::derive_session_key;

    let tmp = std::env::temp_dir();
    let backing = tmp.join("test-encrypted-vfs.bin.enc");

    // Clean up any previous test file
    let _ = stdfs::remove_file(&backing);

    // Create a test key
    let key = derive_session_key("test-session-123");

    // Create VFS and write data
    {
        let vfs = VfsInstance::open_file_backed_encrypted(&backing, &*key).expect("create vfs");
        let arc = Arc::new(vfs);
        let _prev = set_current_vfs(Some(arc.clone()));

        let p = PathBuf::from("/encrypted/secret.txt");
        fs::write(&p, b"sensitive data").expect("write");

        // Persist with encryption
        arc.persist_encrypted(&*key).expect("persist encrypted");
        drop(set_current_vfs(None));
    }

    // Verify encrypted file exists
    assert!(backing.exists());

    // Read the raw bytes - should NOT contain plaintext
    let raw_bytes = stdfs::read(&backing).expect("read raw");
    assert!(!raw_bytes.is_empty());
    // The plaintext should not be visible in the encrypted file
    assert!(!String::from_utf8_lossy(&raw_bytes).contains("sensitive data"));

    // Reload with correct key
    {
        let vfs = VfsInstance::open_file_backed_encrypted(&backing, &*key).expect("reload vfs");
        let arc = Arc::new(vfs);
        let _prev = set_current_vfs(Some(arc.clone()));

        let p = PathBuf::from("/encrypted/secret.txt");
        let s = fs::read_to_string(&p).expect("read decrypted");
        assert_eq!(s, "sensitive data");
        drop(set_current_vfs(None));
    }

    // Cleanup
    let _ = stdfs::remove_file(&backing);
}

#[test]
fn encrypted_wrong_key_fails() {
    use super::derive_session_key;

    let tmp = std::env::temp_dir();
    let backing = tmp.join("test-wrong-key.bin.enc");

    // Clean up any previous test file
    let _ = stdfs::remove_file(&backing);

    // Create and persist with one key
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

    // Try to open with wrong key - should fail
    let key2 = derive_session_key("session-key-2-different");
    let result = VfsInstance::open_file_backed_encrypted(&backing, &*key2);
    assert!(result.is_err());

    // Cleanup
    let _ = stdfs::remove_file(&backing);
}

#[test]
fn derive_session_key_deterministic() {
    use super::derive_session_key;

    // Same session_id should produce same key
    let key1 = derive_session_key("my-session");
    let key2 = derive_session_key("my-session");
    assert_eq!(&*key1, &*key2);

    // Different session_id should produce different key
    let key3 = derive_session_key("other-session");
    assert_ne!(&*key1, &*key3);
}
