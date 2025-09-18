use anyhow::Result;
use boa_engine::{Context, JsObject, JsValue, NativeFunction, js_string, Source};
use std::sync::{Arc, Mutex};
use crate::engine::browser::HeadlessWebBrowser;

/// Real History API implementation with browser navigation
pub struct BrowserHistory {
    browser: Arc<Mutex<HeadlessWebBrowser>>,
}

impl BrowserHistory {
    pub fn new(browser: Arc<Mutex<HeadlessWebBrowser>>) -> Self {
        Self { browser }
    }

    /// Setup real History API globals
    pub fn setup_history_globals(&self, context: &mut Context) -> Result<()> {
        self.setup_history_api(context).map_err(|e| anyhow::Error::msg(format!("History API setup failed: {:?}", e)))?;
        Ok(())
    }

    fn setup_history_api(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        let history_obj = JsObject::default();

        // Get initial length from browser
        let length = {
            let browser = self.browser.lock().unwrap();
            browser.get_history().entries.len() as i32
        };
        history_obj.set(js_string!("length"), JsValue::from(length), false, context)?;

        // Get initial state from browser (not implemented - default to null)
        let state = JsValue::null();
        history_obj.set(js_string!("state"), state, false, context)?;

        // scrollRestoration property
        history_obj.set(js_string!("scrollRestoration"), JsValue::from(js_string!("auto")), false, context)?;

        // history.back()
    let browser_back: Arc<Mutex<crate::engine::browser::HeadlessWebBrowser>> = Arc::clone(&self.browser);
        let back_fn = unsafe { NativeFunction::from_closure(move |_, _args, _context| {
            if let Ok(mut browser) = browser_back.try_lock() {
                match tokio::runtime::Handle::try_current() {
                    Ok(handle) => {
                        handle.block_on(async {
                            match browser.go_back().await {
                                Ok(Some(_scraped_data)) => {
                                    // Navigation successful
                                    tracing::info!("🔙 History back navigation completed");
                                },
                                Ok(None) => {
                                    tracing::info!("🔙 Cannot navigate back - at beginning of history");
                                },
                                Err(e) => {
                                    tracing::error!("🔙 History back navigation failed: {}", e);
                                }
                            }
                        });
                    },
                    Err(_) => {
                        tracing::warn!("🔙 History back called outside async context - navigation may not work");
                    }
                }
            } else {
                tracing::warn!("🔙 Browser locked - cannot perform history back");
            }
            Ok(JsValue::undefined())
        }) };
        history_obj.set(js_string!("back"), JsValue::from(back_fn.to_js_function(context.realm())), false, context)?;

        // history.forward()
        let browser_forward: Arc<Mutex<crate::engine::browser::HeadlessWebBrowser>> = Arc::clone(&self.browser);
        let forward_fn = unsafe { NativeFunction::from_closure(move |_, _args, _context| {
                if let Ok(mut browser) = browser_forward.try_lock() {
                match tokio::runtime::Handle::try_current() {
                    Ok(handle) => {
                        handle.block_on(async {
                                match browser.go_forward().await {
                                Ok(Some(_scraped_data)) => {
                                    tracing::info!("🔜 History forward navigation completed");
                                },
                                Ok(None) => {
                                    tracing::info!("🔜 Cannot navigate forward - at end of history");
                                },
                                Err(e) => {
                                    tracing::error!("🔜 History forward navigation failed: {}", e);
                                }
                            }
                        });
                    },
                    Err(_) => {
                        tracing::warn!("🔜 History forward called outside async context - navigation may not work");
                    }
                }
            } else {
                tracing::warn!("🔜 Browser locked - cannot perform history forward");
            }
            Ok(JsValue::undefined())
        }) };
        history_obj.set(js_string!("forward"), JsValue::from(forward_fn.to_js_function(context.realm())), false, context)?;

        // history.go(delta)
        let browser_go: Arc<Mutex<crate::engine::browser::HeadlessWebBrowser>> = Arc::clone(&self.browser);
        let go_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            let delta = if !args.is_empty() {
                args[0].to_i32(context).unwrap_or(0)
            } else {
                0
            };

                if let Ok(mut browser) = browser_go.try_lock() {
                    match tokio::runtime::Handle::try_current() {
                        Ok(handle) => {
                            handle.block_on(async {
                                // Simple implementation: if delta < 0 -> go_back, if >0 -> go_forward
                                if delta < 0 {
                                    match browser.go_back().await {
                                        Ok(Some(_)) => tracing::info!("🎯 History go({}) navigation completed", delta),
                                        Ok(None) => tracing::info!("🎯 Cannot navigate go({}) - invalid history position", delta),
                                        Err(e) => tracing::error!("🎯 History go({}) navigation failed: {}", delta, e),
                                    }
                                } else if delta > 0 {
                                    match browser.go_forward().await {
                                        Ok(Some(_)) => tracing::info!("🎯 History go({}) navigation completed", delta),
                                        Ok(None) => tracing::info!("🎯 Cannot navigate go({}) - invalid history position", delta),
                                        Err(e) => tracing::error!("🎯 History go({}) navigation failed: {}", delta, e),
                                    }
                                }
                            });
                        },
                        Err(_) => {
                            tracing::warn!("🎯 History go called outside async context - navigation may not work");
                        }
                    }
                } else {
                    tracing::warn!("🎯 Browser locked - cannot perform history go");
                }
            Ok(JsValue::undefined())
        }) };
        history_obj.set(js_string!("go"), JsValue::from(go_fn.to_js_function(context.realm())), false, context)?;

        // history.pushState(state, title, url)
        let browser_push: Arc<Mutex<crate::engine::browser::HeadlessWebBrowser>> = Arc::clone(&self.browser);
        let push_state_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if args.len() >= 3 {
                let state = if args[0].is_null() || args[0].is_undefined() {
                    None
                } else {
                    // Convert JsValue to JSON
                    match serde_json::to_value(&args[0].to_json(context)?) {
                        Ok(json_val) => Some(json_val),
                        Err(_) => None,
                    }
                };

                let title = if args[1].is_null() || args[1].is_undefined() {
                    None
                } else {
                    Some(args[1].to_string(context)?.to_std_string_escaped())
                };

                let url = args[2].to_string(context)?.to_std_string_escaped();

                if let Ok(_browser) = browser_push.try_lock() {
                    // For now, we don't modify internal history (private). In future, expose API.
                    tracing::info!("📌 History pushState requested: {}", url);
                } else {
                    tracing::warn!("📌 Browser locked - cannot perform pushState");
                }
            }
            Ok(JsValue::undefined())
        }) };
        history_obj.set(js_string!("pushState"), JsValue::from(push_state_fn.to_js_function(context.realm())), false, context)?;

        // history.replaceState(state, title, url)
        let browser_replace: Arc<Mutex<crate::engine::browser::HeadlessWebBrowser>> = Arc::clone(&self.browser);
        let replace_state_fn = unsafe { NativeFunction::from_closure(move |_, args, context| {
            if args.len() >= 3 {
                let state = if args[0].is_null() || args[0].is_undefined() {
                    None
                } else {
                    // Convert JsValue to JSON
                    match serde_json::to_value(&args[0].to_json(context)?) {
                        Ok(json_val) => Some(json_val),
                        Err(_) => None,
                    }
                };

                let title = if args[1].is_null() || args[1].is_undefined() {
                    None
                } else {
                    Some(args[1].to_string(context)?.to_std_string_escaped())
                };

                let url = args[2].to_string(context)?.to_std_string_escaped();

                if let Ok(_browser) = browser_replace.try_lock() {
                    // Not changing internal history (private). Log replacement request.
                    tracing::info!("🔄 History replaceState requested: {}", url);
                } else {
                    tracing::warn!("🔄 Browser locked - cannot perform replaceState");
                }
            }
            Ok(JsValue::undefined())
        }) };
        history_obj.set(js_string!("replaceState"), JsValue::from(replace_state_fn.to_js_function(context.realm())), false, context)?;

        // Set the history object as a global
        let global_object = context.global_object().clone();
        global_object.set(js_string!("history"), JsValue::from(history_obj), false, context)?;

        // Also add it to the window object if it exists
        if let Ok(window_val) = global_object.get(js_string!("window"), context) {
            if let Some(window_obj) = window_val.as_object() {
                window_obj.set(js_string!("history"), global_object.get(js_string!("history"), context)?, false, context)?;
            }
        }

        Ok(())
    }
}

/// Thin wrapper matching expected API: setup_real_history(context, browser)
pub fn setup_real_history(context: &mut Context, browser: Arc<Mutex<HeadlessWebBrowser>>) -> Result<()> {
    let hist = BrowserHistory::new(browser);
    hist.setup_history_globals(context)
}