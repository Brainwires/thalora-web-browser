//! Shared DOM event dispatch: listener lookup, listener invocation and the
//! three-phase (capture / at-target / bubble) propagation loop.
//!
//! <https://dom.spec.whatwg.org/#concept-event-dispatch>
//!
//! The propagation path is a list of [`PathEntry`]s, target first. Each entry
//! names the object exposed as `currentTarget` (and used as `this`) and the
//! objects whose listener lists are consulted for it. Listener lists are
//! looked up by downcasting each holder to one of the known listener stores
//! (`ElementData`, `DocumentData`, `WindowData`, `EventTargetData`).
//!
//! Rule: listener lists are snapshotted before any listener runs, and no
//! Mutex / GcRefCell borrow is held while JS is called.

use boa_engine::{Context, JsResult, JsValue, js_string, object::JsObject};

use super::event::{EventData, EventPhase};
use super::event_target::{EventListener, EventTargetData};
use super::ui_events::{
    FocusEventData, InputEventData, KeyboardEventData, MouseEventData, UIEventData,
};
use crate::browser::window::WindowData;
use crate::dom::document::DocumentData;
use crate::dom::element::ElementData;

/// One stop on an event's propagation path.
#[derive(Debug, Clone)]
pub(crate) struct PathEntry {
    /// The object exposed as `event.currentTarget` and passed as `this`.
    pub current_target: JsObject,
    /// Objects whose listener lists are consulted for this stop, in order.
    pub holders: Vec<JsObject>,
}

impl PathEntry {
    /// An entry whose listeners live on the object itself.
    pub(crate) fn single(object: JsObject) -> Self {
        Self {
            holders: vec![object.clone()],
            current_target: object,
        }
    }
}

// ---------------------------------------------------------------------------
// Listener stores
// ---------------------------------------------------------------------------

/// Snapshot of the listeners `holder` has registered for `event_type`.
pub(crate) fn listener_entries(holder: &JsObject, event_type: &str) -> Vec<EventListener> {
    if let Some(element) = holder.downcast_ref::<ElementData>() {
        return element.listener_entries(event_type);
    }
    if let Some(document) = holder.downcast_ref::<DocumentData>() {
        return document.listener_entries(event_type);
    }
    if let Some(window) = holder.downcast_ref::<WindowData>() {
        return window.listener_entries(event_type);
    }
    if let Some(target) = holder.downcast_ref::<EventTargetData>() {
        return target.listener_entries(event_type);
    }
    Vec::new()
}

/// Whether `listener` is still registered on `holder` (it may have been
/// removed by an earlier listener of the same dispatch).
fn is_registered(holder: &JsObject, event_type: &str, listener: &EventListener) -> bool {
    let (callback, capture) = (listener.callback(), listener.capture());
    if let Some(element) = holder.downcast_ref::<ElementData>() {
        return element.has_listener_entry(event_type, callback, capture);
    }
    if let Some(document) = holder.downcast_ref::<DocumentData>() {
        return document.has_listener_entry(event_type, callback, capture);
    }
    if let Some(window) = holder.downcast_ref::<WindowData>() {
        return window.has_listener_entry(event_type, callback);
    }
    if let Some(target) = holder.downcast_ref::<EventTargetData>() {
        return target.has_listener(event_type, callback, capture);
    }
    false
}

/// Remove `listener` from `holder` (used for `once` listeners).
fn remove_listener(holder: &JsObject, event_type: &str, listener: &EventListener) {
    let (callback, capture) = (listener.callback(), listener.capture());
    if let Some(element) = holder.downcast_ref::<ElementData>() {
        element.remove_event_listener_with_capture(event_type, callback, capture);
    } else if let Some(document) = holder.downcast_ref::<DocumentData>() {
        document.remove_event_listener_with_capture(event_type, callback, capture);
    } else if let Some(window) = holder.downcast_ref::<WindowData>() {
        window.remove_event_listener(event_type, callback);
    } else if let Some(target) = holder.downcast_ref::<EventTargetData>() {
        target.remove_event_listener(event_type, callback, capture);
    }
}

