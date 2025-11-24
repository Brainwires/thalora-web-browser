//! PerformanceObserver Web API implementation for Boa
//!
//! Native implementation of the PerformanceObserver standard
//! https://w3c.github.io/performance-timeline/#dom-performanceobserver
//!
//! The PerformanceObserver interface is used to observe performance measurement events
//! and be notified of new performance entries as they are recorded in the browser's
//! performance timeline.

use boa_engine::{
    js_string,
    object::{FunctionObjectBuilder, JsObject, ObjectInitializer},
    property::Attribute,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, NativeFunction,
};

/// JavaScript `PerformanceObserver` implementation.
#[derive(Debug, Copy, Clone)]
pub struct PerformanceObserver;

impl PerformanceObserver {
    /// Initialize the PerformanceObserver constructor in the global scope
    pub fn init(context: &mut Context) {
        // Create the PerformanceObserver constructor function
        let constructor = FunctionObjectBuilder::new(
            context.realm(),
            NativeFunction::from_fn_ptr(Self::constructor),
        )
        .name(js_string!("PerformanceObserver"))
        .length(1)
        .constructor(true)
        .build();

        // Add prototype methods
        let prototype = ObjectInitializer::new(context)
            .function(
                NativeFunction::from_fn_ptr(Self::observe),
                js_string!("observe"),
                1,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::disconnect),
                js_string!("disconnect"),
                0,
            )
            .function(
                NativeFunction::from_fn_ptr(Self::take_records),
                js_string!("takeRecords"),
                0,
            )
            .build();

        // Set prototype on constructor
        constructor
            .set(js_string!("prototype"), prototype.clone(), false, context)
            .expect("Failed to set prototype");

        // Add static supportedEntryTypes property
        let supported_types = Self::create_supported_entry_types(context);
        constructor
            .define_property_or_throw(
                js_string!("supportedEntryTypes"),
                boa_engine::property::PropertyDescriptor::builder()
                    .value(supported_types)
                    .writable(false)
                    .enumerable(false)
                    .configurable(true),
                context,
            )
            .expect("Failed to set supportedEntryTypes");

        // Set constructor on prototype
        prototype
            .set(
                js_string!("constructor"),
                constructor.clone(),
                false,
                context,
            )
            .expect("Failed to set constructor");

