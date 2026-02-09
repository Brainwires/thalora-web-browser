use anyhow::Result;
use super::{EngineFactory, EngineType, ThaloraBrowserEngine};

/// Test helper functions for engine selection

/// Create a JavaScript engine using the configured test engine
pub fn create_test_engine() -> Result<Box<dyn ThaloraBrowserEngine>> {
    EngineFactory::create_default_engine()
}

/// Get the configured test engine type
pub fn get_test_engine_type() -> EngineType {
    EngineFactory::default_engine()
}

/// Check if tests are configured to use Boa
pub fn is_using_boa() -> bool {
    matches!(get_test_engine_type(), EngineType::Boa)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_selection() {
        // Should create an engine without error
        let result = create_test_engine();
        assert!(result.is_ok(), "Failed to create test engine");
    }

    #[test]
    fn test_engine_type_detection() {
        let engine_type = get_test_engine_type();

        // Should be one of the available engines
        assert!(
            EngineFactory::available_engines().contains(&engine_type),
            "Engine type {:?} not in available engines",
            engine_type
        );
    }
}
