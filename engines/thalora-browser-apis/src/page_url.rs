//! The URL of the page loaded into a context, and resolving relative URLs
//! against it (fetch, XMLHttpRequest, EventSource, Request).

use boa_engine::{Context, js_string};
use url::Url;

use crate::browser::location::LocationData;
use crate::browser::window::WindowData;
use crate::dom::document::DocumentData;

/// Record `url` as the page URL: `location.href`, `document.URL` and the
/// window's current URL.
pub fn set_page_url(context: &mut Context, url: &str) {
    let global = context.global_object();
    if let Ok(location) = global.get(js_string!("location"), context)
        && let Some(location) = location.as_object()
        && let Some(mut data) = location.downcast_mut::<LocationData>()
    {
        data.set_href(url.to_string());
    }
    if let Ok(document) = global.get(js_string!("document"), context)
        && let Some(document) = document.as_object()
        && let Some(data) = document.downcast_ref::<DocumentData>()
    {
        data.set_url(url);
    }
    if let Ok(window) = global.get(js_string!("window"), context)
        && let Some(window) = window.as_object()
        && let Some(data) = window.downcast_ref::<WindowData>()
    {
        data.set_current_url(url.to_string());
    }
}

/// The page URL of `context` (`location.href`), if one has been set.
pub fn page_url(context: &mut Context) -> Option<Url> {
    let global = context.global_object();
    let location = global.get(js_string!("location"), context).ok()?;
    let href = location
        .as_object()?
        .downcast_ref::<LocationData>()
        .map(|data| data.href().to_string())?;
    Url::parse(&href).ok().filter(|u| u.scheme() != "about")
}

/// Resolve `input` (absolute or relative) against the page URL, as the URL
/// parser does with the document's base URL. `None` if it can't be parsed.
pub fn resolve_url(context: &mut Context, input: &str) -> Option<Url> {
    match Url::parse(input) {
        Ok(url) => Some(url),
        Err(url::ParseError::RelativeUrlWithoutBase) => page_url(context)?.join(input).ok(),
        Err(_) => None,
    }
}
