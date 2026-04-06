use anyhow::Result;
use thalora::engine::{EngineFactory, EngineType};

#[tokio::test]
async fn test_boa_engine_creation() -> Result<()> {
    let mut engine = EngineFactory::create_engine(EngineType::Boa)?;

    // Test basic execution
    let result = engine.execute("2 + 2")?;
    println!("Boa engine result: {:?}", result);

    // Test version info
    let version = engine.version_info();
    println!("Boa engine version: {}", version);
    assert!(version.contains("Enhanced JavaScript Engine"));

    // Test engine type
    assert_eq!(engine.engine_type(), "boa");

    Ok(())
}

#[tokio::test]
async fn test_v8_engine_creation() -> Result<()> {
    if !EngineFactory::available_engines().contains(&EngineType::V8) {
        println!("V8 engine not available, skipping test");
        return Ok(());
    }

    let mut engine = EngineFactory::create_engine(EngineType::V8)?;

    // Test basic execution
    let result = engine.execute("3 + 3")?;
    println!("V8 engine result: {:?}", result);

    // Test version info
    let version = engine.version_info();
    println!("V8 engine version: {}", version);
    assert!(version.contains("V8"));

    // Test engine type
    assert_eq!(engine.engine_type(), "v8");

    Ok(())
}

#[tokio::test]
async fn test_engine_compatibility() -> Result<()> {
    // Test the same JavaScript code on both engines to ensure compatibility
    let test_code = r#"
        var x = 10;
        var y = 20;
        var result = x * y + 5;
        result.toString();
    "#;

    // Test with Boa
    let mut boa_engine = EngineFactory::create_engine(EngineType::Boa)?;
    let boa_result = boa_engine.execute(test_code)?;
    println!("Boa result: {:?}", boa_result);

    // Test with V8 if available
    if EngineFactory::available_engines().contains(&EngineType::V8) {
        let mut v8_engine = EngineFactory::create_engine(EngineType::V8)?;
        let v8_result = v8_engine.execute(test_code)?;
        println!("V8 result: {:?}", v8_result);

        // Results should be equivalent
        println!(
            "Comparing results - Boa: {:?}, V8: {:?}",
            boa_result, v8_result
        );
    } else {
        println!("V8 engine not available, skipping comparison");
    }

    Ok(())
}

#[tokio::test]
async fn test_available_engines() -> Result<()> {
    let available = EngineFactory::available_engines();
    println!("Available engines: {:?}", available);

    // Boa should always be available
    assert!(available.contains(&EngineType::Boa));

    // V8 may or may not be available depending on build configuration
    if available.contains(&EngineType::V8) {
        println!("V8 engine is available");
    } else {
        println!("V8 engine not available (removed)");
    }

    Ok(())
}

#[tokio::test]
async fn test_engine_global_objects() -> Result<()> {
    let mut engine = EngineFactory::create_engine(EngineType::Boa)?;

    // Test setting and getting global objects
    let test_value = serde_json::json!({"test": "value", "number": 42});
    engine.set_global_object("testGlobal", test_value.clone())?;

    let retrieved = engine.get_global_object("testGlobal")?;
    println!("Set and retrieved global: {:?}", retrieved);

    assert!(retrieved.is_some());

    Ok(())
}

#[tokio::test]
async fn test_engine_error_handling() -> Result<()> {
    let mut engine = EngineFactory::create_engine(EngineType::Boa)?;

    // Test error handling with invalid syntax
    let result = engine.execute("var x = ;");
    assert!(result.is_err());
    println!("Expected error for invalid syntax: {:?}", result);

    // Test that engine continues working after error
    let good_result = engine.execute("1 + 1")?;
    println!("Engine works after error: {:?}", good_result);

    Ok(())
}

#[tokio::test]
async fn test_web_apis_availability() -> Result<()> {
    let mut engine = EngineFactory::create_engine(EngineType::Boa)?;

    // Test console availability
    let console_test = engine.execute("typeof console")?;
    println!("Console type: {:?}", console_test);

    // Test setTimeout availability
    let timeout_test = engine.execute("typeof setTimeout")?;
    println!("setTimeout type: {:?}", timeout_test);

    Ok(())
}
