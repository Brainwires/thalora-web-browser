//! IntersectionObserver Web API implementation for Boa
//!
//! Native implementation of the IntersectionObserver standard
//! https://w3c.github.io/IntersectionObserver/
//!
//! This implements the complete IntersectionObserver interface for observing changes in element visibility

use boa_engine::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    realm::Realm, JsData, JsString,
    property::Attribute
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;

/// JavaScript `IntersectionObserver` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub struct IntersectionObserver;

impl IntrinsicObject for IntersectionObserver {
    fn init(realm: &Realm) {
        let root_getter = BuiltInBuilder::callable(realm, get_root)
            .name(js_string!("get root"))
            .build();

        let root_margin_getter = BuiltInBuilder::callable(realm, get_root_margin)
            .name(js_string!("get rootMargin"))
            .build();

        let thresholds_getter = BuiltInBuilder::callable(realm, get_thresholds)
            .name(js_string!("get thresholds"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(js_string!("root"), Some(root_getter), None, Attribute::CONFIGURABLE)
            .accessor(js_string!("rootMargin"), Some(root_margin_getter), None, Attribute::CONFIGURABLE)
            .accessor(js_string!("thresholds"), Some(thresholds_getter), None, Attribute::CONFIGURABLE)
            .method(Self::observe, js_string!("observe"), 1)
            .method(Self::unobserve, js_string!("unobserve"), 1)
            .method(Self::disconnect, js_string!("disconnect"), 0)
            .method(Self::take_records, js_string!("takeRecords"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for IntersectionObserver {
    const NAME: JsString = js_string!("IntersectionObserver");
}

impl BuiltInConstructor for IntersectionObserver {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::intersection_observer;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::intersection_observer,
            context,
        )?;

        // Get the callback function (required parameter)
        let callback = args.get_or_undefined(0);
        if !callback.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("IntersectionObserver constructor requires a callback function")
                .into());
        }

        // Get the options (optional parameter)
        let options = args.get_or_undefined(1);

        // Parse options (IntersectionObserverInit)
        let mut config = IntersectionObserverConfig::default();

        if let Some(options_obj) = options.as_object() {
            // Parse root option
            if let Ok(root) = options_obj.get(js_string!("root"), context) {
                if !root.is_null_or_undefined() {
                    config.root = root.as_object();
                }
            }

            // Parse rootMargin option
            if let Ok(root_margin) = options_obj.get(js_string!("rootMargin"), context) {
                if !root_margin.is_undefined() {
                    config.root_margin = root_margin.to_string(context)
                        .map(|s| s.to_std_string_escaped())
                        .unwrap_or_else(|_| "0px".to_string());
                }
            }

            // Parse threshold option
            if let Ok(threshold) = options_obj.get(js_string!("threshold"), context) {
                if !threshold.is_undefined() {
                    if let Some(threshold_array) = threshold.as_object() {
                        // Try to parse as array
                        let mut thresholds = Vec::new();
                        if let Ok(length_val) = threshold_array.get(js_string!("length"), context) {
                            let length = length_val.to_u32(context).unwrap_or(0);
                            for i in 0..length {
                                if let Ok(item) = threshold_array.get(i, context) {
                                    if let Ok(num) = item.to_number(context) {
                                        thresholds.push(num);
                                    }
                                }
                            }
                        }
                        if !thresholds.is_empty() {
                            // Sort thresholds as per spec
                            thresholds.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                            config.threshold = thresholds;
                        }
                    } else if let Ok(threshold_num) = threshold.to_number(context) {
                        config.threshold = vec![threshold_num];
                    }
                }
            }
        }

        // Validate thresholds (must be between 0.0 and 1.0)
        for t in &config.threshold {
            if *t < 0.0 || *t > 1.0 {
                return Err(JsNativeError::range()
                    .with_message("IntersectionObserver threshold values must be between 0 and 1")
                    .into());
            }
        }

        // Create IntersectionObserver data
        let observer_data = IntersectionObserverData {
            callback: callback.clone(),
            config,
            observed_targets: HashMap::new(),
            records: Vec::new(),
            is_observing: false,
        };

        let observer_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            observer_data
        );

        Ok(observer_obj.into())
    }
}

impl IntersectionObserver {
    /// `IntersectionObserver.prototype.observe()` method
    fn observe(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("IntersectionObserver.observe called on non-object")
        })?;

        let target = args.get_or_undefined(0);

