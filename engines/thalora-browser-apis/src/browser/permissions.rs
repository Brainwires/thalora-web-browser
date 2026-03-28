//! Permissions API implementation for Boa
//!
//! Implements the Permissions API as defined in:
//! https://w3c.github.io/permissions/
//!
//! This provides permission management for headless browser operation.

use boa_engine::{
    Context, JsArgs, JsNativeError, JsResult, NativeFunction,
    builtins::promise::Promise,
    js_string,
    object::{JsObject, ObjectInitializer},
    property::Attribute,
    value::JsValue,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Permission state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    Granted,
    Denied,
    Prompt,
}

impl PermissionState {
    fn as_str(&self) -> &'static str {
        match self {
            PermissionState::Granted => "granted",
            PermissionState::Denied => "denied",
            PermissionState::Prompt => "prompt",
        }
    }
}

/// Global permission states
static PERMISSION_STATES: std::sync::LazyLock<Arc<Mutex<HashMap<String, PermissionState>>>> =
    std::sync::LazyLock::new(|| {
        let mut map = HashMap::new();
        // Default permission states for headless browser
        map.insert("notifications".to_string(), PermissionState::Granted);
        map.insert("push".to_string(), PermissionState::Granted);
        map.insert("geolocation".to_string(), PermissionState::Granted);
        map.insert("camera".to_string(), PermissionState::Granted);
        map.insert("microphone".to_string(), PermissionState::Granted);
        map.insert("clipboard-read".to_string(), PermissionState::Granted);
        map.insert("clipboard-write".to_string(), PermissionState::Granted);
        map.insert("persistent-storage".to_string(), PermissionState::Granted);
        map.insert("midi".to_string(), PermissionState::Granted);
        map.insert("background-sync".to_string(), PermissionState::Granted);
        map.insert("accelerometer".to_string(), PermissionState::Granted);
        map.insert("gyroscope".to_string(), PermissionState::Granted);
        map.insert("magnetometer".to_string(), PermissionState::Granted);
        Arc::new(Mutex::new(map))
    });

/// Create a Permissions instance for navigator.permissions
pub fn create_permissions(context: &mut Context) -> JsResult<JsObject> {
    let permissions = ObjectInitializer::new(context)
        .function(
            NativeFunction::from_fn_ptr(permissions_query),
            js_string!("query"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(permissions_request),
            js_string!("request"),
            1,
        )
        .function(
            NativeFunction::from_fn_ptr(permissions_revoke),
            js_string!("revoke"),
            1,
        )
        .build();

    Ok(permissions)
}

/// Permissions.query(permissionDescriptor) - Returns a Promise that resolves with PermissionStatus
fn permissions_query(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let descriptor = args.get_or_undefined(0);

    // Get permission name from descriptor
    let name = if let Some(desc_obj) = descriptor.as_object() {
        desc_obj
            .get(js_string!("name"), context)?
            .to_string(context)?
            .to_std_string_escaped()
    } else {
        return Err(JsNativeError::typ()
            .with_message("Permissions.query: descriptor must be an object with 'name' property")
            .into());
    };

    // Get the permission state
    let state = {
        let states = PERMISSION_STATES.lock().unwrap();
        states
            .get(&name)
            .copied()
            .unwrap_or(PermissionState::Prompt)
    };

    // Create PermissionStatus object
    let permission_status = create_permission_status(&name, state, context)?;

    // Return a Promise that resolves with the PermissionStatus
    let promise_constructor = context.intrinsics().constructors().promise().constructor();
    Promise::resolve(
        &promise_constructor.into(),
        &[JsValue::from(permission_status)],
        context,
    )
}

/// Permissions.request(permissionDescriptor) - Returns a Promise that resolves with PermissionStatus
fn permissions_request(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let descriptor = args.get_or_undefined(0);

    // Get permission name from descriptor
    let name = if let Some(desc_obj) = descriptor.as_object() {
        desc_obj
            .get(js_string!("name"), context)?
            .to_string(context)?
            .to_std_string_escaped()
    } else {
        return Err(JsNativeError::typ()
            .with_message("Permissions.request: descriptor must be an object with 'name' property")
            .into());
    };

    // For headless browser, always grant permissions
    {
        let mut states = PERMISSION_STATES.lock().unwrap();
        states.insert(name.clone(), PermissionState::Granted);
    }

    // Create PermissionStatus object with granted state
    let permission_status = create_permission_status(&name, PermissionState::Granted, context)?;

    // Return a Promise that resolves with the PermissionStatus
    let promise_constructor = context.intrinsics().constructors().promise().constructor();
    Promise::resolve(
        &promise_constructor.into(),
        &[JsValue::from(permission_status)],
        context,
    )
}

/// Permissions.revoke(permissionDescriptor) - Returns a Promise that resolves with PermissionStatus
fn permissions_revoke(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let descriptor = args.get_or_undefined(0);

    // Get permission name from descriptor
    let name = if let Some(desc_obj) = descriptor.as_object() {
        desc_obj
            .get(js_string!("name"), context)?
            .to_string(context)?
            .to_std_string_escaped()
    } else {
        return Err(JsNativeError::typ()
            .with_message("Permissions.revoke: descriptor must be an object with 'name' property")
            .into());
    };

    // Set permission to prompt state
    {
        let mut states = PERMISSION_STATES.lock().unwrap();
        states.insert(name.clone(), PermissionState::Prompt);
    }

    // Create PermissionStatus object
    let permission_status = create_permission_status(&name, PermissionState::Prompt, context)?;

    // Return a Promise that resolves with the PermissionStatus
    let promise_constructor = context.intrinsics().constructors().promise().constructor();
    Promise::resolve(
        &promise_constructor.into(),
        &[JsValue::from(permission_status)],
        context,
    )
}

/// Create a PermissionStatus object
fn create_permission_status(
    name: &str,
    state: PermissionState,
    context: &mut Context,
) -> JsResult<JsObject> {
    let status = ObjectInitializer::new(context)
        .property(
            js_string!("name"),
            js_string!(name),
            Attribute::READONLY | Attribute::ENUMERABLE,
        )
        .property(
            js_string!("state"),
            js_string!(state.as_str()),
            Attribute::READONLY | Attribute::ENUMERABLE,
        )
        // Event handlers
        .property(
            js_string!("onchange"),
            JsValue::null(),
            Attribute::WRITABLE | Attribute::CONFIGURABLE,
        )
        .build();

    Ok(status)
}

/// Set a permission state programmatically (for testing)
pub fn set_permission_state(name: &str, state: PermissionState) {
    let mut states = PERMISSION_STATES.lock().unwrap();
    states.insert(name.to_string(), state);
}

/// Get a permission state
pub fn get_permission_state(name: &str) -> PermissionState {
    let states = PERMISSION_STATES.lock().unwrap();
    states.get(name).copied().unwrap_or(PermissionState::Prompt)
}
