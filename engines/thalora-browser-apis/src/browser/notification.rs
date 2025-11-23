//! Notification API implementation for Boa
//!
//! Implements the Notifications API as defined in:
//! https://notifications.spec.whatwg.org/
//!
//! This provides notifications support for headless browser operation.

use boa_engine::{
    builtins::promise::Promise,
    object::{JsObject, ObjectInitializer},
    property::Attribute,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string, NativeFunction,
};
use boa_gc::{Finalize, Trace, GcRefMut};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Notification permission state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationPermission {
    Default,
    Granted,
    Denied,
}

impl NotificationPermission {
    fn as_str(&self) -> &'static str {
        match self {
            NotificationPermission::Default => "default",
            NotificationPermission::Granted => "granted",
            NotificationPermission::Denied => "denied",
        }
    }
}

/// Global notification permission state
static NOTIFICATION_PERMISSION: std::sync::LazyLock<Arc<Mutex<NotificationPermission>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(NotificationPermission::Granted)));

/// Storage for active notifications
static ACTIVE_NOTIFICATIONS: std::sync::LazyLock<Arc<Mutex<HashMap<u64, NotificationData>>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Counter for notification IDs
static NOTIFICATION_ID_COUNTER: std::sync::LazyLock<Arc<Mutex<u64>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(0)));

/// Internal notification data
#[derive(Debug, Clone)]
struct NotificationData {
    id: u64,
    title: String,
    body: String,
    icon: String,
    tag: String,
    require_interaction: bool,
    silent: bool,
    timestamp: u64,
}

/// Create the Notification constructor
pub fn create_notification_constructor(context: &mut Context) -> JsResult<JsObject> {
    // Create prototype with methods
    let prototype = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(notification_close),
            js_string!("close"),
            0,
        )
        .build();

    // Create constructor function
    let constructor = NativeFunction::from_copy_closure(move |_this, args, context| {
        // Get title (required)
        let title = args.get_or_undefined(0);
        if title.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Notification constructor: At least 1 argument required")
                .into());
        }
        let title_str = title.to_string(context)?.to_std_string_escaped();

        // Get options (optional)
        let options = args.get_or_undefined(1);

        // Parse options
        let mut body = String::new();
        let mut icon = String::new();
        let mut tag = String::new();
        let mut require_interaction = false;
        let mut silent = false;

        if let Some(opts_obj) = options.as_object() {
            if let Ok(body_val) = opts_obj.get(js_string!("body"), context) {
                if !body_val.is_undefined() {
                    body = body_val.to_string(context)?.to_std_string_escaped();
                }
            }
            if let Ok(icon_val) = opts_obj.get(js_string!("icon"), context) {
                if !icon_val.is_undefined() {
                    icon = icon_val.to_string(context)?.to_std_string_escaped();
                }
            }
            if let Ok(tag_val) = opts_obj.get(js_string!("tag"), context) {
                if !tag_val.is_undefined() {
                    tag = tag_val.to_string(context)?.to_std_string_escaped();
                }
            }
            if let Ok(ri_val) = opts_obj.get(js_string!("requireInteraction"), context) {
                require_interaction = ri_val.to_boolean();
            }
            if let Ok(silent_val) = opts_obj.get(js_string!("silent"), context) {
                silent = silent_val.to_boolean();
            }
        }

        // Generate notification ID
        let id = {
            let mut counter = NOTIFICATION_ID_COUNTER.lock().unwrap();
            *counter += 1;
            *counter
        };

        // Get current timestamp
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);

        // Store notification data
        let notification_data = NotificationData {
            id,
            title: title_str.clone(),
            body: body.clone(),
            icon: icon.clone(),
            tag: tag.clone(),
            require_interaction,
            silent,
            timestamp,
        };

        {
            let mut notifications = ACTIVE_NOTIFICATIONS.lock().unwrap();
            notifications.insert(id, notification_data);
        }

        // Create notification object
        let notification = ObjectInitializer::new(context)
            .property(js_string!("title"), js_string!(title_str), Attribute::READONLY | Attribute::ENUMERABLE)
            .property(js_string!("body"), js_string!(body), Attribute::READONLY | Attribute::ENUMERABLE)
            .property(js_string!("icon"), js_string!(icon), Attribute::READONLY | Attribute::ENUMERABLE)
            .property(js_string!("tag"), js_string!(tag), Attribute::READONLY | Attribute::ENUMERABLE)
            .property(js_string!("requireInteraction"), require_interaction, Attribute::READONLY | Attribute::ENUMERABLE)
            .property(js_string!("silent"), silent, Attribute::READONLY | Attribute::ENUMERABLE)
            .property(js_string!("timestamp"), timestamp as f64, Attribute::READONLY | Attribute::ENUMERABLE)
            .property(js_string!("_id"), id as f64, Attribute::all())
            // Event handlers (initially null)
            .property(js_string!("onclick"), JsValue::null(), Attribute::WRITABLE | Attribute::CONFIGURABLE)
            .property(js_string!("onclose"), JsValue::null(), Attribute::WRITABLE | Attribute::CONFIGURABLE)
            .property(js_string!("onerror"), JsValue::null(), Attribute::WRITABLE | Attribute::CONFIGURABLE)
            .property(js_string!("onshow"), JsValue::null(), Attribute::WRITABLE | Attribute::CONFIGURABLE)
            .function(
                NativeFunction::from_fn_ptr(notification_close),
                js_string!("close"),
                0,
            )
            .build();

        Ok(JsValue::from(notification))
    });

    let constructor_obj: JsObject = constructor.to_js_function(context.realm()).into();

    // Add static properties
    constructor_obj.set(js_string!("permission"), js_string!("granted"), false, context)?;

    // Add requestPermission static method
    let request_permission = NativeFunction::from_fn_ptr(notification_request_permission);
    constructor_obj.set(
        js_string!("requestPermission"),
        request_permission.to_js_function(context.realm()),
        false,
        context,
    )?;

    // Add prototype
    constructor_obj.set(js_string!("prototype"), prototype, false, context)?;

    Ok(constructor_obj)
}

