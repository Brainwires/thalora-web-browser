//! ResizeObserver Web API implementation for Boa
//!
//! Native implementation of the ResizeObserver standard
//! https://wicg.github.io/ResizeObserver/
//!
//! This implements the complete ResizeObserver interface for observing changes in element size

use boa_engine::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject, ObjectInitializer},
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    realm::Realm, JsData, JsString, property::Attribute
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;

/// JavaScript `ResizeObserver` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub struct ResizeObserver;

impl IntrinsicObject for ResizeObserver {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::observe, js_string!("observe"), 1)
            .method(Self::unobserve, js_string!("unobserve"), 1)
            .method(Self::disconnect, js_string!("disconnect"), 0)
            .method(Self::trigger_resize, js_string!("triggerResize"), 2)
            .method(Self::deliver_entries, js_string!("deliverEntries"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for ResizeObserver {
    const NAME: JsString = js_string!("ResizeObserver");
}

impl BuiltInConstructor for ResizeObserver {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::resize_observer;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::resize_observer,
            context,
        )?;

        // Get the callback function (required parameter)
        let callback = args.get_or_undefined(0);
        if !callback.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("ResizeObserver constructor requires a callback function")
                .into());
        }

        // Create ResizeObserver data
        let observer_data = ResizeObserverData {
            callback: callback.clone(),
            observed_targets: HashMap::new(),
            entries: Vec::new(),
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

impl ResizeObserver {
    /// `ResizeObserver.prototype.observe()` method
    fn observe(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ResizeObserver.observe called on non-object")
        })?;

        let target = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // Validate target (should be an Element, but for now accept any object)
        if !target.is_object() {
            return Err(JsNativeError::typ()
                .with_message("ResizeObserver.observe: target must be an Element")
                .into());
        }

        // Parse options (ResizeObserverOptions)
        let mut box_type = ResizeObserverBoxOptions::ContentBox;

        if let Some(options_obj) = options.as_object() {
            if let Ok(box_option) = options_obj.get(js_string!("box"), context) {
                let box_str = box_option.to_string(context)
                    .map(|s| s.to_std_string_escaped())
                    .unwrap_or_else(|_| "content-box".to_string());

                box_type = match box_str.as_str() {
                    "border-box" => ResizeObserverBoxOptions::BorderBox,
                    "content-box" => ResizeObserverBoxOptions::ContentBox,
                    "device-pixel-content-box" => ResizeObserverBoxOptions::DevicePixelContentBox,
                    _ => ResizeObserverBoxOptions::ContentBox,
                };
            }
        }

        // Update observer data
        if let Some(mut observer_data) = observer_obj.downcast_mut::<ResizeObserverData>() {
            let target_id = format!("{:p}", target.as_object().unwrap().as_ref());
            observer_data.observed_targets.insert(target_id, (target.clone(), box_type));
            observer_data.is_observing = true;

            // In a real implementation, we would start observing the element's size changes
            // For now, we'll just track that it's being observed
        }

        Ok(JsValue::undefined())
    }

    /// `ResizeObserver.prototype.unobserve()` method
    fn unobserve(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ResizeObserver.unobserve called on non-object")
        })?;

        let target = args.get_or_undefined(0);

        if !target.is_object() {
            return Err(JsNativeError::typ()
                .with_message("ResizeObserver.unobserve: target must be an Element")
                .into());
        }

        if let Some(mut observer_data) = observer_obj.downcast_mut::<ResizeObserverData>() {
            let target_id = format!("{:p}", target.as_object().unwrap().as_ref());
            observer_data.observed_targets.remove(&target_id);

            if observer_data.observed_targets.is_empty() {
                observer_data.is_observing = false;
            }
        }

        Ok(JsValue::undefined())
    }

    /// `ResizeObserver.prototype.disconnect()` method
    fn disconnect(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ResizeObserver.disconnect called on non-object")
        })?;

        if let Some(mut observer_data) = observer_obj.downcast_mut::<ResizeObserverData>() {
            observer_data.observed_targets.clear();
            observer_data.entries.clear();
            observer_data.is_observing = false;
        }

        Ok(JsValue::undefined())
    }

    /// `ResizeObserver.prototype.triggerResize()` method
    ///
    /// This is a Thalora-specific extension that allows programmatic triggering of resize events
    /// in a headless browser context where there's no real DOM layout.
    ///
    /// Usage: observer.triggerResize(target, { width: 100, height: 200 })
    fn trigger_resize(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ResizeObserver.triggerResize called on non-object")
        })?;

        let target = args.get_or_undefined(0);
        let dimensions = args.get_or_undefined(1);

        if !target.is_object() {
            return Err(JsNativeError::typ()
                .with_message("ResizeObserver.triggerResize: target must be an Element")
                .into());
        }

        // Extract dimensions from the options object
        let (width, height, x, y) = if let Some(dim_obj) = dimensions.as_object() {
            let width = dim_obj.get(js_string!("width"), context)
                .ok()
                .and_then(|v| v.as_number())
                .unwrap_or(0.0);
            let height = dim_obj.get(js_string!("height"), context)
                .ok()
                .and_then(|v| v.as_number())
                .unwrap_or(0.0);
            let x = dim_obj.get(js_string!("x"), context)
                .ok()
                .and_then(|v| v.as_number())
                .unwrap_or(0.0);
            let y = dim_obj.get(js_string!("y"), context)
                .ok()
                .and_then(|v| v.as_number())
                .unwrap_or(0.0);
            (width, height, x, y)
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

        // Check if this target is being observed
        let target_id = format!("{:p}", target.as_object().unwrap().as_ref());

        if let Some(mut observer_data) = observer_obj.downcast_mut::<ResizeObserverData>() {
            if !observer_data.observed_targets.contains_key(&target_id) {
                // Target is not being observed, do nothing
                return Ok(JsValue::undefined());
            }

            // Create a resize entry
            let entry = ResizeObserverEntry {
                target: target_id.clone(),
                target_value: target.clone(),
                content_rect: DOMRectReadOnly {
                    x,
                    y,
                    width,
                    height,
                    top: y,
                    right: x + width,
                    bottom: y + height,
                    left: x,
                },
                border_box_size: vec![ResizeObserverSize {
                    inline_size: width,
                    block_size: height,
                }],
                content_box_size: vec![ResizeObserverSize {
                    inline_size: width,
                    block_size: height,
                }],
                device_pixel_content_box_size: vec![ResizeObserverSize {
                    inline_size: width,
                    block_size: height,
                }],
            };

            observer_data.entries.push(entry);
        }

        Ok(JsValue::undefined())
    }

    /// `ResizeObserver.prototype.deliverEntries()` method
    ///
    /// Delivers any pending resize entries to the callback.
    /// This is typically called by the event loop but can be triggered manually.
    fn deliver_entries(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let observer_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("ResizeObserver.deliverEntries called on non-object")
        })?;

        // Extract entries and callback
        let (entries, callback) = {
            if let Some(mut observer_data) = observer_obj.downcast_mut::<ResizeObserverData>() {
                let entries = std::mem::take(&mut observer_data.entries);
                let callback = observer_data.callback.clone();
                (entries, callback)
            } else {
                return Err(JsNativeError::typ()
                    .with_message("ResizeObserver.deliverEntries called on non-ResizeObserver")
                    .into());
            }
        };

        if entries.is_empty() {
            return Ok(JsValue::undefined());
        }

        // Convert entries to JavaScript objects
        let js_entries = boa_engine::object::JsArray::new(context);
        for (i, entry) in entries.iter().enumerate() {
            let entry_obj = create_resize_observer_entry(entry, context)?;
            js_entries.set(i as u32, JsValue::from(entry_obj), false, context)?;
        }

        // Call the callback with (entries, observer)
        if let Some(func) = callback.as_callable() {
            func.call(&JsValue::undefined(), &[js_entries.into(), this.clone()], context)?;
        }

        Ok(JsValue::undefined())
    }
}

