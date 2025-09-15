use anyhow::Result;
use boa_engine::{Context, JsObject, JsValue, NativeFunction, js_string};

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
        let hid_request_device_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
            Err(boa_engine::JsError::from_opaque(JsValue::from(js_string!("HID access requires user gesture and secure context"))))
        }) };

        let hid_get_devices_fn = unsafe { NativeFunction::from_closure(|_, _args, context| {
            let empty_array = boa_engine::object::builtins::JsArray::new(context);
            Ok(JsValue::from(empty_array))
        }) };

        hid.set(js_string!("requestDevice"), JsValue::from(hid_request_device_fn.to_js_function(context.realm())), false, context)?;
        hid.set(js_string!("getDevices"), JsValue::from(hid_get_devices_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("hid"), JsValue::from(hid), false, context)?;

        // USB API
        let usb = JsObject::with_object_proto(context.intrinsics());
        let usb_request_device_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
            Err(boa_engine::JsError::from_opaque(JsValue::from(js_string!("USB access requires user gesture and secure context"))))
        }) };

        let usb_get_devices_fn = unsafe { NativeFunction::from_closure(|_, _args, context| {
            let empty_array = boa_engine::object::builtins::JsArray::new(context);
            Ok(JsValue::from(empty_array))
        }) };

        usb.set(js_string!("requestDevice"), JsValue::from(usb_request_device_fn.to_js_function(context.realm())), false, context)?;
        usb.set(js_string!("getDevices"), JsValue::from(usb_get_devices_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("usb"), JsValue::from(usb), false, context)?;

        // Serial API
        let serial = JsObject::with_object_proto(context.intrinsics());
        let serial_request_port_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
            Err(boa_engine::JsError::from_opaque(JsValue::from(js_string!("Serial access requires user gesture and secure context"))))
        }) };

        let serial_get_ports_fn = unsafe { NativeFunction::from_closure(|_, _args, context| {
            let empty_array = boa_engine::object::builtins::JsArray::new(context);
            Ok(JsValue::from(empty_array))
        }) };

        serial.set(js_string!("requestPort"), JsValue::from(serial_request_port_fn.to_js_function(context.realm())), false, context)?;
        serial.set(js_string!("getPorts"), JsValue::from(serial_get_ports_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("serial"), JsValue::from(serial), false, context)?;

        // Bluetooth API
        let bluetooth = JsObject::with_object_proto(context.intrinsics());
        let bluetooth_request_device_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
            Err(boa_engine::JsError::from_opaque(JsValue::from(js_string!("Bluetooth access requires user gesture and secure context"))))
        }) };

        let bluetooth_get_availability_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
            Ok(JsValue::from(false))
        }) };

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
        let query_fn = unsafe { NativeFunction::from_closure(|_, _args, context| {
            // Return permission status object directly (simplified)
            let result = JsObject::with_object_proto(context.intrinsics());
            result.set(js_string!("state"), JsValue::from(js_string!("granted")), false, context)?;
            result.set(js_string!("name"), JsValue::from(js_string!("geolocation")), false, context)?;
            Ok(JsValue::from(result))
        }) };

        permissions.set(js_string!("query"), JsValue::from(query_fn.to_js_function(context.realm())), false, context)?;
        navigator.set(js_string!("permissions"), JsValue::from(permissions), false, context)?;

        // Navigator getUserMedia (deprecated but still used)
        let get_user_media_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
            // This is deprecated in favor of navigator.mediaDevices.getUserMedia
            Err(boa_engine::JsError::from_opaque(JsValue::from(js_string!("navigator.getUserMedia is deprecated. Use navigator.mediaDevices.getUserMedia instead."))))
        }) };

        navigator.set(js_string!("getUserMedia"), JsValue::from(get_user_media_fn.to_js_function(context.realm())), false, context)?;

        // Clipboard API
        let clipboard = JsObject::with_object_proto(context.intrinsics());

        let write_text_fn = unsafe { NativeFunction::from_closure(|_, args, _context| {
            if args.is_empty() {
                return Err(boa_engine::JsError::from_opaque(JsValue::from(js_string!("writeText requires a text parameter"))));
            }

            // In a real implementation, this would write to the system clipboard
            // For headless mode, we just return successfully
            Ok(JsValue::undefined())
        }) };

        let read_text_fn = unsafe { NativeFunction::from_closure(|_, _args, _context| {
            // In headless mode, return empty string
            Ok(JsValue::from(js_string!("")))
        }) };

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

        // Set navigator in global scope
        let global = context.global_object();
        global.set(js_string!("navigator"), JsValue::from(navigator), false, context)?;

        Ok(())
    }
}