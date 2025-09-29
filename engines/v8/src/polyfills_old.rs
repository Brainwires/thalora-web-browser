        global.set(scope, text_encoder_key.into(), text_encoder.into());

        Ok(())
    }
}"[V8 Polyfills] setTimeout: {}ms", timeout_ms);
                
                // Return a timer ID (use a simple counter for now)
                let timer_id = v8::Integer::new(scope, 1);
                rv.set(timer_id.into());
            }
        }).unwrap();

        let set_timeout_key = v8::String::new(scope, "setTimeout").unwrap();
        global.set(scope, set_timeout_key.into(), set_timeout.into());

        Ok(())
    }

    /// Setup Promise polyfills
    fn setup_promises(scope: &mut v8::ContextScope<HandleScope>) -> Result<()> {
        // V8 has native Promise support, so mostly ensure it's accessible
        let global = scope.get_current_context().global(scope);
        
        // Ensure Promise constructor is available
        let _global = scope.get_current_context().global(scope);
        tracing::debug!("[V8 Polyfills] Promise support verified");

        Ok(())
    }

    /// Setup additional console methods beyond basic console.log
    fn setup_console_enhancements(scope: &mut v8::ContextScope<HandleScope>) -> Result<()> {
        let _global = scope.get_current_context().global(scope);
        tracing::debug!("[V8 Polyfills] Console enhancements available");
        
        // console.log is already setup in engine.rs
        // Could add console.error, console.warn, etc. here if needed
        
        Ok(())
    }

    /// Setup URL API
    fn setup_url_api(scope: &mut v8::ContextScope<HandleScope>) -> Result<()> {
        let global = scope.get_current_context().global(scope);

        // URL constructor
        let url_constructor = v8::Function::new(scope, |scope, args: v8::FunctionCallbackArguments, mut rv| {
            if args.length() > 0 {
                let url_str = args.get(0).to_rust_string_lossy(scope);
                tracing::debug!("[V8 Polyfills] URL constructor: {}", url_str);
                
                // Return a basic URL object
                let url_obj = v8::Object::new(scope);
                let href_key = v8::String::new(scope, "href").unwrap();
                let href_val = v8::String::new(scope, &url_str).unwrap();
                url_obj.set(scope, href_key.into(), href_val.into());
                
                rv.set(url_obj.into());
            }
        }).unwrap();

        let url_key = v8::String::new(scope, "URL").unwrap();
        global.set(scope, url_key.into(), url_constructor.into());

        Ok(())
    }

    /// Setup TextEncoder API
    fn setup_text_encoder(scope: &mut v8::ContextScope<HandleScope>) -> Result<()> {
        let global = scope.get_current_context().global(scope);

        // TextEncoder constructor
        let text_encoder = v8::Function::new(scope, |scope, _args, mut rv| {
            let encoder_obj = v8::Object::new(scope);
            
            let encode_func = v8::Function::new(scope, |scope, args: v8::FunctionCallbackArguments, mut rv| {
                if args.length() > 0 {
                    let text = args.get(0).to_rust_string_lossy(scope);
                    let bytes = text.as_bytes();
                    
                    // Create a Uint8Array-like object
                    let array = v8::Array::new(scope, bytes.len() as i32);
                    for (i, &byte) in bytes.iter().enumerate() {
                        let idx = v8::Integer::new(scope, i as i32);
                        let val = v8::Integer::new(scope, byte as i32);
                        array.set(scope, idx.into(), val.into());
                    }
                    rv.set(array.into());
                }
            }).unwrap();
            
            let encode_key = v8::String::new(scope, "encode").unwrap();
            encoder_obj.set(scope, encode_key.into(), encode_func.into());
            
            rv.set(encoder_obj.into());
        }).unwrap();

        let text_encoder_key = v8::String::new(scope, "TextEncoder").unwrap();
        global.set(scope, text_encoder_key.into(), text_encoder.into());

        Ok(())
    }
}"[V8 Polyfills] setTimeout: {}ms", timeout_ms);
                
                // Return a timer ID (use a simple counter for now)
                let timer_id = v8::Integer::new(scope, 1);
                rv.set(timer_id.into());
            }
        }).unwrap();

        let set_timeout_key = v8::String::new(scope, "setTimeout").unwrap();
        global.set(scope, set_timeout_key.into(), set_timeout.into());

        Ok(())
    }

    /// Setup Promise polyfills
    fn setup_promises(scope: &mut v8::ContextScope<HandleScope>) -> Result<()> {
        // V8 has native Promise support, so mostly ensure it's accessible
        let global = scope.get_current_context().global(scope);
        
        // Ensure Promise constructor is available
        let js_code = "if (typeof Promise === 'undefined') { throw new Error('Promise not available'); }";
        let code_v8 = v8::String::new(scope, js_code).unwrap();
        let script = v8::Script::compile(scope, code_v8, None).unwrap();
        script.run(scope);

        Ok(())
    }

    /// Setup URL APIs
    fn setup_url_apis(scope: &mut v8::ContextScope<HandleScope>) -> Result<()> {
        let global = scope.get_current_context().global(scope);

        // Basic URL constructor
        let url_constructor = v8::Function::new(scope, |scope, args: v8::FunctionCallbackArguments, mut rv| {
            if args.length() > 0 {
                let url_str = args.get(0).to_rust_string_lossy(scope);
                tracing::debug!("[V8 Polyfills] URL constructor: {}", url_str);
                
                                // Return a basic URL object\n                let url_obj = v8::Object::new(scope);\n                let href_key = v8::String::new(scope, \"href\").unwrap();\n                let href_val = v8::String::new(scope, &url_str).unwrap();\n                url_obj.set(scope, href_key.into(), href_val.into());\n                \n                rv.set(url_obj.into());\n            }"
        }).unwrap();

        let url_key = v8::String::new(scope, "URL").unwrap();
        global.set(scope, url_key.into(), url_constructor.into());

        Ok(())
    }

    /// Setup text encoding/decoding APIs
    fn setup_encoding_apis(scope: &mut v8::ContextScope<HandleScope>) -> Result<()> {
        let global = scope.get_current_context().global(scope);

        // TextEncoder
        let text_encoder = v8::Function::new(scope, |scope, _args, mut rv| {
            let encoder_obj = v8::Object::new(scope);
            
            let encode_func = v8::Function::new(scope, |scope, args: v8::FunctionCallbackArguments, mut rv| {
                if args.length() > 0 {
                    let text = args.get(0).to_rust_string_lossy(scope);
                    let bytes = text.as_bytes();
                    
                    // Create a Uint8Array-like object
                    let array = v8::Array::new(scope, bytes.len() as i32);\n                    for (i, &byte) in bytes.iter().enumerate() {\n                        let idx = v8::Integer::new(scope, i as i32);\n                        let val = v8::Integer::new(scope, byte as i32);\n                        array.set(scope, idx.into(), val.into());\n                    }\n                    rv.set(array.into());\n                }\n            }).unwrap();"
            
            let encode_key = v8::String::new(scope, "encode").unwrap();
            encoder_obj.set(scope, encode_key.into(), encode_func.into());
            
            rv.set(encoder_obj.into());
        }).unwrap();

        let text_encoder_key = v8::String::new(scope, "TextEncoder").unwrap();
        global.set(scope, text_encoder_key.into(), text_encoder.into());

        Ok(())
    }
}