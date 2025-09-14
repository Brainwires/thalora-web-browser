use anyhow::{anyhow, Result};
use boa_engine::JsResult;
use boa_engine::{
    Context, JsError, JsNativeError, JsObject, JsValue, NativeFunction, Source,
    js_string, property::Attribute,
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

pub struct EnhancedJavaScriptEngine {
    context: Context,
    timers: Arc<Mutex<HashMap<u32, TimerHandle>>>,
    next_timer_id: Arc<Mutex<u32>>,
    promises: Vec<JsObject>,
    start_time: Instant,
}

#[derive(Debug)]
struct TimerHandle {
    id: u32,
    callback: JsValue,
    interval: Option<Duration>,
    created_at: Instant,
}

impl EnhancedJavaScriptEngine {
    pub fn new() -> Result<Self> {
        let mut context = Context::default();
        let timers = Arc::new(Mutex::new(HashMap::new()));
        let next_timer_id = Arc::new(Mutex::new(1));

        Self::setup_enhanced_globals(&mut context, timers.clone(), next_timer_id.clone())
            .map_err(|e| anyhow!("Failed to setup enhanced globals: {}", e))?;

        Ok(Self {
            context,
            timers,
            next_timer_id,
            promises: Vec::new(),
            start_time: Instant::now(),
        })
    }

    fn setup_enhanced_globals(
        context: &mut Context,
        timers: Arc<Mutex<HashMap<u32, TimerHandle>>>,
        next_timer_id: Arc<Mutex<u32>>,
    ) -> JsResult<()> {
        // Enhanced console with multiple log levels
        let console_obj = JsObject::default();
        
        // Console.log
        let log_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            let message = args
                .iter()
                .map(|arg| arg.to_string(context).unwrap_or_default().to_std_string_escaped())
                .collect::<Vec<_>>()
                .join(" ");
            tracing::info!("JS Console: {}", message);
            Ok(JsValue::undefined())
        });
        console_obj.set(js_string!("log"), log_fn, false, context)?;

        // Console.error
        let error_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            let message = args
                .iter()
                .map(|arg| arg.to_string(context).unwrap_or_default().to_std_string_escaped())
                .collect::<Vec<_>>()
                .join(" ");
            tracing::error!("JS Console Error: {}", message);
            Ok(JsValue::undefined())
        });
        console_obj.set(js_string!("error"), error_fn, false, context)?;

        context.register_global_property(js_string!("console"), console_obj, Attribute::all())?;

        // Enhanced setTimeout (simplified for thread safety)
        let set_timeout_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            if args.len() < 2 {
                return Err(JsError::from_native(
                    JsNativeError::typ()
                        .with_message("setTimeout requires callback and delay")
                ));
            }

            let callback = args[0].clone();
            let delay_ms = args[1].to_number(context)? as u64;
            
            // For now, just log the timer setup
            tracing::debug!("setTimeout called with delay: {}ms", delay_ms);

            Ok(JsValue::from(1u32)) // Return dummy timer ID
        });
        context.register_global_property(js_string!("setTimeout"), set_timeout_fn, Attribute::all())?;

        // Performance.now() for timing
        let performance_now_fn = NativeFunction::from_fn_ptr(|_, _, _context| {
            // Return current timestamp in milliseconds since epoch
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default();
            let ms = now.as_secs_f64() * 1000.0 + now.subsec_nanos() as f64 / 1_000_000.0;
            Ok(JsValue::from(ms))
        });
        
        let performance_obj = JsObject::default();
        performance_obj.set(js_string!("now"), performance_now_fn, false, context)?;
        context.register_global_property(js_string!("performance"), performance_obj, Attribute::all())?;

        // Date.now() override for consistency
        context.eval(Source::from_bytes(
            "Date.now = function() { return performance.now() + Date.UTC(1970, 0, 1); };"
        ))?;

        // Enhanced fetch API (simplified implementation)
        let fetch_fn = NativeFunction::from_fn_ptr(|_, args, _context| {
            if args.is_empty() {
                return Err(JsError::from_native(
                    JsNativeError::typ()
                        .with_message("fetch requires a URL")
                ));
            }

            let url = args[0].to_string(_context)?.to_std_string_escaped();
            
            // Create a Promise (simplified - would need proper Promise implementation)
            let promise = _context.eval(Source::from_bytes(&format!(
                r#"
                new Promise((resolve, reject) => {{
                    // Simulate fetch response
                    setTimeout(() => {{
                        resolve({{
                            ok: true,
                            status: 200,
                            json: () => Promise.resolve({{}}),
                            text: () => Promise.resolve(''),
                            url: '{}'
                        }});
                    }}, 10);
                }})
                "#,
                url
            )))?;

            Ok(promise)
        });
        context.register_global_property(js_string!("fetch"), fetch_fn, Attribute::all())?;

        // URL Constructor
        let url_constructor = NativeFunction::from_fn_ptr(|_, args, context| {
            if args.is_empty() {
                return Err(JsError::from_native(
                    JsNativeError::typ()
                        .with_message("URL constructor requires a URL string")
                ));
            }

            let url_str = args[0].to_string(context)?.to_std_string_escaped();
            
            // Basic URL parsing (would use a proper URL parser in practice)
            let url_obj = JsObject::default();
            url_obj.set(js_string!("href"), JsValue::from(js_string!(url_str.clone())), false, context)?;
            url_obj.set(js_string!("toString"), JsValue::from(js_string!(url_str.clone())), false, context)?;

            Ok(JsValue::from(url_obj))
        });
        context.register_global_property(js_string!("URL"), url_constructor, Attribute::all())?;

        // JSON enhancements
        context.eval(Source::from_bytes(r#"
            // Enhanced JSON with better error handling
            window.JSON = JSON;
            window.JSON.safeParse = function(str) {
                try {
                    return { success: true, data: JSON.parse(str) };
                } catch (e) {
                    return { success: false, error: e.message };
                }
            };
        "#))?;

        Ok(())
    }

    pub async fn execute_enhanced(&mut self, code: &str) -> Result<JsValue> {
        // Pre-process the code to handle modern JS patterns
        let processed_code = self.preprocess_modern_js(code)?;

        let _execution_timeout = Duration::from_secs(10);
        
        // Execute synchronously for now due to Boa thread safety constraints
        let result = self.context.eval(Source::from_bytes(&processed_code))
            .map_err(|e| anyhow!("JavaScript execution error: {}", e))?;
        
        // Process any pending promises (simplified)
        // self.process_pending_promises().await;
        
        Ok(result)
    }

    fn preprocess_modern_js(&self, code: &str) -> Result<String> {
        // Use SWC to parse and transform modern JavaScript
        use swc_core::ecma::{
            parser::{lexer::Lexer, Parser, StringInput, Syntax},
        };
        use swc_common::{SourceMap, FilePathMapping};
        
        let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
        let fm = cm.new_source_file(
            swc_common::FileName::Custom("input.js".into()),
            code.to_string()
        );

        let lexer = Lexer::new(
            Syntax::Es(Default::default()),
            Default::default(),
            StringInput::from(&*fm),
            None,
        );

        let mut parser = Parser::new_from(lexer);
        
        match parser.parse_module() {
            Ok(_module) => {
                // For now, return the original code
                // In a full implementation, we would transform:
                // - import/export statements to compatible forms
                // - async/await to promise chains
                // - arrow functions to regular functions
                // - template literals to string concatenation
                Ok(self.transform_basic_patterns(code))
            }
            Err(_) => {
                // If parsing fails, try basic transformations
                Ok(self.transform_basic_patterns(code))
            }
        }
    }

    fn transform_basic_patterns(&self, code: &str) -> String {
        let mut transformed = code.to_string();

        // Transform arrow functions to regular functions (basic)
        transformed = regex::Regex::new(r"const\s+(\w+)\s*=\s*\(([^)]*)\)\s*=>\s*\{")
            .unwrap()
            .replace_all(&transformed, "function $1($2) {")
            .to_string();

        // Transform arrow functions (expression form)
        transformed = regex::Regex::new(r"const\s+(\w+)\s*=\s*\(([^)]*)\)\s*=>\s*([^;]+);?")
            .unwrap()
            .replace_all(&transformed, "function $1($2) { return $3; }")
            .to_string();

        // Transform template literals to string concatenation (basic)
        transformed = regex::Regex::new(r"`([^`]*)`")
            .unwrap()
            .replace_all(&transformed, |caps: &regex::Captures| {
                let content = &caps[1];
                // Basic variable substitution
                let with_vars = regex::Regex::new(r"\$\{([^}]+)\}")
                    .unwrap()
                    .replace_all(content, "' + ($1) + '");
                format!("'{}'", with_vars)
            })
            .to_string();

        // Transform let/const to var
        transformed = regex::Regex::new(r"\b(let|const)\b")
            .unwrap()
            .replace_all(&transformed, "var")
            .to_string();

        transformed
    }

    async fn process_pending_promises(&mut self) {
        // Process any promises that might be pending
        // This is a simplified implementation
        let mut completed_promises = Vec::new();
        
        for (i, promise) in self.promises.iter().enumerate() {
            // Check promise state (simplified)
            if let Ok(state) = promise.get(js_string!("state"), &mut self.context) {
                if state.is_string() {
                    let state_str = state.to_string(&mut self.context)
                        .unwrap_or_default()
                        .to_std_string_escaped();
                    if state_str == "fulfilled" || state_str == "rejected" {
                        completed_promises.push(i);
                    }
                }
            }
        }

        // Remove completed promises
        for &i in completed_promises.iter().rev() {
            self.promises.remove(i);
        }
    }

    pub fn get_global_object(&mut self, name: &str) -> Result<Option<JsValue>> {
        match self.context.global_object().get(js_string!(name), &mut self.context) {
            Ok(value) if !value.is_undefined() => Ok(Some(value)),
            Ok(_) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    pub fn set_global_object(&mut self, name: &str, value: JsValue) -> Result<()> {
        self.context.global_object()
            .set(js_string!(name), value, true, &mut self.context)
            .map_err(|e| anyhow!("Failed to set global object: {}", e))?;
        Ok(())
    }
}