/// Notification.requestPermission() - Returns a Promise that resolves with the permission
fn notification_request_permission(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // For headless browser, always return "granted"
    let permission = {
        let perm = NOTIFICATION_PERMISSION.lock().unwrap();
        perm.as_str().to_string()
    };

    let promise_constructor = context.intrinsics().constructors().promise().constructor();
    Promise::resolve(
        &promise_constructor.into(),
        &[JsValue::from(js_string!(permission))],
        context,
    )
}

/// Notification.prototype.close() - Closes the notification
fn notification_close(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Notification.close called on non-object")
    })?;

    // Get notification ID
    if let Ok(id_val) = this_obj.get(js_string!("_id"), context) {
        if let Some(id) = id_val.as_number() {
            let id = id as u64;

            // Remove from active notifications
            {
                let mut notifications = ACTIVE_NOTIFICATIONS.lock().unwrap();
                notifications.remove(&id);
            }

            // Call onclose handler if set
            if let Ok(onclose) = this_obj.get(js_string!("onclose"), context) {
                if let Some(func) = onclose.as_callable() {
                    // Create a simple event object
                    let event = ObjectInitializer::new(context)
                        .property(js_string!("type"), js_string!("close"), Attribute::all())
                        .property(js_string!("target"), this.clone(), Attribute::all())
                        .build();

                    let _ = func.call(this, &[JsValue::from(event)], context);
                }
            }
        }
    }

    Ok(JsValue::undefined())
}

/// Set the notification permission (for testing)
pub fn set_notification_permission(permission: NotificationPermission) {
    let mut perm = NOTIFICATION_PERMISSION.lock().unwrap();
    *perm = permission;
}

/// Get the current notification permission
pub fn get_notification_permission() -> NotificationPermission {
    let perm = NOTIFICATION_PERMISSION.lock().unwrap();
    *perm
}

/// Get the count of active notifications
pub fn get_active_notification_count() -> usize {
    let notifications = ACTIVE_NOTIFICATIONS.lock().unwrap();
    notifications.len()
}