        // Register globally
        context
            .register_global_property(
                js_string!("PerformanceObserver"),
                constructor,
                Attribute::WRITABLE | Attribute::CONFIGURABLE,
            )
            .expect("Failed to register PerformanceObserver");
    }

    /// Create the supportedEntryTypes array
    fn create_supported_entry_types(context: &mut Context) -> JsValue {
        let entry_types = vec![
            "mark",
            "measure",
            "navigation",
            "resource",
            "longtask",
            "paint",
            "element",
            "largest-contentful-paint",
            "layout-shift",
            "first-input",
        ];

        let array = boa_engine::object::builtins::JsArray::new(context);
        for (i, entry_type) in entry_types.iter().enumerate() {
            array
                .set(i as u32, js_string!(*entry_type), false, context)
                .expect("Failed to set entry type");
        }

        array.into()
    }

    /// Constructor function for PerformanceObserver
    fn constructor(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Get the callback function (required parameter)
        let callback = args.get_or_undefined(0);
        if !callback.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("PerformanceObserver constructor requires a callback function")
                .into());
        }

        // Create the observer object
        let observer_obj = ObjectInitializer::new(context).build();

        // Store callback on the object
        observer_obj.set(js_string!("__callback__"), callback.clone(), false, context)?;

        // Initialize internal state
        observer_obj.set(
            js_string!("__entryTypes__"),
            boa_engine::object::builtins::JsArray::new(context),
            false,
            context,
        )?;
        observer_obj.set(js_string!("__isObserving__"), false, false, context)?;
        observer_obj.set(
            js_string!("__buffer__"),
            boa_engine::object::builtins::JsArray::new(context),
            false,
            context,
        )?;

        // Add methods
        observer_obj.set(
            js_string!("observe"),
            FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_fn_ptr(Self::observe),
            )
            .length(1)
            .build(),
            false,
            context,
        )?;

        observer_obj.set(
            js_string!("disconnect"),
            FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_fn_ptr(Self::disconnect),
            )
            .length(0)
            .build(),
            false,
            context,
        )?;

        observer_obj.set(
            js_string!("takeRecords"),
            FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_fn_ptr(Self::take_records),
            )
            .length(0)
            .build(),
            false,
            context,
        )?;

        Ok(observer_obj.into())
    }

    /// `PerformanceObserver.prototype.observe()` method
    fn observe(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("PerformanceObserver.observe called on non-object")
        })?;

        let options = args.get_or_undefined(0);

        if !options.is_object() {
            return Err(JsNativeError::typ()
                .with_message("PerformanceObserver.observe: options must be an object")
                .into());
        }

        let options_obj = options.as_object().unwrap();

        // Check for entryTypes or type (but not both per spec)
        let entry_types_val = options_obj.get(js_string!("entryTypes"), context)?;
        let type_val = options_obj.get(js_string!("type"), context)?;

        let has_entry_types = !entry_types_val.is_undefined();
        let has_type = !type_val.is_undefined();

        if has_entry_types && has_type {
            return Err(JsNativeError::typ()
                .with_message(
                    "PerformanceObserver.observe: cannot specify both entryTypes and type",
                )
                .into());
        }

        if !has_entry_types && !has_type {
            return Err(JsNativeError::typ()
                .with_message("PerformanceObserver.observe: must specify entryTypes or type")
                .into());
        }

        // Collect entry types
        let types_array = boa_engine::object::builtins::JsArray::new(context);
        let mut index = 0u32;

        if has_entry_types {
            // entryTypes is an array of strings
            if let Some(arr_obj) = entry_types_val.as_object() {
                let length = arr_obj
                    .get(js_string!("length"), context)?
                    .to_u32(context)?;

                for i in 0..length {
                    let entry_type = arr_obj.get(i, context)?;
                    types_array.set(index, entry_type, false, context)?;
                    index += 1;
                }
            }
        } else if has_type {
            // type is a single string
            types_array.set(0, type_val, false, context)?;
        }

        // Store configuration
        obj.set(js_string!("__entryTypes__"), types_array, false, context)?;
        obj.set(js_string!("__isObserving__"), true, false, context)?;

        Ok(JsValue::undefined())
    }

    /// `PerformanceObserver.prototype.disconnect()` method
    fn disconnect(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("PerformanceObserver.disconnect called on non-object")
        })?;

        // Clear observation state
        obj.set(
            js_string!("__entryTypes__"),
            boa_engine::object::builtins::JsArray::new(context),
            false,
            context,
        )?;
        obj.set(js_string!("__isObserving__"), false, false, context)?;
        obj.set(
            js_string!("__buffer__"),
            boa_engine::object::builtins::JsArray::new(context),
            false,
            context,
        )?;

        Ok(JsValue::undefined())
    }

    /// `PerformanceObserver.prototype.takeRecords()` method
    fn take_records(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("PerformanceObserver.takeRecords called on non-object")
        })?;

        // Get buffered entries and clear buffer
        let buffer = obj.get(js_string!("__buffer__"), context)?;
        if buffer.is_undefined() {
            return Ok(boa_engine::object::builtins::JsArray::new(context).into());
        }

        // Clear buffer
        obj.set(
            js_string!("__buffer__"),
            boa_engine::object::builtins::JsArray::new(context),
            false,
            context,
        )?;

        Ok(buffer)
    }
}

/// A performance entry
#[derive(Debug, Clone)]
pub struct PerformanceEntry {
    /// Name of the entry
    pub name: String,
    /// Type of entry (e.g., "mark", "measure", "resource")
    pub entry_type: String,
    /// Start time in milliseconds
    pub start_time: f64,
    /// Duration in milliseconds
    pub duration: f64,
}

impl PerformanceEntry {
    /// Create a new performance entry
    pub fn new(name: String, entry_type: String, start_time: f64, duration: f64) -> Self {
        Self {
            name,
            entry_type,
            start_time,
            duration,
        }
    }

    /// Convert to a JS object
    pub fn to_js_object(&self, context: &mut Context) -> JsObject {
        ObjectInitializer::new(context)
            .property(
                js_string!("name"),
                js_string!(self.name.as_str()),
                Attribute::default(),
            )
            .property(
                js_string!("entryType"),
                js_string!(self.entry_type.as_str()),
                Attribute::default(),
            )
            .property(
                js_string!("startTime"),
                self.start_time,
                Attribute::default(),
            )
            .property(js_string!("duration"), self.duration, Attribute::default())
            .build()
    }
}

/// PerformanceObserverEntryList - provides access to performance entries
#[derive(Debug, Clone)]
pub struct PerformanceObserverEntryList {
    entries: Vec<PerformanceEntry>,
}

impl PerformanceObserverEntryList {
    /// Create a new entry list
    pub fn new(entries: Vec<PerformanceEntry>) -> Self {
        Self { entries }
    }

    /// Get all entries
    pub fn get_entries(&self) -> &[PerformanceEntry] {
        &self.entries
    }

    /// Get entries by type
    pub fn get_entries_by_type(&self, entry_type: &str) -> Vec<&PerformanceEntry> {
        self.entries
            .iter()
            .filter(|e| e.entry_type == entry_type)
            .collect()
    }

    /// Get entries by name
    pub fn get_entries_by_name(&self, name: &str) -> Vec<&PerformanceEntry> {
        self.entries.iter().filter(|e| e.name == name).collect()
    }
}