/// Internal data for ResizeObserver instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct ResizeObserverData {
    /// The callback function to call when size changes occur
    callback: JsValue,
    /// Map of observed target elements and their box options
    #[unsafe_ignore_trace]
    observed_targets: HashMap<String, (JsValue, ResizeObserverBoxOptions)>,
    /// Queue of resize entries waiting to be delivered
    #[unsafe_ignore_trace]
    entries: Vec<ResizeObserverEntry>,
    /// Whether this observer is currently observing any targets
    #[unsafe_ignore_trace]
    is_observing: bool,
}

/// Box options for ResizeObserver
#[derive(Debug, Clone, Copy)]
pub enum ResizeObserverBoxOptions {
    /// Observe changes to the content box (default)
    ContentBox,
    /// Observe changes to the border box
    BorderBox,
    /// Observe changes to the device pixel content box
    DevicePixelContentBox,
}

/// Represents a single resize observer entry
#[derive(Debug)]
pub struct ResizeObserverEntry {
    /// The target element being observed (ID for internal tracking)
    pub target: String,
    /// The actual JsValue target
    pub target_value: JsValue,
    /// The new content rect
    pub content_rect: DOMRectReadOnly,
    /// The new border box size
    pub border_box_size: Vec<ResizeObserverSize>,
    /// The new content box size
    pub content_box_size: Vec<ResizeObserverSize>,
    /// The new device pixel content box size
    pub device_pixel_content_box_size: Vec<ResizeObserverSize>,
}

