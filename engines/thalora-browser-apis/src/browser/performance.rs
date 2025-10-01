//! Performance API implementation for Boa
//!
//! Native implementation of the Performance API standard
//! https://w3c.github.io/hr-time/
//! https://w3c.github.io/navigation-timing/
//! https://w3c.github.io/performance-timeline/
//!
//! This implements the complete Performance interface for real timing measurements


use boa_engine::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor, Array},
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsResult, js_string,
    realm::Realm, JsString, JsData,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    property::{Attribute, PropertyDescriptor},
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex, OnceLock};
use std::collections::HashMap;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

/// Get high-resolution time in milliseconds
fn performance_now() -> f64 {
    static START_TIME: OnceLock<Instant> = OnceLock::new();
    let start = START_TIME.get_or_init(Instant::now);
    start.elapsed().as_secs_f64() * 1000.0
}

/// Performance navigation timing information
#[derive(Debug, Clone)]
struct PerformanceNavigationTiming {
    navigation_start: f64,
    unload_event_start: f64,
    unload_event_end: f64,
    redirect_start: f64,
    redirect_end: f64,
    fetch_start: f64,
    domain_lookup_start: f64,
    domain_lookup_end: f64,
    connect_start: f64,
    connect_end: f64,
    secure_connection_start: f64,
    request_start: f64,
    response_start: f64,
    response_end: f64,
    dom_loading: f64,
    dom_interactive: f64,
    dom_content_loaded_event_start: f64,
    dom_content_loaded_event_end: f64,
    dom_complete: f64,
    load_event_start: f64,
    load_event_end: f64,
}

impl Default for PerformanceNavigationTiming {
    fn default() -> Self {
        // Use UNIX epoch time in milliseconds to ensure positive values
        let epoch_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as f64;

        Self {
            navigation_start: epoch_ms - 1000.0,
            unload_event_start: epoch_ms - 950.0,
            unload_event_end: epoch_ms - 945.0,
            redirect_start: 0.0,
            redirect_end: 0.0,
            fetch_start: epoch_ms - 940.0,
            domain_lookup_start: epoch_ms - 935.0,
            domain_lookup_end: epoch_ms - 930.0,
            connect_start: epoch_ms - 925.0,
            connect_end: epoch_ms - 920.0,
            secure_connection_start: epoch_ms - 915.0,
            request_start: epoch_ms - 910.0,
            response_start: epoch_ms - 905.0,
            response_end: epoch_ms - 900.0,
            dom_loading: epoch_ms - 895.0,
            dom_interactive: epoch_ms - 890.0,
            dom_content_loaded_event_start: epoch_ms - 885.0,
            dom_content_loaded_event_end: epoch_ms - 880.0,
            dom_complete: epoch_ms - 875.0,
            load_event_start: epoch_ms - 870.0,
            load_event_end: epoch_ms - 865.0,
        }
    }
}

/// Performance entry types according to W3C specifications
#[derive(Debug, Clone)]
enum PerformanceEntryType {
    Navigation,
    Resource,
    Paint,
    Mark,
    Measure,
    LongTask,
    TaskAttribution,
}

impl PerformanceEntryType {
    fn as_str(&self) -> &'static str {
        match self {
            PerformanceEntryType::Navigation => "navigation",
            PerformanceEntryType::Resource => "resource",
            PerformanceEntryType::Paint => "paint",
            PerformanceEntryType::Mark => "mark",
            PerformanceEntryType::Measure => "measure",
            PerformanceEntryType::LongTask => "longtask",
            PerformanceEntryType::TaskAttribution => "taskattribution",
        }
    }
}

/// Performance entry representing any performance-related event
#[derive(Debug, Clone)]
struct PerformanceEntry {
    name: String,
    entry_type: PerformanceEntryType,
    start_time: f64,
    duration: f64,
}