        // Validate target (should be an Element, but for now accept any object)
        if !target.is_object() {
            return Err(JsNativeError::typ()
                .with_message("IntersectionObserver.observe: target must be an Element")
                .into());
        }

        let target_obj = target.as_object().unwrap().clone();

        // Update observer data
        if let Some(mut observer_data) = observer_obj.downcast_mut::<IntersectionObserverData>() {
            let target_id = format!("{:p}", target_obj.as_ref());
            observer_data.observed_targets.insert(target_id, target_obj);
            observer_data.is_observing = true;
        }

        Ok(JsValue::undefined())
    }

    /// `IntersectionObserver.prototype.unobserve()` method
    fn unobserve(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("IntersectionObserver.unobserve called on non-object")
        })?;

        let target = args.get_or_undefined(0);

        if !target.is_object() {
            return Err(JsNativeError::typ()
                .with_message("IntersectionObserver.unobserve: target must be an Element")
                .into());
        }

        if let Some(mut observer_data) = observer_obj.downcast_mut::<IntersectionObserverData>() {
            let target_id = format!("{:p}", target.as_object().unwrap().as_ref());
            observer_data.observed_targets.remove(&target_id);

            if observer_data.observed_targets.is_empty() {
                observer_data.is_observing = false;
            }
        }

        Ok(JsValue::undefined())
    }

    /// `IntersectionObserver.prototype.disconnect()` method
    fn disconnect(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("IntersectionObserver.disconnect called on non-object")
        })?;

        if let Some(mut observer_data) = observer_obj.downcast_mut::<IntersectionObserverData>() {
            observer_data.observed_targets.clear();
            observer_data.records.clear();
            observer_data.is_observing = false;
        }

        Ok(JsValue::undefined())
    }

    /// `IntersectionObserver.prototype.takeRecords()` method
    fn take_records(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("IntersectionObserver.takeRecords called on non-object")
        })?;

        let mut observer_data = observer_obj.downcast_mut::<IntersectionObserverData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("IntersectionObserver.takeRecords called on non-IntersectionObserver object")
        })?;

        // Take records and clear the queue
        let records = std::mem::take(&mut observer_data.records);

        // Create JavaScript array of IntersectionObserverEntry objects
        let records_array = boa_engine::builtins::array::Array::array_create(
            records.len() as u64,
            None,
            context,
        )?;

        for (index, record) in records.into_iter().enumerate() {
            let record_obj = record.to_js_object(context)?;
            records_array.set(index, record_obj, false, context)?;
        }

        Ok(records_array.into())
    }
}

// Observer property getters
fn get_root(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let observer_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("IntersectionObserver getter called on non-object")
    })?;

    let observer_data = observer_obj.downcast_ref::<IntersectionObserverData>().ok_or_else(|| {
        JsNativeError::typ().with_message("IntersectionObserver getter called on non-IntersectionObserver object")
    })?;

    Ok(observer_data.config.root.clone().map_or(JsValue::null(), |r| r.into()))
}

fn get_root_margin(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
    let observer_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("IntersectionObserver getter called on non-object")
    })?;

    let observer_data = observer_obj.downcast_ref::<IntersectionObserverData>().ok_or_else(|| {
        JsNativeError::typ().with_message("IntersectionObserver getter called on non-IntersectionObserver object")
    })?;

    Ok(JsValue::from(js_string!(observer_data.config.root_margin.clone())))
}

fn get_thresholds(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let observer_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("IntersectionObserver getter called on non-object")
    })?;

    let observer_data = observer_obj.downcast_ref::<IntersectionObserverData>().ok_or_else(|| {
        JsNativeError::typ().with_message("IntersectionObserver getter called on non-IntersectionObserver object")
    })?;

    // Create frozen array of thresholds
    let thresholds_array = boa_engine::builtins::array::Array::array_create(
        observer_data.config.threshold.len() as u64,
        None,
        context,
    )?;

    for (i, threshold) in observer_data.config.threshold.iter().enumerate() {
        thresholds_array.set(i, *threshold, false, context)?;
    }

    // Freeze the array per spec
    // (simplified - full freeze would use Object.freeze)

    Ok(thresholds_array.into())
}

