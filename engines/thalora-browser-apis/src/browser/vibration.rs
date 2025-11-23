//! Vibration API implementation for Boa
//!
//! Implements the Vibration API as defined in:
//! https://w3c.github.io/vibration/
//!
//! This provides vibration support for headless browser operation.
//! In headless mode, vibration requests are recorded but no actual vibration occurs.

use boa_engine::{
    value::JsValue,
    Context, JsArgs, JsResult, js_string,
};
use std::sync::{Arc, Mutex};

/// Global vibration state for testing/inspection
static VIBRATION_STATE: std::sync::LazyLock<Arc<Mutex<VibrationState>>> =
    std::sync::LazyLock::new(|| Arc::new(Mutex::new(VibrationState::default())));

/// Internal vibration state
#[derive(Debug, Clone, Default)]
struct VibrationState {
    /// Current vibration pattern (alternating vibrate/pause durations in ms)
    pattern: Vec<u64>,
    /// Whether vibration is currently active
    is_vibrating: bool,
    /// Total vibration requests count
    request_count: u64,
}

/// navigator.vibrate(pattern) - Vibrates the device
///
/// The pattern can be:
/// - A single number: vibrate for that many milliseconds
/// - An array of numbers: alternating vibrate/pause durations
///
/// Returns true if vibration was initiated, false otherwise
pub fn navigator_vibrate(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let pattern_arg = args.get_or_undefined(0);

    // Parse the vibration pattern
    let pattern = parse_vibration_pattern(pattern_arg, context)?;

    // Validate and normalize the pattern
    let normalized_pattern = normalize_pattern(pattern);

    // Update global state
    {
        let mut state = VIBRATION_STATE.lock().unwrap();

        // If pattern is empty or all zeros, cancel vibration
        if normalized_pattern.is_empty() || normalized_pattern.iter().all(|&v| v == 0) {
            state.is_vibrating = false;
            state.pattern.clear();
        } else {
            state.is_vibrating = true;
            state.pattern = normalized_pattern;
            state.request_count += 1;
        }
    }

    // In headless mode, we always return true (simulating successful vibration)
    Ok(JsValue::from(true))
}

/// Parse the vibration pattern from a JsValue
fn parse_vibration_pattern(value: &JsValue, context: &mut Context) -> JsResult<Vec<u64>> {
    // If it's a number, treat as single vibration duration
    if let Some(num) = value.as_number() {
        return Ok(vec![num.max(0.0) as u64]);
    }

    // If it's an array, parse each element
    if let Some(obj) = value.as_object() {
        let length = obj.get(js_string!("length"), context)?
            .to_u32(context)
            .unwrap_or(0);

        let mut pattern = Vec::with_capacity(length as usize);

        for i in 0..length {
            let element = obj.get(i, context)?;
            let duration = element.to_number(context)?.max(0.0) as u64;
            pattern.push(duration);
        }

        return Ok(pattern);
    }

    // If undefined or null, return empty pattern (cancels vibration)
    if value.is_undefined() || value.is_null() {
        return Ok(vec![]);
    }

    // Try to convert to number
    let num = value.to_number(context)?.max(0.0) as u64;
    Ok(vec![num])
}

/// Normalize the vibration pattern according to spec
/// - Truncate values to reasonable maximums
/// - Handle edge cases
fn normalize_pattern(pattern: Vec<u64>) -> Vec<u64> {
    // Maximum single vibration duration (10 seconds)
    const MAX_DURATION: u64 = 10000;
    // Maximum pattern length
    const MAX_PATTERN_LENGTH: usize = 100;

    pattern
        .into_iter()
        .take(MAX_PATTERN_LENGTH)
        .map(|d| d.min(MAX_DURATION))
        .collect()
}

/// Cancel any ongoing vibration
pub fn cancel_vibration() {
    let mut state = VIBRATION_STATE.lock().unwrap();
    state.is_vibrating = false;
    state.pattern.clear();
}

/// Get the current vibration pattern (for testing)
pub fn get_vibration_pattern() -> Vec<u64> {
    let state = VIBRATION_STATE.lock().unwrap();
    state.pattern.clone()
}

/// Check if device is currently vibrating (for testing)
pub fn is_vibrating() -> bool {
    let state = VIBRATION_STATE.lock().unwrap();
    state.is_vibrating
}

/// Get the total vibration request count (for testing)
pub fn get_vibration_request_count() -> u64 {
    let state = VIBRATION_STATE.lock().unwrap();
    state.request_count
}

/// Reset vibration state (for testing)
pub fn reset_vibration_state() {
    let mut state = VIBRATION_STATE.lock().unwrap();
    *state = VibrationState::default();
}
