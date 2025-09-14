use synaptic::enhanced_web_apis::EnhancedWebApis;
use boa_engine::{Context, js_string};

fn main() -> anyhow::Result<()> {
    let mut context = Context::default();
    
    // Setup enhanced web APIs
    EnhancedWebApis::setup_enhanced_apis(&mut context)?;
    
    // Test that fetch API is available
    let result = context.eval("typeof window.fetch");
    match result {
        Ok(value) => println!("window.fetch type: {}", value.to_string(&mut context)?),
        Err(e) => println!("Error: {}", e)
    }
    
    // Test that URL constructor is available
    let result = context.eval("typeof window.URL");
    match result {
        Ok(value) => println!("window.URL type: {}", value.to_string(&mut context)?),
        Err(e) => println!("Error: {}", e)
    }
    
    // Test that crypto API is available
    let result = context.eval("typeof window.crypto");
    match result {
        Ok(value) => println!("window.crypto type: {}", value.to_string(&mut context)?),
        Err(e) => println!("Error: {}", e)
    }
    
    println!("Enhanced web APIs test completed successfully!");
    Ok(())
}