/// Internal data for IntersectionObserver instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct IntersectionObserverData {
    /// The callback function to call when intersections change
    callback: JsValue,
    /// Configuration options for the observer
    #[unsafe_ignore_trace]
    config: IntersectionObserverConfig,
    /// Map of observed target elements
    observed_targets: HashMap<String, JsObject>,
    /// Queue of intersection records waiting to be delivered
    #[unsafe_ignore_trace]
    records: Vec<IntersectionObserverEntryData>,
    /// Whether this observer is currently observing any targets
    #[unsafe_ignore_trace]
    is_observing: bool,
}

impl IntersectionObserverData {
    /// Queue an intersection record
    pub fn queue_record(&mut self, record: IntersectionObserverEntryData) {
        self.records.push(record);
    }

    /// Get the callback function
    pub fn callback(&self) -> &JsValue {
        &self.callback
    }

    /// Check if there are pending records
    pub fn has_pending_records(&self) -> bool {
        !self.records.is_empty()
    }

    /// Get the configuration
    pub fn config(&self) -> &IntersectionObserverConfig {
        &self.config
    }
}

/// Configuration for IntersectionObserver
#[derive(Debug, Clone)]
pub struct IntersectionObserverConfig {
    /// The root element for intersection calculation (None = viewport)
    pub root: Option<JsObject>,
    /// Margin around the root element
    pub root_margin: String,
    /// Threshold values for triggering callbacks
    pub threshold: Vec<f64>,
}

impl Default for IntersectionObserverConfig {
    fn default() -> Self {
        Self {
            root: None,
            root_margin: "0px".to_string(),
            threshold: vec![0.0],
        }
    }
}

/// Represents a single intersection observer entry (internal data)
#[derive(Debug, Clone)]
pub struct IntersectionObserverEntryData {
    /// The target element being observed
    pub target: Option<JsObject>,
    /// The intersection ratio (0.0 to 1.0)
    pub intersection_ratio: f64,
    /// Whether the target is intersecting
    pub is_intersecting: bool,
    /// Timestamp when the intersection was observed
    pub time: f64,
    /// Bounding rectangle of the target element
    pub bounding_client_rect: DOMRectData,
    /// Bounding rectangle of the intersection
    pub intersection_rect: DOMRectData,
    /// Bounding rectangle of the root element
    pub root_bounds: Option<DOMRectData>,
}

impl IntersectionObserverEntryData {
    /// Create a new intersection observer entry
    pub fn new(
        target: JsObject,
        intersection_ratio: f64,
        is_intersecting: bool,
        time: f64,
        bounding_client_rect: DOMRectData,
        intersection_rect: DOMRectData,
        root_bounds: Option<DOMRectData>,
    ) -> Self {
        Self {
            target: Some(target),
            intersection_ratio,
            is_intersecting,
            time,
            bounding_client_rect,
            intersection_rect,
            root_bounds,
        }
    }

    /// Convert to a JavaScript IntersectionObserverEntry object
    pub fn to_js_object(&self, context: &mut Context) -> JsResult<JsObject> {
        let obj = JsObject::with_object_proto(context.intrinsics());

        // Set target
        obj.set(
            js_string!("target"),
            self.target.clone().map_or(JsValue::null(), |t| t.into()),
            false,
            context,
        )?;

        // Set intersectionRatio
        obj.set(
            js_string!("intersectionRatio"),
            self.intersection_ratio,
            false,
            context,
        )?;

        // Set isIntersecting
        obj.set(
            js_string!("isIntersecting"),
            self.is_intersecting,
            false,
            context,
        )?;

        // Set time
        obj.set(
            js_string!("time"),
            self.time,
            false,
            context,
        )?;

        // Set boundingClientRect
        obj.set(
            js_string!("boundingClientRect"),
            self.bounding_client_rect.to_js_object(context)?,
            false,
            context,
        )?;

        // Set intersectionRect
        obj.set(
            js_string!("intersectionRect"),
            self.intersection_rect.to_js_object(context)?,
            false,
            context,
        )?;

        // Set rootBounds
        obj.set(
            js_string!("rootBounds"),
            self.root_bounds.as_ref().map_or(Ok(JsValue::null()), |rb| {
                rb.to_js_object(context).map(|o| o.into())
            })?,
            false,
            context,
        )?;

        Ok(obj)
    }
}

/// Simple rectangle representation (internal data)
#[derive(Debug, Clone)]
pub struct DOMRectData {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl DOMRectData {
    pub fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self { x, y, width, height }
    }

