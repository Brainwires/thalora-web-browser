// Tests for src/apis/polyfills/console.rs
#[cfg(test)]
mod console_tests {
    use boa_engine::Context;

    #[test]
    fn test_console_setup() {
        let mut context = Context::default();
        // Test that console setup doesn't panic
        let result = super::super::super::apis::polyfills::console::setup_console(&mut context);
        assert!(result.is_ok());
    }
}