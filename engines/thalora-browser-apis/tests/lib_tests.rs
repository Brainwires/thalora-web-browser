//! Tests for lib

use thalora_browser_apis::boa_engine::{Context, JsValue, JsResult};
use thalora_browser_apis::lib::*;

use super::*;

    #[test]
    fn test_api_initialization() {
        let mut context = boa_engine::Context::default();
        assert!(initialize_browser_apis(&mut context).is_ok());
    }
