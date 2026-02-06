//! Dispatch helpers for Element.prototype methods
//!
//! These helpers enable Element.prototype property getters/setters to work on
//! specialized element types (HTMLIFrameElement, HTMLScriptElement, etc.) by
//! dispatching across all known element data types that embed ElementData.

use boa_engine::{
    object::JsObject,
    JsNativeError, JsResult,
};

use super::types::ElementData;
use crate::dom::html_iframe_element::HTMLIFrameElementData;
use crate::dom::html_script_element::HTMLScriptElementData;

/// Execute a closure with `&ElementData`, dispatching across all element types.
///
/// Tries `ElementData` first (most common), then specialized types that embed it.
/// The `GcRef` guard from `downcast_ref` stays alive for the duration of `f()`.
pub(crate) fn with_element_data<T>(
    obj: &JsObject,
    f: impl FnOnce(&ElementData) -> T,
    error_msg: &'static str,
) -> JsResult<T> {
    if let Some(el) = obj.downcast_ref::<ElementData>() {
        Ok(f(&*el))
    } else if let Some(iframe) = obj.downcast_ref::<HTMLIFrameElementData>() {
        Ok(f(iframe.element_data()))
    } else if let Some(script) = obj.downcast_ref::<HTMLScriptElementData>() {
        Ok(f(script.element_data()))
    } else {
        Err(JsNativeError::typ().with_message(error_msg).into())
    }
}

/// Check if a JsObject has element data (any element type).
///
/// Returns `true` for plain ElementData, HTMLIFrameElementData, HTMLScriptElementData,
/// or any other specialized element type that embeds ElementData.
pub(crate) fn has_element_data(obj: &JsObject) -> bool {
    obj.downcast_ref::<ElementData>().is_some()
        || obj.downcast_ref::<HTMLIFrameElementData>().is_some()
        || obj.downcast_ref::<HTMLScriptElementData>().is_some()
}
