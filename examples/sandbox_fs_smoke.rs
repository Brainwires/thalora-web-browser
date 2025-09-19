use thalora::JavaScriptEngine;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let mut engine = JavaScriptEngine::new()?;
    println!("Created engine with IdleModuleLoader");

    match engine.execute_enhanced("import('./does_not_exist.js')").await {
        Ok(v) => {
            println!("Got value from import(): {:?}. Now running jobs to settle the promise...", v);
            // Flush microtasks / jobs so the dynamic import attempt actually completes.
            if let Err(e) = engine.run_jobs() {
                println!("Run jobs error: {}", e);
            }
        }
        Err(e) => println!("Import rejected immediately as expected: {}", e),
    }

    Ok(())
}