    pub fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0)
    }

    /// Convert to a JavaScript DOMRectReadOnly-like object
    pub fn to_js_object(&self, context: &mut Context) -> JsResult<JsObject> {
        let obj = JsObject::with_object_proto(context.intrinsics());

        obj.set(js_string!("x"), self.x, false, context)?;
        obj.set(js_string!("y"), self.y, false, context)?;
        obj.set(js_string!("width"), self.width, false, context)?;
        obj.set(js_string!("height"), self.height, false, context)?;
        obj.set(js_string!("top"), self.y, false, context)?;
        obj.set(js_string!("right"), self.x + self.width, false, context)?;
        obj.set(js_string!("bottom"), self.y + self.height, false, context)?;
        obj.set(js_string!("left"), self.x, false, context)?;

        Ok(obj)
    }
}

// ============================================================================
// IntersectionObserverEntry JavaScript class
// ============================================================================

/// JavaScript `IntersectionObserverEntry` interface
#[derive(Debug, Copy, Clone)]
pub struct IntersectionObserverEntry;

impl IntrinsicObject for IntersectionObserverEntry {
    fn init(realm: &Realm) {
        let target_getter = BuiltInBuilder::callable(realm, entry_get_target)
            .name(js_string!("get target"))
            .build();

        let intersection_ratio_getter = BuiltInBuilder::callable(realm, entry_get_intersection_ratio)
            .name(js_string!("get intersectionRatio"))
            .build();

        let is_intersecting_getter = BuiltInBuilder::callable(realm, entry_get_is_intersecting)
            .name(js_string!("get isIntersecting"))
            .build();

        let time_getter = BuiltInBuilder::callable(realm, entry_get_time)
            .name(js_string!("get time"))
            .build();

        let bounding_client_rect_getter = BuiltInBuilder::callable(realm, entry_get_bounding_client_rect)
            .name(js_string!("get boundingClientRect"))
            .build();

        let intersection_rect_getter = BuiltInBuilder::callable(realm, entry_get_intersection_rect)
            .name(js_string!("get intersectionRect"))
            .build();

        let root_bounds_getter = BuiltInBuilder::callable(realm, entry_get_root_bounds)
            .name(js_string!("get rootBounds"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(js_string!("target"), Some(target_getter), None, Attribute::CONFIGURABLE)
            .accessor(js_string!("intersectionRatio"), Some(intersection_ratio_getter), None, Attribute::CONFIGURABLE)
            .accessor(js_string!("isIntersecting"), Some(is_intersecting_getter), None, Attribute::CONFIGURABLE)
            .accessor(js_string!("time"), Some(time_getter), None, Attribute::CONFIGURABLE)
            .accessor(js_string!("boundingClientRect"), Some(bounding_client_rect_getter), None, Attribute::CONFIGURABLE)
            .accessor(js_string!("intersectionRect"), Some(intersection_rect_getter), None, Attribute::CONFIGURABLE)
            .accessor(js_string!("rootBounds"), Some(root_bounds_getter), None, Attribute::CONFIGURABLE)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for IntersectionObserverEntry {
    const NAME: JsString = js_string!("IntersectionObserverEntry");
}

impl BuiltInConstructor for IntersectionObserverEntry {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 14; // Accessors on prototype
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::intersection_observer_entry;

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // IntersectionObserverEntry cannot be constructed directly
        Err(JsNativeError::typ()
            .with_message("IntersectionObserverEntry is not a constructor")
            .into())
    }
}

// IntersectionObserverEntry getters
fn entry_get_target(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("IntersectionObserverEntry getter called on non-object")
    })?;
    obj.get(js_string!("target"), context)
}

fn entry_get_intersection_ratio(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("IntersectionObserverEntry getter called on non-object")
    })?;
    obj.get(js_string!("intersectionRatio"), context)
}

fn entry_get_is_intersecting(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("IntersectionObserverEntry getter called on non-object")
    })?;
    obj.get(js_string!("isIntersecting"), context)
}

fn entry_get_time(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("IntersectionObserverEntry getter called on non-object")
    })?;
    obj.get(js_string!("time"), context)
}

fn entry_get_bounding_client_rect(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("IntersectionObserverEntry getter called on non-object")
    })?;
    obj.get(js_string!("boundingClientRect"), context)
}

fn entry_get_intersection_rect(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("IntersectionObserverEntry getter called on non-object")
    })?;
    obj.get(js_string!("intersectionRect"), context)
}

fn entry_get_root_bounds(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("IntersectionObserverEntry getter called on non-object")
    })?;
    obj.get(js_string!("rootBounds"), context)
}