/// The path entry for the global window, if one can be found.
///
/// `window.addEventListener` stores on the `window` object (`WindowData`),
/// while the bare global `addEventListener` stores on the hidden
/// `__globalEventTarget__` (`EventTargetData`); both are consulted.
pub(crate) fn window_path_entry(context: &mut Context) -> Option<PathEntry> {
    let global = context.global_object();
    let mut holders = Vec::new();
    let mut current_target = global.clone();

    if let Ok(window) = global.get(js_string!("window"), context)
        && let Some(window) = window.as_object()
    {
        let is_window = window.downcast_ref::<WindowData>().is_some();
        if is_window {
            holders.push(window.clone());
            current_target = window;
        }
    }
    if let Ok(target) = global.get(js_string!("__globalEventTarget__"), context)
        && let Some(target) = target.as_object()
    {
        let is_target = target.downcast_ref::<EventTargetData>().is_some();
        if is_target {
            holders.push(target);
        }
    }

    if holders.is_empty() {
        return None;
    }
    Some(PathEntry {
        current_target,
        holders,
    })
}

// ---------------------------------------------------------------------------
// Event state
// ---------------------------------------------------------------------------

/// Run `f` on the `EventData` backing `event` (directly, or nested inside a
/// UI event subclass). Returns `None` for objects without event data.
fn with_event_data<R>(event: &JsObject, f: impl FnOnce(&mut EventData) -> R) -> Option<R> {
    if let Some(mut data) = event.downcast_mut::<EventData>() {
        return Some(f(&mut *data));
    }
    if let Some(mut data) = event.downcast_mut::<UIEventData>() {
        return Some(f(&mut data.event));
    }
    if let Some(mut data) = event.downcast_mut::<MouseEventData>() {
        return Some(f(&mut data.ui_event.event));
    }
    if let Some(mut data) = event.downcast_mut::<KeyboardEventData>() {
        return Some(f(&mut data.ui_event.event));
    }
    if let Some(mut data) = event.downcast_mut::<FocusEventData>() {
        return Some(f(&mut data.ui_event.event));
    }
    if let Some(mut data) = event.downcast_mut::<InputEventData>() {
        return Some(f(&mut data.ui_event.event));
    }
    None
}

/// Whether `event`'s `target` / `currentTarget` / `eventPhase` are served by
/// `Event.prototype` accessors over `EventData`. Other objects (CustomEvent,
/// UI event subclasses, plain objects) get them as own properties.
fn has_event_accessors(event: &JsObject) -> bool {
    event.downcast_ref::<EventData>().is_some()
}

/// The event's `type`.
pub(crate) fn event_type_of(event: &JsObject, context: &mut Context) -> JsResult<String> {
    if let Some(event_type) = with_event_data(event, |d| d.get_type().to_string()) {
        return Ok(event_type);
    }
    let value = event.get(js_string!("type"), context)?;
    if value.is_undefined() {
        return Ok(String::new());
    }
    Ok(value.to_string(context)?.to_std_string_escaped())
}

fn bubbles_of(event: &JsObject, context: &mut Context) -> bool {
    if let Some(bubbles) = with_event_data(event, |d| d.get_bubbles()) {
        return bubbles;
    }
    event
        .get(js_string!("bubbles"), context)
        .map(|v| v.to_boolean())
        .unwrap_or(false)
}

fn default_prevented_of(event: &JsObject, context: &mut Context) -> bool {
    if let Some(prevented) = with_event_data(event, |d| d.get_default_prevented()) {
        return prevented;
    }
    event
        .get(js_string!("defaultPrevented"), context)
        .map(|v| v.to_boolean())
        .unwrap_or(false)
}

fn propagation_stopped(event: &JsObject) -> bool {
    with_event_data(event, |d| d.should_stop_propagation()).unwrap_or(false)
}

fn immediate_propagation_stopped(event: &JsObject) -> bool {
    with_event_data(event, |d| d.should_stop_immediate_propagation()).unwrap_or(false)
}

fn phase_number(phase: &EventPhase) -> i32 {
    match phase {
        EventPhase::None => 0,
        EventPhase::CapturingPhase => 1,
        EventPhase::AtTarget => 2,
        EventPhase::BubblingPhase => 3,
    }
}

/// Set `eventPhase` and `currentTarget` on the event.
fn set_phase(
    event: &JsObject,
    accessors: bool,
    phase: EventPhase,
    current_target: Option<&JsObject>,
    context: &mut Context,
) {
    let number = phase_number(&phase);
    let current = current_target.cloned();
    with_event_data(event, |d| {
        d.set_phase(phase);
        d.set_current_target(current);
    });
    if !accessors {
        let current: JsValue = current_target.map_or(JsValue::null(), |o| o.clone().into());
        let _ = event.set(js_string!("eventPhase"), number, false, context);
        let _ = event.set(js_string!("currentTarget"), current, false, context);
    }
}

