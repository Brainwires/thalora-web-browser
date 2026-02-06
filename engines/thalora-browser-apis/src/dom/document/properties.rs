//! Property getters/setters for Document
//!
//! Simple property accessors: readyState, title, URL, cookie, referrer,
//! domain, characterSet, contentType, visibilityState, hidden, activeElement,
//! currentScript, scrollingElement, body, head

use boa_engine::{
    builtins::BuiltInBuilder,
    object::JsObject,
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    JsString, property::PropertyDescriptorBuilder
};

use super::types::DocumentData;
use super::collections::get_document_element;

/// `Document.prototype.readyState` getter
pub(super) fn get_ready_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.readyState called on non-object")
    })?;

    let value = {
            let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("Document.prototype.readyState called on non-Document object")
            })?;
            document.get_ready_state()
        };
    Ok(JsString::from(value).into())
}

/// `Document.prototype.URL` getter
pub(super) fn get_url(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.URL called on non-object")
    })?;

    let value = {
            let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("Document.prototype.URL called on non-Document object")
            })?;
            document.get_url()
        };
    Ok(JsString::from(value).into())
}

/// `Document.prototype.title` getter
pub(super) fn get_title(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.title called on non-object")
    })?;

    let value = {
            let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("Document.prototype.title called on non-Document object")
            })?;
            document.get_title()
        };
    Ok(JsString::from(value).into())
}

/// `Document.prototype.title` setter
pub(super) fn set_title(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.title setter called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.title setter called on non-Document object")
    })?;

    let title = args.get_or_undefined(0).to_string(context)?;
    document.set_title(&title.to_std_string_escaped());
    Ok(JsValue::undefined())
}

/// `Document.prototype.body` getter
pub(super) fn get_body(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.body called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.body called on non-Document object")
    })?;

    // Create body element if it doesn't exist
    if let Some(body) = document.get_element("body") {
        Ok(body.into())
    } else {
        // Create a new body element using the Element constructor
        let element_constructor = context.intrinsics().constructors().element().constructor();
        let body_element = element_constructor.construct(&[], None, context)?;

        document.add_element("body".to_string(), body_element.clone());
        Ok(body_element.into())
    }
}

/// `Document.prototype.head` getter
pub(super) fn get_head(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.head called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.head called on non-Document object")
    })?;

    // Create head element if it doesn't exist
    if let Some(head) = document.get_element("head") {
        Ok(head.into())
    } else {
        // Create a new head element
        let head_element = JsObject::default(context.intrinsics());

        // Add tagName property
        head_element.define_property_or_throw(
            js_string!("tagName"),
            PropertyDescriptorBuilder::new()
                .configurable(false)
                .enumerable(true)
                .writable(false)
                .value(JsString::from("HEAD"))
                .build(),
            context,
        )?;

        document.add_element("head".to_string(), head_element.clone());
        Ok(head_element.into())
    }
}

/// `Document.prototype.cookie` getter
pub(super) fn get_cookie(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.cookie called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.cookie called on non-Document object")
    })?;

    let cookie = document.cookie.lock().unwrap().clone();
    Ok(JsString::from(cookie).into())
}

/// `Document.prototype.cookie` setter
///
/// Per the spec, `document.cookie = "name=value; Path=/; Secure"` stores ONLY `name=value`.
/// Attributes (Path, Domain, Secure, HttpOnly, SameSite, Expires, Max-Age) are directives
/// to the cookie store, NOT part of the cookie value returned by the getter.
/// Setting `max-age=0` or a negative value deletes the cookie.
pub(super) fn set_cookie(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.cookie setter called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.cookie setter called on non-Document object")
    })?;

    let new_cookie_str = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    // Parse: first part before ';' is the name=value, rest are attributes
    let parts: Vec<&str> = new_cookie_str.split(';').collect();
    let name_value = parts[0].trim();

    // Extract cookie name for merging — must contain '='
    let cookie_name = match name_value.find('=') {
        Some(pos) => &name_value[..pos],
        None => return Ok(JsValue::undefined()), // Invalid cookie string, ignore
    };

    // Check for deletion directives: max-age=0 or max-age=<negative>
    let is_delete = parts.iter().skip(1).any(|attr| {
        let attr = attr.trim().to_lowercase();
        if let Some(rest) = attr.strip_prefix("max-age=") {
            match rest.trim().parse::<i64>() {
                Ok(v) => v <= 0,
                Err(_) => false,
            }
        } else {
            false
        }
    });

    let mut cookies = document.cookie.lock().unwrap();

    if is_delete {
        // Remove this cookie by name
        let existing: Vec<&str> = cookies.split("; ").filter(|c| !c.is_empty()).collect();
        let updated: Vec<&str> = existing.into_iter()
            .filter(|c| {
                // Match by cookie name prefix "name="
                !c.starts_with(&format!("{}=", cookie_name))
            })
            .collect();
        *cookies = updated.join("; ");
    } else {
        // Add or update — store ONLY name=value (strip all attributes)
        if cookies.is_empty() {
            *cookies = name_value.to_string();
        } else {
            let existing: Vec<&str> = cookies.split("; ").filter(|c| !c.is_empty()).collect();
            let mut found = false;
            let mut updated: Vec<String> = Vec::new();
            for cookie in existing {
                if cookie.starts_with(&format!("{}=", cookie_name)) {
                    updated.push(name_value.to_string());
                    found = true;
                } else {
                    updated.push(cookie.to_string());
                }
            }
            if !found {
                updated.push(name_value.to_string());
            }
            *cookies = updated.join("; ");
        }
    }

    Ok(JsValue::undefined())
}