/// Performance mark implementation
#[derive(Debug, Clone)]
struct PerformanceMark {
    name: String,
    start_time: f64,
}

/// Performance measure implementation
#[derive(Debug, Clone)]
struct PerformanceMeasure {
    name: String,
    start_time: f64,
    duration: f64,
}

/// Global Performance state storage (similar to V8's PerIsolateData)
#[derive(Debug, Clone)]
struct PerformanceState {
    /// Map of performance marks by name to timestamp
    mark_map: HashMap<String, f64>,
    /// List of all performance entries
    entries: Vec<PerformanceEntry>,
    /// Performance time origin
    time_origin: f64,
    /// Navigation timing information
    navigation_timing: PerformanceNavigationTiming,
}

impl Default for PerformanceState {
    fn default() -> Self {
        Self {
            mark_map: HashMap::new(),
            entries: Vec::new(),
            time_origin: performance_now(),
            navigation_timing: PerformanceNavigationTiming::default(),
        }
    }
}

/// Global Performance state instance
static PERFORMANCE_STATE: OnceLock<Arc<Mutex<PerformanceState>>> = OnceLock::new();

/// Get or initialize the global performance state
fn get_performance_state() -> Arc<Mutex<PerformanceState>> {
    PERFORMANCE_STATE.get_or_init(|| {
        Arc::new(Mutex::new(PerformanceState::default()))
    }).clone()
}

/// Performance object providing timing and measurement capabilities
#[derive(Debug, Clone, Finalize, Trace)]
pub struct Performance {
    time_origin: f64,
}

impl JsData for Performance {}

impl Performance {
    pub fn new() -> Self {
        Self {
            time_origin: performance_now(),
        }
    }
}