// ---------------------------------------------------------------------------
// Invocation
// ---------------------------------------------------------------------------

/// Call one listener: a function with `this` = `current_target`, or an
/// object's `handleEvent` with `this` = the object. Exceptions are swallowed
/// (the spec reports them and continues with the next listener).
fn call_listener(
    callback: &JsValue,
    current_target: &JsObject,
    event: &JsObject,
    context: &mut Context,
) {
    let args = [JsValue::from(event.clone())];
    if let Some(function) = callback.as_callable() {
        let this: JsValue = current_target.clone().into();
        let _ = function.call(&this, &args, context);
    } else if let Some(object) = callback.as_object()
        && let Ok(handler) = object.get(js_string!("handleEvent"), context)
        && let Some(handler) = handler.as_callable()
    {
        let _ = handler.call(callback, &args, context);
    }
}

/// Invoke the listeners of one path entry for `phase`.
///
/// * capture phase: only capture listeners;
/// * at target: every listener, in registration order;
/// * bubble phase: only non-capture listeners.
///
/// `once` listeners are removed before they run; listeners removed by an
/// earlier listener of this dispatch are skipped; `stopImmediatePropagation`
/// ends the loop. The caller sets `eventPhase` / `currentTarget` first.
pub(crate) fn invoke_listeners(
    entry: &PathEntry,
    event: &JsObject,
    event_type: &str,
    phase: &EventPhase,
    context: &mut Context,
) -> JsResult<()> {
    for holder in &entry.holders {
        let listeners = listener_entries(holder, event_type);
        for listener in listeners {
            let wanted = match phase {
                EventPhase::CapturingPhase => listener.capture(),
                EventPhase::BubblingPhase => !listener.capture(),
                EventPhase::AtTarget => true,
                EventPhase::None => false,
            };
            if !wanted || !is_registered(holder, event_type, &listener) {
                continue;
            }
            if listener.once() {
                remove_listener(holder, event_type, &listener);
            }
            call_listener(listener.callback(), &entry.current_target, event, context);
            if immediate_propagation_stopped(event) {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// Dispatch `event` along `path` (target first, then ancestors nearest
/// first, then document / window entries) with full capture, at-target and
/// bubble phases. Returns `!defaultPrevented`.
pub(crate) fn dispatch_along_path(
    event: &JsObject,
    path: &[PathEntry],
    context: &mut Context,
) -> JsResult<bool> {
    let Some(target_entry) = path.first() else {
        return Ok(true);
    };
    let target = target_entry.current_target.clone();
    let event_type = event_type_of(event, context)?;
    let bubbles = bubbles_of(event, context);
    let accessors = has_event_accessors(event);
    let ancestors = &path[1..];

    with_event_data(event, |d| d.set_target(Some(target.clone())));
    if !accessors {
        let _ = event.set(js_string!("target"), target.clone(), false, context);
    }

    // Capture: outermost ancestor down to the target's parent.
    for entry in ancestors.iter().rev() {
        if propagation_stopped(event) {
            break;
        }
        set_phase(
            event,
            accessors,
            EventPhase::CapturingPhase,
            Some(&entry.current_target),
            context,
        );
        invoke_listeners(
            entry,
            event,
            &event_type,
            &EventPhase::CapturingPhase,
            context,
        )?;
    }

    // At target: every listener.
    if !propagation_stopped(event) {
        set_phase(
            event,
            accessors,
            EventPhase::AtTarget,
            Some(&target),
            context,
        );
        invoke_listeners(
            target_entry,
            event,
            &event_type,
            &EventPhase::AtTarget,
            context,
        )?;
    }

    // Bubble: the target's parent up to the outermost ancestor.
    if bubbles {
        for entry in ancestors {
            if propagation_stopped(event) {
                break;
            }
            set_phase(
                event,
                accessors,
                EventPhase::BubblingPhase,
                Some(&entry.current_target),
                context,
            );
            invoke_listeners(
                entry,
                event,
                &event_type,
                &EventPhase::BubblingPhase,
                context,
            )?;
        }
    }

    set_phase(event, accessors, EventPhase::None, None, context);
    with_event_data(event, |d| d.clear_propagation_flags());

    Ok(!default_prevented_of(event, context))
}
