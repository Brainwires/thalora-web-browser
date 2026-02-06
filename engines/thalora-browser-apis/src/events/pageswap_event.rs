//! PageSwapEvent Web API implementation for Boa
//!
//! Native implementation of PageSwapEvent standard (Chrome 124+)
//! https://wicg.github.io/navigation-api/#pageswapevent

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::{Attribute, PropertyDescriptorBuilder}
};
use boa_gc::{Finalize, Trace};

use super::event::EventData;

/// JavaScript `PageSwapEvent` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub(crate) struct PageSwapEvent;

impl IntrinsicObject for PageSwapEvent {
    fn init(realm: &Realm) {
        let activation_func = BuiltInBuilder::callable(realm, get_activation)
            .name(js_string!("get activation"))
            .build();

        let view_transition_func = BuiltInBuilder::callable(realm, get_view_transition)
            .name(js_string!("get viewTransition"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().event().prototype()))
            .accessor(
                js_string!("activation"),
                Some(activation_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("viewTransition"),
                Some(view_transition_func),
                None,
                Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for PageSwapEvent {
    const NAME: JsString = StaticJsStrings::PAGESWAP_EVENT;
}

impl BuiltInConstructor for PageSwapEvent {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 4;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 0;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::pageswap_event;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::pageswap_event,
            context,
        )?;

        let event_type = args.get_or_undefined(0);
        let event_init = args.get_or_undefined(1);

        // Validate event type
        let type_string = event_type.to_string(context)?;

        // Parse eventInitDict
        let mut bubbles = false;
        let mut cancelable = false;
        let mut activation: Option<JsValue> = None;
        let mut view_transition: Option<JsValue> = None;

        if let Some(init_obj) = event_init.as_object() {
            if let Ok(v) = init_obj.get(js_string!("bubbles"), context) {
                bubbles = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("cancelable"), context) {
                cancelable = v.to_boolean();
            }
            if let Ok(v) = init_obj.get(js_string!("activation"), context) {
                if !v.is_null() && !v.is_undefined() {
                    activation = Some(v);
                }
            }
            if let Ok(v) = init_obj.get(js_string!("viewTransition"), context) {
                if !v.is_null() && !v.is_undefined() {
                    view_transition = Some(v);
                }
            }
        }

        let event_data = EventData::new(type_string.to_std_string_escaped(), bubbles, cancelable);
        let pageswap_data = PageSwapEventData::new(event_data, activation, view_transition);

        let pageswap_event = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            pageswap_data,
        );

        Ok(pageswap_event.into())
    }
}

/// Internal data for PageSwapEvent objects - embeds EventData for proper inheritance
#[derive(Debug, Trace, Finalize, JsData)]
pub struct PageSwapEventData {
    /// Base event data
    pub event: EventData,
    activation: Option<JsValue>,
    view_transition: Option<JsValue>,
}

impl PageSwapEventData {
    pub fn new(event: EventData, activation: Option<JsValue>, view_transition: Option<JsValue>) -> Self {
        Self { event, activation, view_transition }
    }

    pub fn get_activation(&self) -> Option<JsValue> {
        self.activation.clone()
    }

    pub fn get_view_transition(&self) -> Option<JsValue> {
        self.view_transition.clone()
    }

    pub fn set_activation(&mut self, activation: Option<JsValue>) {
        self.activation = activation;
    }

    pub fn set_view_transition(&mut self, view_transition: Option<JsValue>) {
        self.view_transition = view_transition;
    }
}

/// `PageSwapEvent.prototype.activation` getter
fn get_activation(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PageSwapEvent.prototype.activation called on non-object")
    })?;

    let pageswap_event = this_obj.downcast_ref::<PageSwapEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("PageSwapEvent.prototype.activation called on non-PageSwapEvent object")
    })?;

    Ok(pageswap_event.get_activation().unwrap_or(JsValue::null()))
}

/// `PageSwapEvent.prototype.viewTransition` getter
fn get_view_transition(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("PageSwapEvent.prototype.viewTransition called on non-object")
    })?;

    let pageswap_event = this_obj.downcast_ref::<PageSwapEventData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("PageSwapEvent.prototype.viewTransition called on non-PageSwapEvent object")
    })?;

    Ok(pageswap_event.get_view_transition().unwrap_or(JsValue::null()))
}

/// Navigation activation entry for PageSwap events
#[derive(Debug, Trace, Finalize, JsData)]
pub struct NavigationActivationData {
    entry: JsValue,
    from: Option<JsValue>,
    #[unsafe_ignore_trace]
    navigation_type: String,
}

impl NavigationActivationData {
    pub fn new(entry: JsValue, from: Option<JsValue>, navigation_type: String) -> Self {
        Self {
            entry,
            from,
            navigation_type,
        }
    }

    pub fn get_entry(&self) -> JsValue {
        self.entry.clone()
    }

    pub fn get_from(&self) -> Option<JsValue> {
        self.from.clone()
    }

    pub fn get_navigation_type(&self) -> String {
        self.navigation_type.clone()
    }
}

/// Create a PageSwap event for navigation
pub fn create_pageswap_event(
    context: &mut Context,
    activation: Option<JsValue>,
    view_transition: Option<JsValue>,
) -> JsResult<JsValue> {
    // Create event init dictionary
    let event_init = JsObject::default(context.intrinsics());

    if let Some(activation_val) = activation.clone() {
        event_init.define_property_or_throw(
            js_string!("activation"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(activation_val)
                .build(),
            context,
        )?;
    }

    if let Some(view_transition_val) = view_transition.clone() {
        event_init.define_property_or_throw(
            js_string!("viewTransition"),
            PropertyDescriptorBuilder::new()
                .configurable(true)
                .enumerable(true)
                .writable(true)
                .value(view_transition_val)
                .build(),
            context,
        )?;
    }

    // Create the event data directly
    let event_data = EventData::new("pageswap".to_string(), false, false);
    let pageswap_data = PageSwapEventData::new(event_data, activation, view_transition);

    let proto = context.intrinsics().constructors().pageswap_event().prototype();
    let pageswap_event = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        proto,
        pageswap_data,
    );

    Ok(pageswap_event.into())
}

/// Dispatch a pageswap event on the window object
pub fn dispatch_pageswap_event(
    context: &mut Context,
    window_obj: &JsObject,
    activation: Option<JsValue>,
    view_transition: Option<JsValue>,
) -> JsResult<()> {
    // Create the pageswap event
    let pageswap_event = create_pageswap_event(context, activation, view_transition)?;

    // Get event listeners for 'pageswap' from window
    if let Ok(listeners_val) = window_obj.get(js_string!("__pageswap_listeners"), context) {
        if listeners_val.is_object() {
            if let Some(listeners_obj) = listeners_val.as_object() {
                // Get the array of listeners
                if let Ok(length_val) = listeners_obj.get(js_string!("length"), context) {
                    if let Some(length) = length_val.as_number() {
                        let len = length as usize;

                        // Call each listener
                        for i in 0..len {
                            if let Ok(listener) = listeners_obj.get(i, context) {
                                if listener.is_callable() {
                                    let _ = listener.as_callable().unwrap().call(
                                        &window_obj.clone().into(),
                                        &[pageswap_event.clone()],
                                        context,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