/// Represents a resize observer size
#[derive(Debug, Clone)]
pub struct ResizeObserverSize {
    /// The inline size (width in horizontal writing mode)
    pub inline_size: f64,
    /// The block size (height in horizontal writing mode)
    pub block_size: f64,
}

/// Read-only rectangle representation
#[derive(Debug, Clone)]
pub struct DOMRectReadOnly {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}

/// Create a JavaScript ResizeObserverEntry object from a Rust entry
fn create_resize_observer_entry(entry: &ResizeObserverEntry, context: &mut Context) -> JsResult<JsObject> {
    // Create contentRect object
    let content_rect = ObjectInitializer::new(context)
        .property(js_string!("x"), entry.content_rect.x, Attribute::all())
        .property(js_string!("y"), entry.content_rect.y, Attribute::all())
        .property(js_string!("width"), entry.content_rect.width, Attribute::all())
        .property(js_string!("height"), entry.content_rect.height, Attribute::all())
        .property(js_string!("top"), entry.content_rect.top, Attribute::all())
        .property(js_string!("right"), entry.content_rect.right, Attribute::all())
        .property(js_string!("bottom"), entry.content_rect.bottom, Attribute::all())
        .property(js_string!("left"), entry.content_rect.left, Attribute::all())
        .build();

    // Create borderBoxSize array
    let border_box_size = create_size_array(&entry.border_box_size, context)?;

    // Create contentBoxSize array
    let content_box_size = create_size_array(&entry.content_box_size, context)?;

    // Create devicePixelContentBoxSize array
    let device_pixel_content_box_size = create_size_array(&entry.device_pixel_content_box_size, context)?;

    // Create the ResizeObserverEntry object
    let entry_obj = ObjectInitializer::new(context)
        .property(js_string!("target"), entry.target_value.clone(), Attribute::all())
        .property(js_string!("contentRect"), content_rect, Attribute::all())
        .property(js_string!("borderBoxSize"), border_box_size, Attribute::all())
        .property(js_string!("contentBoxSize"), content_box_size, Attribute::all())
        .property(js_string!("devicePixelContentBoxSize"), device_pixel_content_box_size, Attribute::all())
        .build();

    Ok(entry_obj)
}

/// Create an array of ResizeObserverSize objects
fn create_size_array(sizes: &[ResizeObserverSize], context: &mut Context) -> JsResult<JsObject> {
    let array = boa_engine::object::JsArray::new(context);

    for (i, size) in sizes.iter().enumerate() {
        let size_obj = ObjectInitializer::new(context)
            .property(js_string!("inlineSize"), size.inline_size, Attribute::all())
            .property(js_string!("blockSize"), size.block_size, Attribute::all())
            .build();
        array.set(i as u32, JsValue::from(size_obj), false, context)?;
    }

    Ok(array.into())
}