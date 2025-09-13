use anyhow::{anyhow, Result};
use boa_engine::{
    builtins::promise::PromiseState, Context, JsError, JsObject, JsValue, NativeFunction, Source,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::time::{sleep, timeout};

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

        Self::setup_enhanced_globals(&mut context, timers.clone(), next_timer_id.clone())?;

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
    ) -> Result<()> {
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
        let error_fn = NativeFunction::from_fn_ptr(|_, args, _| {
            let message = args
                .iter()
                .map(|arg| arg.to_string(context).unwrap_or_default().to_std_string_escaped())
                .collect::<Vec<_>>()
                .join(" ");
            tracing::error!("JS Console Error: {}", message);
            Ok(JsValue::undefined())
        });
        console_obj.set("error", error_fn, false, context)?;

        context.register_global_property("console", console_obj, boa_engine::property::Attribute::all())?;

        // Enhanced setTimeout with real timing
        let timers_clone = timers.clone();
        let timer_id_clone = next_timer_id.clone();
        let set_timeout_fn = NativeFunction::from_fn_ptr(move |_, args, _| {
            if args.len() < 2 {
                return Err(JsError::from_native("setTimeout requires callback and delay"));
            }

            let callback = args[0].clone();
            let delay_ms = args[1].to_number(context)? as u64;
            
            let rt = tokio::runtime::Handle::current();
            let timers = timers_clone.clone();
            let timer_id_ref = timer_id_clone.clone();
            
            rt.spawn(async move {
                let timer_id = {
                    let mut id_guard = timer_id_ref.lock().await;
                    let id = *id_guard;
                    *id_guard += 1;
                    id
                };

                let timer = TimerHandle {
                    id: timer_id,
                    callback: callback.clone(),
                    interval: None,
                    created_at: Instant::now(),
                };

                {
                    let mut timers_guard = timers.lock().await;
                    timers_guard.insert(timer_id, timer);
                }

                sleep(Duration::from_millis(delay_ms)).await;

                // Execute callback (simplified - would need proper context handling)
                tracing::debug!("Timer {} fired after {}ms", timer_id, delay_ms);
                
                {
                    let mut timers_guard = timers.lock().await;
                    timers_guard.remove(&timer_id);
                }
            });

            Ok(JsValue::from(1)) // Return dummy timer ID
        });
        context.register_global_callable("setTimeout", 2, set_timeout_fn)?;

        // Performance.now() for timing
        let start_time = Instant::now();
        let performance_now_fn = NativeFunction::from_fn_ptr(move |_, _, _| {
            let elapsed = start_time.elapsed();
            let ms = elapsed.as_secs_f64() * 1000.0;
            Ok(JsValue::from(ms))
        });
        
        let performance_obj = JsObject::default();
        performance_obj.set("now", performance_now_fn, false, context)?;
        context.register_global_property("performance", performance_obj, boa_engine::property::Attribute::all())?;

        // Date.now() override for consistency
        context.eval(Source::from_bytes(
            "Date.now = function() { return performance.now() + Date.UTC(1970, 0, 1); };"
        ))?;

        // Enhanced fetch API (simplified implementation)
        let fetch_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            if args.is_empty() {
                return Err(JsError::from_native("fetch requires a URL"));
            }

            let url = args[0].to_string(context)?.to_std_string_escaped();
            
            // Create a Promise (simplified - would need proper Promise implementation)
            let promise = context.eval(Source::from_bytes(&format!(
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
        context.register_global_callable("fetch", 1, fetch_fn)?;

        // URL Constructor
        let url_constructor = NativeFunction::from_fn_ptr(|_, args, context| {
            if args.is_empty() {
                return Err(JsError::from_native("URL constructor requires a URL string"));
            }

            let url_str = args[0].to_string(context)?.to_std_string_escaped();
            
            // Basic URL parsing (would use a proper URL parser in practice)
            let url_obj = JsObject::default();
            url_obj.set("href", JsValue::from(url_str.clone()), false, context)?;
            url_obj.set("toString", NativeFunction::from_fn_ptr(move |_, _, _| {
                Ok(JsValue::from(url_str.clone()))
            }), false, context)?;

            Ok(JsValue::from(url_obj))
        });
        context.register_global_class(url_constructor)?;

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

        let execution_timeout = Duration::from_secs(10);
        
        match timeout(execution_timeout, async {
            let result = self.context.eval(Source::from_bytes(&processed_code));
            
            // Process any pending promises (simplified)
            self.process_pending_promises().await;
            
            result
        }).await {
            Ok(result) => result.map_err(|e| anyhow!("JavaScript execution error: {}", e)),
            Err(_) => Err(anyhow!("JavaScript execution timed out")),
        }
    }

    fn preprocess_modern_js(&self, code: &str) -> Result<String> {
        // Use SWC to parse and transform modern JavaScript
        use swc_core::ecma::{
            parser::{lexer::Lexer, Parser, StringInput, Syntax},
            ast::Module,
        };
        use swc_common::{SourceMap, FilePathMapping};
        
        let cm = Arc::new(SourceMap::new(FilePathMapping::empty()));
        let fm = cm.new_source_file(
            swc_common::FileName::Custom("input.js".into()),
            code.to_string()
        );

        let lexer = Lexer::new(
            Syntax::es(Default::default()),
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
            if let Ok(state) = promise.get("state", &mut self.context) {
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
        match self.context.global_object().get(name, &mut self.context) {
            Ok(value) if !value.is_undefined() => Ok(Some(value)),
            Ok(_) => Ok(None),
            Err(_) => Ok(None),
        }
    }

    pub fn set_global_object(&mut self, name: &str, value: JsValue) -> Result<()> {
        self.context.global_object()
            .set(name, value, true, &mut self.context)
            .map_err(|e| anyhow!("Failed to set global object: {}", e))?;
        Ok(())
    }
}