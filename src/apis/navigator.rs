use anyhow::Result;
use boa_engine::{Context, JsObject, JsValue, NativeFunction, js_string};
use boa_engine::object::builtins::JsArray;

/// Navigator API implementation with Chrome-compatible browser information
pub struct NavigatorManager {
    user_agent: String,
    platform: String,
    vendor: String,
    language: String,
    languages: Vec<String>,
    hardware_concurrency: u32,
    device_memory: u32,
    max_touch_points: u32,
}

impl NavigatorManager {
    pub fn new() -> Self {
        Self {
            user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36".to_string(),
            platform: "MacIntel".to_string(),
            vendor: "Google Inc.".to_string(),
            language: "en-US".to_string(),
            languages: vec!["en-US".to_string(), "en".to_string()],
            hardware_concurrency: 8,
            device_memory: 8,
            max_touch_points: 0,
        }
    }

    /// Setup navigator object with Chrome-compatible properties
    pub fn setup_navigator_api(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        let navigator = JsObject::with_object_proto(context.intrinsics());

        // Basic navigator properties
        navigator.set(js_string!("userAgent"), JsValue::from(js_string!(self.user_agent.clone())), false, context)?;
        navigator.set(js_string!("platform"), JsValue::from(js_string!(self.platform.clone())), false, context)?;
        navigator.set(js_string!("vendor"), JsValue::from(js_string!(self.vendor.clone())), false, context)?;
        navigator.set(js_string!("language"), JsValue::from(js_string!(self.language.clone())), false, context)?;

        // Languages array
        let languages_array = boa_engine::object::builtins::JsArray::new(context);
        for (i, lang) in self.languages.iter().enumerate() {
            languages_array.set(i, JsValue::from(js_string!(lang.clone())), false, context)?;
        }
        navigator.set(js_string!("languages"), JsValue::from(languages_array), false, context)?;

        // Hardware information
        navigator.set(js_string!("hardwareConcurrency"), JsValue::from(self.hardware_concurrency), false, context)?;
        navigator.set(js_string!("deviceMemory"), JsValue::from(self.device_memory), false, context)?;
        navigator.set(js_string!("maxTouchPoints"), JsValue::from(self.max_touch_points), false, context)?;

        // Browser capabilities
        navigator.set(js_string!("cookieEnabled"), JsValue::from(true), false, context)?;
        navigator.set(js_string!("onLine"), JsValue::from(true), false, context)?;
        navigator.set(js_string!("doNotTrack"), JsValue::null(), false, context)?;

        // Chrome-specific properties
        navigator.set(js_string!("webdriver"), JsValue::from(false), false, context)?;
        navigator.set(js_string!("pdfViewerEnabled"), JsValue::from(true), false, context)?;

        // Chrome 131+ features - Device APIs that typically require user gestures
        // WebHID API (chrome 131+)
        let hid = JsObject::with_object_proto(context.intrinsics());
        let hid_request_device_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            Err(boa_engine::JsError::from_opaque(JsValue::from(js_string!("HID access requires user gesture and secure context"))))
        });

        let hid_get_devices_fn = NativeFunction::from_fn_ptr(|_, _args, context| {
            let empty_array = boa_engine::object::builtins::JsArray::new(context);
            Ok(JsValue::from(empty_array))
        });

        hid.set(js_string!("requestDevice"), JsValue::from(hid_request_device_fn.to_js_function(context.realm())), false, context)?;
        hid.set(js_string!("getDevices"), JsValue::from(hid_get_devices_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("hid"), JsValue::from(hid), false, context)?;

        // USB API
        let usb = JsObject::with_object_proto(context.intrinsics());
        let usb_request_device_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            Err(boa_engine::JsError::from_opaque(JsValue::from(js_string!("USB access requires user gesture and secure context"))))
        });

        let usb_get_devices_fn = NativeFunction::from_fn_ptr(|_, _args, context| {
            let empty_array = boa_engine::object::builtins::JsArray::new(context);
            Ok(JsValue::from(empty_array))
        });

        usb.set(js_string!("requestDevice"), JsValue::from(usb_request_device_fn.to_js_function(context.realm())), false, context)?;
        usb.set(js_string!("getDevices"), JsValue::from(usb_get_devices_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("usb"), JsValue::from(usb), false, context)?;

        // Serial API
        let serial = JsObject::with_object_proto(context.intrinsics());
        let serial_request_port_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            Err(boa_engine::JsError::from_opaque(JsValue::from(js_string!("Serial access requires user gesture and secure context"))))
        });

        let serial_get_ports_fn = NativeFunction::from_fn_ptr(|_, _args, context| {
            let empty_array = boa_engine::object::builtins::JsArray::new(context);
            Ok(JsValue::from(empty_array))
        });

        serial.set(js_string!("requestPort"), JsValue::from(serial_request_port_fn.to_js_function(context.realm())), false, context)?;
        serial.set(js_string!("getPorts"), JsValue::from(serial_get_ports_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("serial"), JsValue::from(serial), false, context)?;

        // Bluetooth API
        let bluetooth = JsObject::with_object_proto(context.intrinsics());
        let bluetooth_request_device_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            Err(boa_engine::JsError::from_opaque(JsValue::from(js_string!("Bluetooth access requires user gesture and secure context"))))
        });

        let bluetooth_get_availability_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            Ok(JsValue::from(false))
        });

        bluetooth.set(js_string!("requestDevice"), JsValue::from(bluetooth_request_device_fn.to_js_function(context.realm())), false, context)?;
        bluetooth.set(js_string!("getAvailability"), JsValue::from(bluetooth_get_availability_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("bluetooth"), JsValue::from(bluetooth), false, context)?;

        // App information (legacy but still checked)
        navigator.set(js_string!("appName"), JsValue::from(js_string!("Netscape")), false, context)?;
        navigator.set(js_string!("appVersion"), JsValue::from(js_string!("5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")), false, context)?;
        navigator.set(js_string!("appCodeName"), JsValue::from(js_string!("Mozilla")), false, context)?;
        navigator.set(js_string!("product"), JsValue::from(js_string!("Gecko")), false, context)?;
        navigator.set(js_string!("productSub"), JsValue::from(js_string!("20030107")), false, context)?;
        navigator.set(js_string!("vendorSub"), JsValue::from(js_string!("")), false, context)?;

        // Chrome permissions API
        let permissions = JsObject::with_object_proto(context.intrinsics());
        let query_fn = NativeFunction::from_fn_ptr(|_, _args, context| {
            // Return permission status object directly (simplified)
            let result = JsObject::with_object_proto(context.intrinsics());
            result.set(js_string!("state"), JsValue::from(js_string!("granted")), false, context)?;
            result.set(js_string!("name"), JsValue::from(js_string!("geolocation")), false, context)?;
            Ok(JsValue::from(result))
        });

        permissions.set(js_string!("query"), JsValue::from(query_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("permissions"), JsValue::from(permissions), false, context)?;

        // Navigator getUserMedia (deprecated but still used)
        let get_user_media_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // This is deprecated in favor of navigator.mediaDevices.getUserMedia
            Err(boa_engine::JsError::from_opaque(JsValue::from(js_string!("navigator.getUserMedia is deprecated. Use navigator.mediaDevices.getUserMedia instead."))))
        });

        navigator.set(js_string!("getUserMedia"), JsValue::from(get_user_media_fn.to_js_function(context.realm())), false, context)?;

        // Clipboard API
        let clipboard = JsObject::with_object_proto(context.intrinsics());

        let write_text_fn = NativeFunction::from_fn_ptr(|_, args, _context| {
            if args.is_empty() {
                return Err(boa_engine::JsError::from_opaque(JsValue::from(js_string!("writeText requires a text parameter"))));
            }

            // In a real implementation, this would write to the system clipboard
            // For headless mode, we just return successfully
            Ok(JsValue::undefined())
        });

        let read_text_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // In headless mode, return empty string
            Ok(JsValue::from(js_string!("")))
        });

        clipboard.set(js_string!("writeText"), JsValue::from(write_text_fn.to_js_function(context.realm())), false, context)?;
        clipboard.set(js_string!("readText"), JsValue::from(read_text_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("clipboard"), JsValue::from(clipboard), false, context)?;

        // Connection API
        let connection = JsObject::with_object_proto(context.intrinsics());
        connection.set(js_string!("effectiveType"), JsValue::from(js_string!("4g")), false, context)?;
        connection.set(js_string!("downlink"), JsValue::from(10.0), false, context)?;
        connection.set(js_string!("rtt"), JsValue::from(50), false, context)?;
        connection.set(js_string!("saveData"), JsValue::from(false), false, context)?;
        navigator.set(js_string!("connection"), JsValue::from(connection), false, context)?;

        // WebMIDI API (Chrome 124 - now requires permissions)
        let request_midi_access_fn = NativeFunction::from_fn_ptr(|_, _args, context| {
            // Create a mock MIDI access promise that resolves with empty ports
            let midi_access = JsObject::with_object_proto(context.intrinsics());
            midi_access.set(js_string!("sysexEnabled"), JsValue::from(false), false, context)?;

            let empty_map = JsObject::with_object_proto(context.intrinsics());
            empty_map.set(js_string!("size"), JsValue::from(0), false, context)?;
            midi_access.set(js_string!("inputs"), JsValue::from(empty_map.clone()), false, context)?;
            midi_access.set(js_string!("outputs"), JsValue::from(empty_map), false, context)?;

            // Return a resolved promise with the MIDI access object
            let resolved_promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::from(midi_access), context);
            Ok(JsValue::from(resolved_promise))
        });
        navigator.set(js_string!("requestMIDIAccess"), JsValue::from(request_midi_access_fn.to_js_function(context.realm())), false, context)?;

        // WebGPU API (Chrome 124 enhancements)
        let gpu = JsObject::with_object_proto(context.intrinsics());

        let request_adapter_fn = NativeFunction::from_fn_ptr(|_, _args, context| {
            // Return null adapter (WebGPU not available in headless mode)
            let resolved_promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::null(), context);
            Ok(JsValue::from(resolved_promise))
        });

        let get_preferred_canvas_format_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            Ok(JsValue::from(js_string!("rgba8unorm")))
        });

        gpu.set(js_string!("requestAdapter"), JsValue::from(request_adapter_fn.to_js_function(context.realm())), false, context)?;
        gpu.set(js_string!("getPreferredCanvasFormat"), JsValue::from(get_preferred_canvas_format_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("gpu"), JsValue::from(gpu), false, context)?;

        // User Agent Client Hints (Chrome 124 - Sec-CH-UA-Form-Factors)
        let user_agent_data = JsObject::with_object_proto(context.intrinsics());

        // Basic brand info
        let brands_array = boa_engine::object::builtins::JsArray::new(context);
        let brand = JsObject::with_object_proto(context.intrinsics());
        brand.set(js_string!("brand"), JsValue::from(js_string!("Chromium")), false, context)?;
        brand.set(js_string!("version"), JsValue::from(js_string!("131")), false, context)?;
        brands_array.set(0, JsValue::from(brand), false, context)?;
        user_agent_data.set(js_string!("brands"), JsValue::from(brands_array), false, context)?;
        user_agent_data.set(js_string!("mobile"), JsValue::from(false), false, context)?;
        user_agent_data.set(js_string!("platform"), JsValue::from(js_string!(self.platform.clone())), false, context)?;

        let get_high_entropy_values_fn = NativeFunction::from_fn_ptr(|_, args, context| {
            // Mock implementation - in real browser this would return actual system info
            let hints = JsObject::with_object_proto(context.intrinsics());
            hints.set(js_string!("platform"), JsValue::from(js_string!("macOS")), false, context)?;
            hints.set(js_string!("platformVersion"), JsValue::from(js_string!("13.0.0")), false, context)?;
            hints.set(js_string!("architecture"), JsValue::from(js_string!("x86")), false, context)?;
            hints.set(js_string!("model"), JsValue::from(js_string!("")), false, context)?;
            hints.set(js_string!("uaFullVersion"), JsValue::from(js_string!("131.0.6778.86")), false, context)?;

            // Chrome 124 feature: Form factors
            let form_factors_array = boa_engine::object::builtins::JsArray::new(context);
            form_factors_array.set(0, JsValue::from(js_string!("desktop")), false, context)?;
            hints.set(js_string!("formFactors"), JsValue::from(form_factors_array), false, context)?;

            let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::from(hints), context);
            Ok(JsValue::from(promise))
        });

        user_agent_data.set(js_string!("getHighEntropyValues"), JsValue::from(get_high_entropy_values_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("userAgentData"), JsValue::from(user_agent_data), false, context)?;

        // Direct Sockets API (Chrome 125) - TCP and UDP
        let tcp = JsObject::with_object_proto(context.intrinsics());
        let tcp_connect_fn = NativeFunction::from_fn_ptr(|_, _args, context| {
            // TCP sockets require Chrome Apps context - not available in headless mode
            let error = boa_engine::JsError::from_opaque(JsValue::from(js_string!("TCP sockets are only available in Chrome Apps")));
            Err(error)
        });
        tcp.set(js_string!("connect"), JsValue::from(tcp_connect_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("tcp"), JsValue::from(tcp), false, context)?;

        let udp = JsObject::with_object_proto(context.intrinsics());
        let udp_bind_fn = NativeFunction::from_fn_ptr(|_, _args, context| {
            // UDP sockets require Chrome Apps context - not available in headless mode
            let error = boa_engine::JsError::from_opaque(JsValue::from(js_string!("UDP sockets are only available in Chrome Apps")));
            Err(error)
        });
        udp.set(js_string!("bind"), JsValue::from(udp_bind_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("udp"), JsValue::from(udp), false, context)?;

        // Storage Access API (Chrome 125) - also add to navigator for testing
        let request_storage_access_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // In headless mode, assume storage access is granted
            let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::undefined(), _context);
            Ok(JsValue::from(promise))
        });

        let has_storage_access_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // In headless mode, assume storage access is available
            let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::from(true), _context);
            Ok(JsValue::from(promise))
        });

        navigator.set(js_string!("requestStorageAccess"), JsValue::from(request_storage_access_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("hasStorageAccess"), JsValue::from(has_storage_access_fn.to_js_function(context.realm())), false, context)?;

        // Also try to set on document if it exists
        let global = context.global_object();
        if let Ok(document_val) = global.get(js_string!("document"), context) {
            if let Some(document_obj) = document_val.as_object() {
                // Create separate functions for document
                let doc_request_storage_access_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
                    let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::undefined(), _context);
                    Ok(JsValue::from(promise))
                });

                let doc_has_storage_access_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
                    let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::from(true), _context);
                    Ok(JsValue::from(promise))
                });

                document_obj.set(js_string!("requestStorageAccess"), JsValue::from(doc_request_storage_access_fn.to_js_function(context.realm())), false, context)?;
                document_obj.set(js_string!("hasStorageAccess"), JsValue::from(doc_has_storage_access_fn.to_js_function(context.realm())), false, context)?;
            }
        }

        // Chrome 126: Gamepad API
        let get_gamepads_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // Return empty array - no gamepads connected in headless mode
            let gamepads_array = boa_engine::object::builtins::JsArray::new(_context);
            Ok(JsValue::from(gamepads_array))
        });
        navigator.set(js_string!("getGamepads"), JsValue::from(get_gamepads_fn.to_js_function(context.realm())), false, context)?;

        // Chrome 127: User Activation API
        let user_activation = JsObject::with_object_proto(context.intrinsics());
        user_activation.set(js_string!("hasBeenActive"), JsValue::from(true), false, context)?;
        user_activation.set(js_string!("isActive"), JsValue::from(true), false, context)?;
        navigator.set(js_string!("userActivation"), JsValue::from(user_activation), false, context)?;

        // Chrome 128: WebAuthn Credentials API
        let credentials = JsObject::with_object_proto(context.intrinsics());

        let create_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // Mock WebAuthn create implementation
            let error = boa_engine::JsError::from_opaque(JsValue::from(js_string!("WebAuthn not supported in headless mode")));
            Err(error)
        });
        credentials.set(js_string!("create"), JsValue::from(create_fn.to_js_function(context.realm())), false, context)?;

        let get_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // Mock WebAuthn get implementation
            let error = boa_engine::JsError::from_opaque(JsValue::from(js_string!("WebAuthn not supported in headless mode")));
            Err(error)
        });
        credentials.set(js_string!("get"), JsValue::from(get_fn.to_js_function(context.realm())), false, context)?;

        navigator.set(js_string!("credentials"), JsValue::from(credentials), false, context)?;

        // Chrome 128: Media Session API
        let media_session = JsObject::with_object_proto(context.intrinsics());

        let set_action_handler_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // Mock setActionHandler implementation - accepts skipad action
            Ok(JsValue::undefined())
        });
        media_session.set(js_string!("setActionHandler"), JsValue::from(set_action_handler_fn.to_js_function(context.realm())), false, context)?;

        let set_metadata_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // Mock setMetadata implementation
            Ok(JsValue::undefined())
        });
        media_session.set(js_string!("setMetadata"), JsValue::from(set_metadata_fn.to_js_function(context.realm())), false, context)?;

        let set_playback_state_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // Mock setPlaybackState implementation
            Ok(JsValue::undefined())
        });
        media_session.set(js_string!("setPlaybackState"), JsValue::from(set_playback_state_fn.to_js_function(context.realm())), false, context)?;

        navigator.set(js_string!("mediaSession"), JsValue::from(media_session), false, context)?;

        // Chrome 130: Machine Learning API for Language Detection
        let ml_api = JsObject::with_object_proto(context.intrinsics());

        let create_language_detector_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // Create a mock language detector
            let detector = JsObject::with_object_proto(_context.intrinsics());

            let detect_fn = NativeFunction::from_fn_ptr(|_, _args, _ctx| {
                // Mock language detection returning English with high confidence
                let result_array = boa_engine::object::builtins::JsArray::new(_ctx);
                let detection = JsObject::with_object_proto(_ctx.intrinsics());
                detection.set(js_string!("language"), JsValue::from(js_string!("en")), false, _ctx)?;
                detection.set(js_string!("confidence"), JsValue::from(0.95), false, _ctx)?;
                result_array.push(JsValue::from(detection), _ctx)?;

                let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::from(result_array), _ctx);
                Ok(JsValue::from(promise))
            });
            detector.set(js_string!("detect"), JsValue::from(detect_fn.to_js_function(_context.realm())), false, _context)?;

            let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::from(detector), _context);
            Ok(JsValue::from(promise))
        });
        ml_api.set(js_string!("createLanguageDetector"), JsValue::from(create_language_detector_fn.to_js_function(context.realm())), false, context)?;

        // Chrome 131: Translator API (Origin Trial)
        let create_translator_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // Create a mock translator
            let translator = JsObject::with_object_proto(_context.intrinsics());

            let translate_fn = NativeFunction::from_fn_ptr(|_, args, ctx| {
                // Mock translation - just return the input text
                let text = if args.len() > 0 {
                    args[0].to_string(ctx).unwrap_or_else(|_| js_string!("")).to_std_string_escaped()
                } else {
                    "".to_string()
                };

                let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::from(js_string!(format!("Translated: {}", text))), ctx);
                Ok(JsValue::from(promise))
            });
            translator.set(js_string!("translate"), JsValue::from(translate_fn.to_js_function(_context.realm())), false, _context)?;

            let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::from(translator), _context);
            Ok(JsValue::from(promise))
        });
        ml_api.set(js_string!("createTranslator"), JsValue::from(create_translator_fn.to_js_function(context.realm())), false, context)?;

        // Chrome 131: Summarizer API (Origin Trial)
        let create_summarizer_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // Create a mock summarizer
            let summarizer = JsObject::with_object_proto(_context.intrinsics());

            let summarize_fn = NativeFunction::from_fn_ptr(|_, args, ctx| {
                // Mock summarization - return first 100 chars + "..."
                let text = if args.len() > 0 {
                    args[0].to_string(ctx).unwrap_or_else(|_| js_string!("")).to_std_string_escaped()
                } else {
                    "".to_string()
                };

                let summary = if text.len() > 100 {
                    format!("{}...", &text[..100])
                } else {
                    format!("Summary: {}", text)
                };

                let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::from(js_string!(summary)), ctx);
                Ok(JsValue::from(promise))
            });
            summarizer.set(js_string!("summarize"), JsValue::from(summarize_fn.to_js_function(_context.realm())), false, _context)?;

            let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::from(summarizer), _context);
            Ok(JsValue::from(promise))
        });
        ml_api.set(js_string!("createSummarizer"), JsValue::from(create_summarizer_fn.to_js_function(context.realm())), false, context)?;

        navigator.set(js_string!("ml"), JsValue::from(ml_api), false, context)?;

        // Chrome 131: WebXR API
        let xr_api = JsObject::with_object_proto(context.intrinsics());

        let is_session_supported_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // Mock session support check - always resolve to false for headless mode
            let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::from(false), _context);
            Ok(JsValue::from(promise))
        });
        xr_api.set(js_string!("isSessionSupported"), JsValue::from(is_session_supported_fn.to_js_function(context.realm())), false, context)?;

        let request_session_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            // Mock request session - reject for headless mode
            let error = boa_engine::JsError::from_opaque(JsValue::from(js_string!("WebXR not supported in headless mode")));
            let promise = boa_engine::object::builtins::JsPromise::reject(error, _context);
            Ok(JsValue::from(promise))
        });
        xr_api.set(js_string!("requestSession"), JsValue::from(request_session_fn.to_js_function(context.realm())), false, context)?;

        navigator.set(js_string!("xr"), JsValue::from(xr_api), false, context)?;

        // Chrome 134: Web Locks API
        let locks_api = JsObject::with_object_proto(context.intrinsics());

        let request_fn = NativeFunction::from_fn_ptr(|_, args, _context| {
            // Mock Web Locks API implementation
            let _name = args.get(0).cloned().unwrap_or(JsValue::undefined());
            let _options_or_callback = args.get(1).cloned().unwrap_or(JsValue::undefined());
            let _callback = args.get(2).cloned().unwrap_or(JsValue::undefined());

            // Return a resolved promise
            let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::undefined(), _context);
            Ok(JsValue::from(promise))
        });
        locks_api.set(js_string!("request"), JsValue::from(request_fn.to_js_function(context.realm())), false, context)?;

        let query_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            let result = JsObject::with_object_proto(_context.intrinsics());
            result.set(js_string!("pending"), JsValue::from(JsArray::new(_context)), false, _context)?;
            result.set(js_string!("held"), JsValue::from(JsArray::new(_context)), false, _context)?;

            let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::from(result), _context);
            Ok(JsValue::from(promise))
        });
        locks_api.set(js_string!("query"), JsValue::from(query_fn.to_js_function(context.realm())), false, context)?;

        navigator.set(js_string!("locks"), JsValue::from(locks_api), false, context)?;

        // Chrome 132: Device Posture API
        let device_posture = JsObject::with_object_proto(context.intrinsics());
        device_posture.set(js_string!("type"), JsValue::from(js_string!("continuous")), false, context)?;
        navigator.set(js_string!("devicePosture"), JsValue::from(device_posture), false, context)?;

        // Set navigator in global scope
        let global = context.global_object();
        global.set(js_string!("navigator"), JsValue::from(navigator), false, context)?;

        // Chrome 126: GamepadHapticActuator constructor
        let gamepad_haptic_actuator_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            let actuator = JsObject::with_object_proto(_context.intrinsics());

            actuator.set(js_string!("type"), JsValue::from(js_string!("dual-rumble")), false, _context)?;

            let can_play_fn = NativeFunction::from_fn_ptr(|_, _args, _ctx| {
                Ok(JsValue::from(true))
            });
            actuator.set(js_string!("canPlay"), JsValue::from(can_play_fn.to_js_function(_context.realm())), false, _context)?;

            let play_effect_fn = NativeFunction::from_fn_ptr(|_, _args, _ctx| {
                let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::undefined(), _ctx);
                Ok(JsValue::from(promise))
            });
            actuator.set(js_string!("playEffect"), JsValue::from(play_effect_fn.to_js_function(_context.realm())), false, _context)?;

            let reset_fn = NativeFunction::from_fn_ptr(|_, _args, _ctx| {
                let promise = boa_engine::object::builtins::JsPromise::resolve(JsValue::undefined(), _ctx);
                Ok(JsValue::from(promise))
            });
            actuator.set(js_string!("reset"), JsValue::from(reset_fn.to_js_function(_context.realm())), false, _context)?;

            Ok(JsValue::from(actuator))
        });
        global.set(js_string!("GamepadHapticActuator"), JsValue::from(gamepad_haptic_actuator_fn.to_js_function(context.realm())), false, context)?;

        // Chrome 126: WebGLObject constructor
        let webgl_object_fn = NativeFunction::from_fn_ptr(|_, _args, _context| {
            let webgl_obj = JsObject::with_object_proto(_context.intrinsics());
            Ok(JsValue::from(webgl_obj))
        });
        global.set(js_string!("WebGLObject"), JsValue::from(webgl_object_fn.to_js_function(context.realm())), false, context)?;

        Ok(())
    }
}