/// `Document.prototype.referrer` getter
pub(super) fn get_referrer(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.referrer called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.referrer called on non-Document object")
    })?;

    let referrer = document.referrer.lock().unwrap().clone();
    Ok(JsString::from(referrer).into())
}

/// `Document.prototype.domain` getter
pub(super) fn get_domain(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.domain called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.domain called on non-Document object")
    })?;

    // Extract domain from URL
    let url = document.get_url();
    let domain = if let Ok(parsed) = url::Url::parse(&url) {
        parsed.host_str().unwrap_or("").to_string()
    } else {
        document.domain.lock().unwrap().clone()
    };
    Ok(JsString::from(domain).into())
}

/// `Document.prototype.characterSet` getter
pub(super) fn get_character_set(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.characterSet called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.characterSet called on non-Document object")
    })?;

    let charset = document.character_set.lock().unwrap().clone();
    Ok(JsString::from(charset).into())
}

/// `Document.prototype.contentType` getter
pub(super) fn get_content_type(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.contentType called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.contentType called on non-Document object")
    })?;

    let content_type = document.content_type.lock().unwrap().clone();
    Ok(JsString::from(content_type).into())
}

/// `Document.prototype.visibilityState` getter
pub(super) fn get_visibility_state(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // In a headless browser, document is always "visible"
    Ok(JsString::from("visible").into())
}

/// `Document.prototype.hidden` getter
pub(super) fn get_hidden(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    // In a headless browser, document is never hidden
    Ok(false.into())
}

/// `Document.prototype.activeElement` getter
/// Returns the currently focused element, or body if nothing is focused
pub(super) fn get_active_element(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.activeElement called on non-object")
    })?;

    let document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.activeElement called on non-Document object")
    })?;

    // Check the focus manager for the currently active element
    if let Some(active) = crate::browser::focus_manager::FocusManager::get_active_element() {
        return Ok(active.into());
    }

    // Per spec: return body as default active element when no element has focus
    if let Some(body) = document.get_element("body") {
        Ok(body.into())
    } else {
        // Create body if it doesn't exist
        let element_constructor = context.intrinsics().constructors().element().constructor();
        let body_element = element_constructor.construct(&[], None, context)?;
        if let Some(elem_data) = body_element.downcast_ref::<crate::dom::element::ElementData>() {
            elem_data.set_tag_name("BODY".to_string());
        }
        document.add_element("body".to_string(), body_element.clone());
        Ok(body_element.into())
    }
}

/// `Document.prototype.currentScript` getter
/// Returns the script element that is currently being executed, or null if no script is executing.
pub(super) fn get_current_script(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Try to get the currently executing script from global state
    // The script execution code should set this when executing a script element
    if let Ok(current) = context.global_object().get(js_string!("__currentScript__"), context) {
        if !current.is_undefined() && !current.is_null() {
            return Ok(current);
        }
    }
    // Per spec, returns null when not inside a script element's execution
    Ok(JsValue::null())
}

/// `Document.prototype.scrollingElement` getter
/// Returns the Element that scrolls the document, typically document.documentElement or document.body
/// https://drafts.csswg.org/cssom-view/#dom-document-scrollingelement
pub(super) fn get_scrolling_element(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // In standards mode, return documentElement (html element)
    // In quirks mode, return body
    // For simplicity, we always return documentElement (standards mode behavior)
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("Document.prototype.scrollingElement called on non-object")
    })?;

    let _document = this_obj.downcast_ref::<DocumentData>().ok_or_else(|| {
        JsNativeError::typ()
            .with_message("Document.prototype.scrollingElement called on non-Document object")
    })?;

    // Return documentElement (html element)
    get_document_element(this, _args, context)
}