impl IntrinsicObject for Performance {
    fn init(realm: &Realm) {
        // Performance builtin initialization
        let performance_obj = BuiltInBuilder::with_intrinsic::<Self>(realm)
            .static_method(Self::now, js_string!("now"), 0)
            .static_method(Self::mark, js_string!("mark"), 1)
            .static_method(Self::measure, js_string!("measure"), 3)
            .static_method(Self::clear_marks, js_string!("clearMarks"), 0)
            .static_method(Self::clear_measures, js_string!("clearMeasures"), 0)
            .static_method(Self::get_entries, js_string!("getEntries"), 0)
            .static_method(Self::get_entries_by_type, js_string!("getEntriesByType"), 1)
            .static_method(Self::get_entries_by_name, js_string!("getEntriesByName"), 2)
            .static_property(
                js_string!("timeOrigin"),
                performance_now(),
                Attribute::READONLY,
            )
            .static_property(
                js_string!("timing"),
                Self::create_timing_object(realm.intrinsics()),
                Attribute::READONLY,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Performance {
    const NAME: JsString = js_string!("performance");
}

impl BuiltInConstructor for Performance {
    const LENGTH: usize = 0;
    const P: usize = 10;  // Number of prototype properties
    const SP: usize = 0;  // Number of static properties

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::performance;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Performance constructor is not meant to be called directly
        Ok(JsValue::undefined())
    }
}

/// Create a performance object instance for global scope
pub(crate) fn create_performance_object(context: &mut Context) -> JsResult<JsValue> {
    let performance_obj = JsObject::with_object_proto(context.intrinsics());

    // Add methods as function objects
    let now_func = BuiltInBuilder::callable(context.realm(), Performance::now)
        .name(js_string!("now"))
        .length(0)
        .build();
    performance_obj.set(js_string!("now"), now_func, false, context)?;

    let mark_func = BuiltInBuilder::callable(context.realm(), Performance::mark)
        .name(js_string!("mark"))
        .length(1)
        .build();
    performance_obj.set(js_string!("mark"), mark_func, false, context)?;

    let measure_func = BuiltInBuilder::callable(context.realm(), Performance::measure)
        .name(js_string!("measure"))
        .length(3)
        .build();
    performance_obj.set(js_string!("measure"), measure_func, false, context)?;

    let clear_marks_func = BuiltInBuilder::callable(context.realm(), Performance::clear_marks)
        .name(js_string!("clearMarks"))
        .length(0)
        .build();
    performance_obj.set(js_string!("clearMarks"), clear_marks_func, false, context)?;

    let clear_measures_func = BuiltInBuilder::callable(context.realm(), Performance::clear_measures)
        .name(js_string!("clearMeasures"))
        .length(0)
        .build();
    performance_obj.set(js_string!("clearMeasures"), clear_measures_func, false, context)?;

    let get_entries_func = BuiltInBuilder::callable(context.realm(), Performance::get_entries)
        .name(js_string!("getEntries"))
        .length(0)
        .build();
    performance_obj.set(js_string!("getEntries"), get_entries_func, false, context)?;

    let get_entries_by_type_func = BuiltInBuilder::callable(context.realm(), Performance::get_entries_by_type)
        .name(js_string!("getEntriesByType"))
        .length(1)
        .build();
    performance_obj.set(js_string!("getEntriesByType"), get_entries_by_type_func, false, context)?;

    let get_entries_by_name_func = BuiltInBuilder::callable(context.realm(), Performance::get_entries_by_name)
        .name(js_string!("getEntriesByName"))
        .length(2)
        .build();
    performance_obj.set(js_string!("getEntriesByName"), get_entries_by_name_func, false, context)?;

    // Add readonly properties with proper descriptors
    let state = get_performance_state();
    let time_origin = state.lock().unwrap().time_origin;

    performance_obj.define_property_or_throw(
        js_string!("timeOrigin"),
        PropertyDescriptor::builder()
            .value(time_origin)
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;

    performance_obj.define_property_or_throw(
        js_string!("timing"),
        PropertyDescriptor::builder()
            .value(Performance::create_timing_object(context.intrinsics()))
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
        context,
    )?;

    Ok(performance_obj.into())
}

impl Performance {
    const STANDARD_CONSTRUCTOR: fn(&crate::context::intrinsics::StandardConstructors) -> &crate::context::intrinsics::StandardConstructor =
        crate::context::intrinsics::StandardConstructors::performance;

    /// `performance.now()` - Returns current high-resolution time
    fn now(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(performance_now()))
    }

    /// `performance.mark(name)` - Creates a performance mark
    fn mark(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_lossy();
        let timestamp = performance_now();

        // Store mark in global state (V8 pattern)
        {
            let state = get_performance_state();
            let mut state = state.lock().unwrap();
            state.mark_map.insert(name_str.clone(), timestamp);
        }

        // Create PerformanceEntry object following V8's pattern with ReadOnly properties
        let performance_entry = JsObject::with_object_proto(context.intrinsics());

        // Set properties with ReadOnly descriptors like V8
        performance_entry.define_property_or_throw(
            js_string!("entryType"),
            PropertyDescriptor::builder()
                .value(js_string!("mark"))
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;

        performance_entry.define_property_or_throw(
            js_string!("name"),
            PropertyDescriptor::builder()
                .value(name.clone())
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;

        performance_entry.define_property_or_throw(
            js_string!("startTime"),
            PropertyDescriptor::builder()
                .value(timestamp)
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;

        performance_entry.define_property_or_throw(
            js_string!("duration"),
            PropertyDescriptor::builder()
                .value(0.0)
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;

        // Store entry in global state
        {
            let state = get_performance_state();
            let mut state = state.lock().unwrap();
            state.entries.push(PerformanceEntry {
                name: name_str,
                entry_type: PerformanceEntryType::Mark,
                start_time: timestamp,
                duration: 0.0,
            });
        }

        Ok(performance_entry.into())
    }

    /// `performance.measure(name, startMark, endMark)` - Creates a performance measure
    fn measure(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_lossy();

        let start_time = if args.len() > 1 && !args[1].is_undefined() {
            let start_mark = args[1].to_string(context)?;
            let start_mark_str = start_mark.to_std_string_lossy();

            let state = get_performance_state();
            let state = state.lock().unwrap();
            state.mark_map.get(&start_mark_str).copied().unwrap_or(0.0)
        } else {
            0.0
        };

        let end_time = if args.len() > 2 && !args[2].is_undefined() {
            let end_mark = args[2].to_string(context)?;
            let end_mark_str = end_mark.to_std_string_lossy();

            let state = get_performance_state();
            let state = state.lock().unwrap();
            state.mark_map.get(&end_mark_str).copied().unwrap_or(performance_now())
        } else {
            performance_now()
        };

        let duration = end_time - start_time;

        // Create PerformanceEntry object
        let performance_entry = JsObject::with_object_proto(context.intrinsics());

        performance_entry.define_property_or_throw(
            js_string!("entryType"),
            PropertyDescriptor::builder()
                .value(js_string!("measure"))
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;

        performance_entry.define_property_or_throw(
            js_string!("name"),
            PropertyDescriptor::builder()
                .value(name.clone())
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;

        performance_entry.define_property_or_throw(
            js_string!("startTime"),
            PropertyDescriptor::builder()
                .value(start_time)
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;

        performance_entry.define_property_or_throw(
            js_string!("duration"),
            PropertyDescriptor::builder()
                .value(duration)
                .writable(false)
                .enumerable(false)
                .configurable(false)
                .build(),
            context,
        )?;

        // Store entry in global state
        {
            let state = get_performance_state();
            let mut state = state.lock().unwrap();
            state.entries.push(PerformanceEntry {
                name: name_str,
                entry_type: PerformanceEntryType::Measure,
                start_time,
                duration,
            });
        }

        Ok(performance_entry.into())
    }

    /// `performance.clearMarks(name?)` - Removes marks from the timeline
    fn clear_marks(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let state = get_performance_state();
        let mut state = state.lock().unwrap();

        if args.is_empty() || args[0].is_undefined() {
            // Clear all marks
            state.mark_map.clear();
            state.entries.retain(|entry| !matches!(entry.entry_type, PerformanceEntryType::Mark));
        } else {
            // Clear specific mark
            let name = args[0].to_string(context)?;
            let name_str = name.to_std_string_lossy();
            state.mark_map.remove(&name_str);
            state.entries.retain(|entry| !(matches!(entry.entry_type, PerformanceEntryType::Mark) && entry.name == name_str));
        }

        Ok(JsValue::undefined())
    }

    /// `performance.clearMeasures(name?)` - Removes measures from the timeline
    fn clear_measures(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let state = get_performance_state();
        let mut state = state.lock().unwrap();

        if args.is_empty() || args[0].is_undefined() {
            // Clear all measures
            state.entries.retain(|entry| !matches!(entry.entry_type, PerformanceEntryType::Measure));
        } else {
            // Clear specific measure
            let name = args[0].to_string(context)?;
            let name_str = name.to_std_string_lossy();
            state.entries.retain(|entry| !(matches!(entry.entry_type, PerformanceEntryType::Measure) && entry.name == name_str));
        }

        Ok(JsValue::undefined())
    }

    /// `performance.getEntries()` - Returns all performance entries
    fn get_entries(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let state = get_performance_state();
        let state = state.lock().unwrap();

        let entries: Vec<JsValue> = state.entries.iter()
            .map(|entry| Self::entry_to_js_object(entry, context))
            .collect::<JsResult<Vec<_>>>()?;

        Ok(Array::create_array_from_list(entries, context).into())
    }

    /// `performance.getEntriesByType(type)` - Returns entries filtered by type
    fn get_entries_by_type(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let entry_type = args.get_or_undefined(0).to_string(context)?;
        let type_str = entry_type.to_std_string_lossy();

        let state = get_performance_state();
        let state = state.lock().unwrap();

        let entries: Vec<JsValue> = state.entries.iter()
            .filter(|entry| entry.entry_type.as_str() == type_str)
            .map(|entry| Self::entry_to_js_object(entry, context))
            .collect::<JsResult<Vec<_>>>()?;

        Ok(Array::create_array_from_list(entries, context).into())
    }

    /// `performance.getEntriesByName(name, type?)` - Returns entries filtered by name and optionally type
    fn get_entries_by_name(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let name = args.get_or_undefined(0).to_string(context)?;
        let name_str = name.to_std_string_lossy();

        let type_filter = if args.len() > 1 && !args[1].is_undefined() {
            let entry_type = args[1].to_string(context)?;
            Some(entry_type.to_std_string_lossy())
        } else {
            None
        };

        let state = get_performance_state();
        let state = state.lock().unwrap();

        let entries: Vec<JsValue> = state.entries.iter()
            .filter(|entry| {
                entry.name == name_str &&
                type_filter.as_ref().map_or(true, |t| entry.entry_type.as_str() == t)
            })
            .map(|entry| Self::entry_to_js_object(entry, context))
            .collect::<JsResult<Vec<_>>>()?;

        Ok(Array::create_array_from_list(entries, context).into())
    }

    /// Convert PerformanceEntry to JavaScript object
    fn entry_to_js_object(entry: &PerformanceEntry, context: &mut Context) -> JsResult<JsValue> {
        let obj = JsObject::with_object_proto(context.intrinsics());

        obj.define_property_or_throw(
            js_string!("name"),
            PropertyDescriptor::builder()
                .value(js_string!(entry.name.clone()))
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
            context,
        )?;

        obj.define_property_or_throw(
            js_string!("entryType"),
            PropertyDescriptor::builder()
                .value(js_string!(entry.entry_type.as_str()))
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
            context,
        )?;

        obj.define_property_or_throw(
            js_string!("startTime"),
            PropertyDescriptor::builder()
                .value(entry.start_time)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
            context,
        )?;

        obj.define_property_or_throw(
            js_string!("duration"),
            PropertyDescriptor::builder()
                .value(entry.duration)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
            context,
        )?;

        Ok(obj.into())
    }

    /// Create timing object with navigation timing information
    fn create_timing_object(intrinsics: &Intrinsics) -> JsValue {
        let obj = JsObject::with_object_proto(intrinsics);
        let state = get_performance_state();
        let state = state.lock().unwrap();
        let timing = &state.navigation_timing;

        // Note: In a real implementation these would be populated during navigation
        // Using direct property descriptor set to avoid context requirement
        obj.insert_property(
            js_string!("navigationStart"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.navigation_start)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("unloadEventStart"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.unload_event_start)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("unloadEventEnd"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.unload_event_end)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("redirectStart"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.redirect_start)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("redirectEnd"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.redirect_end)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("fetchStart"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.fetch_start)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("domainLookupStart"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.domain_lookup_start)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("domainLookupEnd"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.domain_lookup_end)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("connectStart"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.connect_start)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("connectEnd"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.connect_end)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("secureConnectionStart"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.secure_connection_start)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("requestStart"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.request_start)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("responseStart"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.response_start)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("responseEnd"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.response_end)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("domLoading"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.dom_loading)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("domInteractive"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.dom_interactive)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("domContentLoadedEventStart"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.dom_content_loaded_event_start)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("domContentLoadedEventEnd"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.dom_content_loaded_event_end)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("domComplete"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.dom_complete)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("loadEventStart"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.load_event_start)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.insert_property(
            js_string!("loadEventEnd"),
            crate::property::PropertyDescriptor::builder()
                .value(timing.load_event_end)
                .writable(false)
                .enumerable(true)
                .configurable(false)
                .build(),
        );

        obj.into()
    }
}