use anyhow::Result;
use boa_engine::{Context, JsObject, JsValue, NativeFunction, js_string};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

/// Geolocation API implementation for full browser compatibility
pub struct GeolocationManager {
    permissions: Arc<Mutex<HashMap<String, bool>>>,
    positions: Arc<Mutex<Vec<Position>>>,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: f64,
    pub altitude: Option<f64>,
    pub altitude_accuracy: Option<f64>,
    pub heading: Option<f64>,
    pub speed: Option<f64>,
    pub timestamp: u64,
}

impl GeolocationManager {
    pub fn new() -> Self {
        Self {
            permissions: Arc::new(Mutex::new(HashMap::new())),
            positions: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Setup Geolocation API in navigator
    pub fn setup_geolocation_api(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        // Get navigator object
        let navigator_obj = context.global_object().get(js_string!("navigator"), context)?;

        if let Some(nav_obj) = navigator_obj.as_object() {
            let geolocation_obj = JsObject::default();

            // navigator.geolocation.getCurrentPosition()
            let get_current_position_fn = unsafe { NativeFunction::from_closure({
                let positions = Arc::clone(&self.positions);
                move |_, args, context| {
                    if args.is_empty() {
                        return Err(boa_engine::JsNativeError::typ()
                            .with_message("getCurrentPosition requires a success callback")
                            .into());
                    }

                    // Simulate getting position (default to San Francisco for demo)
                    let position_obj = JsObject::default();
                    let coords_obj = JsObject::default();

                    // Default coordinates (San Francisco)
                    coords_obj.set(js_string!("latitude"), JsValue::from(37.7749), false, context)?;
                    coords_obj.set(js_string!("longitude"), JsValue::from(-122.4194), false, context)?;
                    coords_obj.set(js_string!("accuracy"), JsValue::from(100.0), false, context)?;
                    coords_obj.set(js_string!("altitude"), JsValue::null(), false, context)?;
                    coords_obj.set(js_string!("altitudeAccuracy"), JsValue::null(), false, context)?;
                    coords_obj.set(js_string!("heading"), JsValue::null(), false, context)?;
                    coords_obj.set(js_string!("speed"), JsValue::null(), false, context)?;

                    position_obj.set(js_string!("coords"), JsValue::from(coords_obj), false, context)?;
                    position_obj.set(js_string!("timestamp"), JsValue::from(js_sys::Date::now() as i64), false, context)?;

                    // Call success callback
                    if args[0].is_callable() {
                        if let Some(callback) = args[0].as_callable() {
                            let _ = callback.call(&JsValue::undefined(), &[JsValue::from(position_obj)], context);
                        }
                    }

                    Ok(JsValue::undefined())
                }
            }) };
            geolocation_obj.set(js_string!("getCurrentPosition"), JsValue::from(get_current_position_fn.to_js_function(context.realm())), false, context)?;

            // navigator.geolocation.watchPosition()
            let watch_position_fn = unsafe { NativeFunction::from_closure(|_, args, context| {
                if args.is_empty() {
                    return Err(boa_engine::JsNativeError::typ()
                        .with_message("watchPosition requires a success callback")
                        .into());
                }

                // Return a watch ID (simulate)
                let watch_id = js_sys::Math::random() * 1000000.0;
                Ok(JsValue::from(watch_id as i32))
            }) };
            geolocation_obj.set(js_string!("watchPosition"), JsValue::from(watch_position_fn.to_js_function(context.realm())), false, context)?;

            // navigator.geolocation.clearWatch()
            let clear_watch_fn = unsafe { NativeFunction::from_closure(|_, args, _context| {
                Ok(JsValue::undefined())
            }) };
            geolocation_obj.set(js_string!("clearWatch"), JsValue::from(clear_watch_fn.to_js_function(context.realm())), false, context)?;

            nav_obj.set(js_string!("geolocation"), JsValue::from(geolocation_obj), false, context)?;
        }

        Ok(())
    }
}

// Helper module for JS Date.now() simulation
mod js_sys {
    use std::time::{SystemTime, UNIX_EPOCH};

    pub struct Date;
    pub struct Math;

    impl Date {
        pub fn now() -> f64 {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64
        }
    }

    impl Math {
        pub fn random() -> f64 {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            let mut hasher = DefaultHasher::new();
            SystemTime::now().hash(&mut hasher);
            (hasher.finish() % 1000000) as f64 / 1000000.0
        }
    }
}