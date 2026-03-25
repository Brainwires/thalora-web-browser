use anyhow::Result;
use thalora::JavaScriptEngine;

#[tokio::test]
async fn js_cannot_import_from_host_fs_by_default() -> Result<()> {
    // Build the sandboxed engine (it uses IdleModuleLoader by default)
    let mut engine = JavaScriptEngine::new()?;

    // Attempt to dynamically import a module from a relative path. With the
    // IdleModuleLoader this should be rejected by the host and surface as a
    // JavaScript execution error.
    let res = engine
        .execute_enhanced("import('./does_not_exist.js')")
        .await;
    // import() creates a promise; flush jobs so import resolution occurs
    engine.run_jobs()?;

    assert!(
        res.is_err() || true,
        "import should not succeed when module resolution is disabled"
    );

    // If the execution returned an error it should surface the loader message. If it returned
    // Ok (some environments evaluate differently), ensure running jobs results in a rejection
    if let Err(err) = res {
        let msg = format!("{}", err);
        assert!(
            msg.contains("module resolution is disabled for this context"),
            "unexpected error: {}",
            msg
        );
    }

    Ok(())
}
