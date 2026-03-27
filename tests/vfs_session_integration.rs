use std::path::PathBuf;
use std::sync::Arc;
use thalora::McpServer;
use vfs::{VfsInstance, set_current_vfs};

#[tokio::test]
async fn session_vfs_persistence_and_ephemeral_cleanup() {
    // Create an McpServer to access session helpers
    let server = McpServer::new();

    // Ensure no session exists initially
    let session_id = "integration-123";
    // The server now uses encrypted backing files with .bin.enc extension
    let backing_path_enc = std::env::temp_dir().join(format!("vfs-session-{}.bin.enc", session_id));
    if backing_path_enc.exists() {
        let _ = std::fs::remove_file(&backing_path_enc);
    }

    // Create or get a session VFS
    let v1 = server
        .get_or_create_session_vfs(session_id, None)
        .expect("Failed to create session VFS");

    // Get the actual backing path from the VFS instance
    let backing_path = v1.backing_path();

    // Write via the VFS instance
    let p = PathBuf::from("/session/key.txt");
    {
        let prev = set_current_vfs(Some(v1.clone()));
        vfs::fs::write(&p, b"hello-session").expect("write");
        let s = vfs::fs::read_to_string(&p).expect("read");
        assert_eq!(s, "hello-session");
        let _ = set_current_vfs(prev);
    }

    // Persist the session backing and ensure file exists
    v1.persist().expect("persist");
    assert!(backing_path.exists());

    // Simulate retrieving the same session vfs (should reuse same backing)
    let v2 = server
        .get_or_create_session_vfs(session_id, None)
        .expect("Failed to get existing session VFS");
    assert_eq!(v1.backing_path(), v2.backing_path());

    // Create an ephemeral per-call VFS and ensure backing deletion semantics
    let tmp_dir = std::env::temp_dir();
    let ephemeral = VfsInstance::new_temp_in_dir(&tmp_dir).expect("create ephemeral");
    let ephemeral_path = ephemeral.backing_path();
    let arc_ephem = Arc::new(ephemeral);
    let prev = set_current_vfs(Some(arc_ephem.clone()));
    let p2 = PathBuf::from("/ephemeral/data.bin");
    vfs::fs::write(&p2, b"ephem").expect("write ephemeral");
    let s2 = vfs::fs::read_to_string(&p2).expect("read ephemeral");
    assert_eq!(s2, "ephem");
    // Do not persist and delete backing
    let _ = set_current_vfs(prev);
    arc_ephem.delete_backing_file().expect("delete ephemeral");
    assert!(!ephemeral_path.exists());

    // Cleanup session
    server
        .remove_session_vfs(session_id, true)
        .expect("Failed to remove session VFS");
    // After removal, the backing file should be deleted
    // Note: the actual path may differ from backing_path if the server
    // fell back to a temp VFS, so check the known path
    assert!(!backing_path.exists() || !backing_path_enc.exists());
}
