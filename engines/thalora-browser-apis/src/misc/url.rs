//! URL Web APIs: URL, URLSearchParams
//!
//! Implementation of the URL Standard APIs
//! https://url.spec.whatwg.org/

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject, FunctionObjectBuilder},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::{Attribute, PropertyDescriptorBuilder},
    NativeFunction,
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;

// ============================================================================
// URL
// ============================================================================

/// JavaScript `URL` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct Url;

impl IntrinsicObject for Url {
    fn init(realm: &Realm) {
        // Getters
        let href_getter = BuiltInBuilder::callable(realm, get_href)
            .name(js_string!("get href"))
            .build();
        let href_setter = BuiltInBuilder::callable(realm, set_href)
            .name(js_string!("set href"))
            .build();

        let origin_getter = BuiltInBuilder::callable(realm, get_origin)
            .name(js_string!("get origin"))
            .build();

        let protocol_getter = BuiltInBuilder::callable(realm, get_protocol)
            .name(js_string!("get protocol"))
            .build();
        let protocol_setter = BuiltInBuilder::callable(realm, set_protocol)
            .name(js_string!("set protocol"))
            .build();

        let host_getter = BuiltInBuilder::callable(realm, get_host)
            .name(js_string!("get host"))
            .build();
        let host_setter = BuiltInBuilder::callable(realm, set_host)
            .name(js_string!("set host"))
            .build();

        let hostname_getter = BuiltInBuilder::callable(realm, get_hostname)
            .name(js_string!("get hostname"))
            .build();
        let hostname_setter = BuiltInBuilder::callable(realm, set_hostname)
            .name(js_string!("set hostname"))
            .build();

        let port_getter = BuiltInBuilder::callable(realm, get_port)
            .name(js_string!("get port"))
            .build();
        let port_setter = BuiltInBuilder::callable(realm, set_port)
            .name(js_string!("set port"))
            .build();

        let pathname_getter = BuiltInBuilder::callable(realm, get_pathname)
            .name(js_string!("get pathname"))
            .build();
        let pathname_setter = BuiltInBuilder::callable(realm, set_pathname)
            .name(js_string!("set pathname"))
            .build();

        let search_getter = BuiltInBuilder::callable(realm, get_search)
            .name(js_string!("get search"))
            .build();
        let search_setter = BuiltInBuilder::callable(realm, set_search)
            .name(js_string!("set search"))
            .build();

        let search_params_getter = BuiltInBuilder::callable(realm, get_search_params)
            .name(js_string!("get searchParams"))
            .build();

        let hash_getter = BuiltInBuilder::callable(realm, get_hash)
            .name(js_string!("get hash"))
            .build();
        let hash_setter = BuiltInBuilder::callable(realm, set_hash)
            .name(js_string!("set hash"))
            .build();

        let username_getter = BuiltInBuilder::callable(realm, get_username)
            .name(js_string!("get username"))
            .build();

        let password_getter = BuiltInBuilder::callable(realm, get_password)
            .name(js_string!("get password"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("href"),
                Some(href_getter),
                Some(href_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("origin"),
                Some(origin_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("protocol"),
                Some(protocol_getter),
                Some(protocol_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("host"),
                Some(host_getter),
                Some(host_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("hostname"),
                Some(hostname_getter),
                Some(hostname_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("port"),
                Some(port_getter),
                Some(port_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("pathname"),
                Some(pathname_getter),
                Some(pathname_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("search"),
                Some(search_getter),
                Some(search_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("searchParams"),
                Some(search_params_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("hash"),
                Some(hash_getter),
                Some(hash_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("username"),
                Some(username_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("password"),
                Some(password_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(to_string, js_string!("toString"), 0)
            .method(to_json, js_string!("toJSON"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Url {
    const NAME: JsString = StaticJsStrings::URL;
}

impl BuiltInConstructor for Url {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::url;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::url,
            context,
        )?;

        let url_string = args.get_or_undefined(0).to_string(context)?;
        let url_str = url_string.to_std_string_escaped();

        // Parse the URL, optionally with a base
        let parsed_url = if let Some(base) = args.get(1) {
            if !base.is_undefined() {
                let base_string = base.to_string(context)?;
                let base_str = base_string.to_std_string_escaped();

                // Try to parse base URL
                let base_parsed = url::Url::parse(&base_str).map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid base URL: {}", e))
                })?;

                // Join with the relative URL
                base_parsed.join(&url_str).map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid URL: {}", e))
                })?
            } else {
                url::Url::parse(&url_str).map_err(|e| {
                    JsNativeError::typ().with_message(format!("Invalid URL: {}", e))
                })?
            }
        } else {
            url::Url::parse(&url_str).map_err(|e| {
                JsNativeError::typ().with_message(format!("Invalid URL: {}", e))
            })?
        };

        let url_data = UrlData::new(parsed_url);

        let url_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            url_data,
        );

        Ok(url_obj.into())
    }
}

/// Internal data for URL objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct UrlData {
    #[unsafe_ignore_trace]
    inner: url::Url,
}

impl UrlData {
    fn new(url: url::Url) -> Self {
        Self { inner: url }
    }
}

// URL getters and setters
fn get_href(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.href called on non-object")
    })?;
    let url_data = this_obj.downcast_ref::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.href called on non-URL object")
    })?;
    Ok(JsString::from(url_data.inner.as_str()).into())
}

fn set_href(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.href setter called on non-object")
    })?;
    let mut url_data = this_obj.downcast_mut::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.href setter called on non-URL object")
    })?;

    let new_href = args.get_or_undefined(0).to_string(context)?;
    let new_url = url::Url::parse(&new_href.to_std_string_escaped()).map_err(|e| {
        JsNativeError::typ().with_message(format!("Invalid URL: {}", e))
    })?;
    url_data.inner = new_url;
    Ok(JsValue::undefined())
}

fn get_origin(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.origin called on non-object")
    })?;
    let url_data = this_obj.downcast_ref::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.origin called on non-URL object")
    })?;
    Ok(JsString::from(url_data.inner.origin().ascii_serialization().as_str()).into())
}

fn get_protocol(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.protocol called on non-object")
    })?;
    let url_data = this_obj.downcast_ref::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.protocol called on non-URL object")
    })?;
    // Protocol includes the trailing colon
    Ok(JsString::from(format!("{}:", url_data.inner.scheme()).as_str()).into())
}

fn set_protocol(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.protocol setter called on non-object")
    })?;
    let mut url_data = this_obj.downcast_mut::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.protocol setter called on non-URL object")
    })?;

    let new_protocol = args.get_or_undefined(0).to_string(context)?;
    let mut protocol_str = new_protocol.to_std_string_escaped();
    // Remove trailing colon if present
    if protocol_str.ends_with(':') {
        protocol_str.pop();
    }
    let _ = url_data.inner.set_scheme(&protocol_str);
    Ok(JsValue::undefined())
}

fn get_host(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.host called on non-object")
    })?;
    let url_data = this_obj.downcast_ref::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.host called on non-URL object")
    })?;
    let host = url_data.inner.host_str().unwrap_or("");
    let port = url_data.inner.port();
    if let Some(p) = port {
        Ok(JsString::from(format!("{}:{}", host, p).as_str()).into())
    } else {
        Ok(JsString::from(host).into())
    }
}

fn set_host(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.host setter called on non-object")
    })?;
    let mut url_data = this_obj.downcast_mut::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.host setter called on non-URL object")
    })?;

    let new_host = args.get_or_undefined(0).to_string(context)?;
    let host_str = new_host.to_std_string_escaped();

    // Parse host:port format
    if let Some(colon_idx) = host_str.rfind(':') {
        let host = &host_str[..colon_idx];
        let port_str = &host_str[colon_idx + 1..];
        let _ = url_data.inner.set_host(Some(host));
        if let Ok(port) = port_str.parse::<u16>() {
            let _ = url_data.inner.set_port(Some(port));
        }
    } else {
        let _ = url_data.inner.set_host(Some(&host_str));
    }
    Ok(JsValue::undefined())
}

fn get_hostname(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.hostname called on non-object")
    })?;
    let url_data = this_obj.downcast_ref::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.hostname called on non-URL object")
    })?;
    Ok(JsString::from(url_data.inner.host_str().unwrap_or("")).into())
}

fn set_hostname(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.hostname setter called on non-object")
    })?;
    let mut url_data = this_obj.downcast_mut::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.hostname setter called on non-URL object")
    })?;

    let new_hostname = args.get_or_undefined(0).to_string(context)?;
    let _ = url_data.inner.set_host(Some(&new_hostname.to_std_string_escaped()));
    Ok(JsValue::undefined())
}

fn get_port(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.port called on non-object")
    })?;
    let url_data = this_obj.downcast_ref::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.port called on non-URL object")
    })?;
    match url_data.inner.port() {
        Some(p) => Ok(JsString::from(p.to_string().as_str()).into()),
        None => Ok(js_string!("").into()),
    }
}

fn set_port(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.port setter called on non-object")
    })?;
    let mut url_data = this_obj.downcast_mut::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.port setter called on non-URL object")
    })?;

    let new_port = args.get_or_undefined(0).to_string(context)?;
    let port_str = new_port.to_std_string_escaped();
    if port_str.is_empty() {
        let _ = url_data.inner.set_port(None);
    } else if let Ok(port) = port_str.parse::<u16>() {
        let _ = url_data.inner.set_port(Some(port));
    }
    Ok(JsValue::undefined())
}

fn get_pathname(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.pathname called on non-object")
    })?;
    let url_data = this_obj.downcast_ref::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.pathname called on non-URL object")
    })?;
    Ok(JsString::from(url_data.inner.path()).into())
}

fn set_pathname(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.pathname setter called on non-object")
    })?;
    let mut url_data = this_obj.downcast_mut::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.pathname setter called on non-URL object")
    })?;

    let new_pathname = args.get_or_undefined(0).to_string(context)?;
    url_data.inner.set_path(&new_pathname.to_std_string_escaped());
    Ok(JsValue::undefined())
}

fn get_search(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.search called on non-object")
    })?;
    let url_data = this_obj.downcast_ref::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.search called on non-URL object")
    })?;
    match url_data.inner.query() {
        Some(q) => Ok(JsString::from(format!("?{}", q).as_str()).into()),
        None => Ok(js_string!("").into()),
    }
}

fn set_search(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.search setter called on non-object")
    })?;
    let mut url_data = this_obj.downcast_mut::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.search setter called on non-URL object")
    })?;

    let new_search = args.get_or_undefined(0).to_string(context)?;
    let mut search_str = new_search.to_std_string_escaped();
    // Remove leading ? if present
    if search_str.starts_with('?') {
        search_str.remove(0);
    }
    if search_str.is_empty() {
        url_data.inner.set_query(None);
    } else {
        url_data.inner.set_query(Some(&search_str));
    }
    Ok(JsValue::undefined())
}

fn get_search_params(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.searchParams called on non-object")
    })?;
    let url_data = this_obj.downcast_ref::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.searchParams called on non-URL object")
    })?;

    // Create a new URLSearchParams object from the query string
    let query = url_data.inner.query().unwrap_or("");
    let params = UrlSearchParamsData::from_query_string(query);

    let prototype = get_prototype_from_constructor(
        &context.intrinsics().constructors().url_search_params().constructor().into(),
        StandardConstructors::url_search_params,
        context,
    )?;

    let params_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        prototype,
        params,
    );

    Ok(params_obj.into())
}

fn get_hash(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.hash called on non-object")
    })?;
    let url_data = this_obj.downcast_ref::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.hash called on non-URL object")
    })?;
    match url_data.inner.fragment() {
        Some(f) => Ok(JsString::from(format!("#{}", f).as_str()).into()),
        None => Ok(js_string!("").into()),
    }
}

fn set_hash(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.hash setter called on non-object")
    })?;
    let mut url_data = this_obj.downcast_mut::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.hash setter called on non-URL object")
    })?;

    let new_hash = args.get_or_undefined(0).to_string(context)?;
    let mut hash_str = new_hash.to_std_string_escaped();
    // Remove leading # if present
    if hash_str.starts_with('#') {
        hash_str.remove(0);
    }
    if hash_str.is_empty() {
        url_data.inner.set_fragment(None);
    } else {
        url_data.inner.set_fragment(Some(&hash_str));
    }
    Ok(JsValue::undefined())
}

fn get_username(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.username called on non-object")
    })?;
    let url_data = this_obj.downcast_ref::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.username called on non-URL object")
    })?;
    Ok(JsString::from(url_data.inner.username()).into())
}

fn get_password(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.password called on non-object")
    })?;
    let url_data = this_obj.downcast_ref::<UrlData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URL.prototype.password called on non-URL object")
    })?;
    Ok(JsString::from(url_data.inner.password().unwrap_or("")).into())
}

fn to_string(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    get_href(this, _args, _context)
}

fn to_json(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    get_href(this, _args, _context)
}

// ============================================================================
// URLSearchParams
// ============================================================================

/// JavaScript `URLSearchParams` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct UrlSearchParams;

impl IntrinsicObject for UrlSearchParams {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(append, js_string!("append"), 2)
            .method(delete_param, js_string!("delete"), 1)
            .method(get_param, js_string!("get"), 1)
            .method(get_all, js_string!("getAll"), 1)
            .method(has_param, js_string!("has"), 1)
            .method(set_param, js_string!("set"), 2)
            .method(sort_params, js_string!("sort"), 0)
            .method(params_to_string, js_string!("toString"), 0)
            .method(entries, js_string!("entries"), 0)
            .method(keys, js_string!("keys"), 0)
            .method(values, js_string!("values"), 0)
            .method(for_each, js_string!("forEach"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for UrlSearchParams {
    const NAME: JsString = StaticJsStrings::URL_SEARCH_PARAMS;
}

impl BuiltInConstructor for UrlSearchParams {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::url_search_params;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::url_search_params,
            context,
        )?;

        let params = if let Some(init) = args.first() {
            if !init.is_undefined() {
                let init_str = init.to_string(context)?;
                let mut query = init_str.to_std_string_escaped();
                // Remove leading ? if present
                if query.starts_with('?') {
                    query.remove(0);
                }
                UrlSearchParamsData::from_query_string(&query)
            } else {
                UrlSearchParamsData::new()
            }
        } else {
            UrlSearchParamsData::new()
        };

        let params_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            params,
        );

        Ok(params_obj.into())
    }
}

/// Internal data for URLSearchParams objects
#[derive(Debug, Trace, Finalize, JsData)]
pub struct UrlSearchParamsData {
    #[unsafe_ignore_trace]
    params: Vec<(String, String)>,
}

impl UrlSearchParamsData {
    fn new() -> Self {
        Self { params: Vec::new() }
    }

    fn from_query_string(query: &str) -> Self {
        let params: Vec<(String, String)> = query
            .split('&')
            .filter(|s| !s.is_empty())
            .map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next().unwrap_or("");
                let value = parts.next().unwrap_or("");
                (
                    urlencoding::decode(key).unwrap_or_else(|_| key.into()).into_owned(),
                    urlencoding::decode(value).unwrap_or_else(|_| value.into()).into_owned(),
                )
            })
            .collect();
        Self { params }
    }
}

// URLSearchParams methods
fn append(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.append called on non-object")
    })?;
    let mut params = this_obj.downcast_mut::<UrlSearchParamsData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.append called on non-URLSearchParams object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    let value = args.get_or_undefined(1).to_string(context)?;

    params.params.push((
        name.to_std_string_escaped(),
        value.to_std_string_escaped(),
    ));

    Ok(JsValue::undefined())
}

fn delete_param(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.delete called on non-object")
    })?;
    let mut params = this_obj.downcast_mut::<UrlSearchParamsData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.delete called on non-URLSearchParams object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    let name_str = name.to_std_string_escaped();

    params.params.retain(|(k, _)| k != &name_str);

    Ok(JsValue::undefined())
}

fn get_param(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.get called on non-object")
    })?;
    let params = this_obj.downcast_ref::<UrlSearchParamsData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.get called on non-URLSearchParams object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    let name_str = name.to_std_string_escaped();

    for (k, v) in &params.params {
        if k == &name_str {
            return Ok(JsString::from(v.as_str()).into());
        }
    }

    Ok(JsValue::null())
}

fn get_all(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.getAll called on non-object")
    })?;
    let params = this_obj.downcast_ref::<UrlSearchParamsData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.getAll called on non-URLSearchParams object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    let name_str = name.to_std_string_escaped();

    let values: Vec<JsValue> = params.params
        .iter()
        .filter(|(k, _)| k == &name_str)
        .map(|(_, v)| JsString::from(v.as_str()).into())
        .collect();

    let array = boa_engine::object::builtins::JsArray::from_iter(values, context);

    Ok(array.into())
}

fn has_param(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.has called on non-object")
    })?;
    let params = this_obj.downcast_ref::<UrlSearchParamsData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.has called on non-URLSearchParams object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    let name_str = name.to_std_string_escaped();

    let exists = params.params.iter().any(|(k, _)| k == &name_str);

    Ok(exists.into())
}

fn set_param(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.set called on non-object")
    })?;
    let mut params = this_obj.downcast_mut::<UrlSearchParamsData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.set called on non-URLSearchParams object")
    })?;

    let name = args.get_or_undefined(0).to_string(context)?;
    let value = args.get_or_undefined(1).to_string(context)?;
    let name_str = name.to_std_string_escaped();
    let value_str = value.to_std_string_escaped();

    // Remove all existing entries with this name
    params.params.retain(|(k, _)| k != &name_str);
    // Add the new entry
    params.params.push((name_str, value_str));

    Ok(JsValue::undefined())
}

fn sort_params(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.sort called on non-object")
    })?;
    let mut params = this_obj.downcast_mut::<UrlSearchParamsData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.sort called on non-URLSearchParams object")
    })?;

    params.params.sort_by(|a, b| a.0.cmp(&b.0));

    Ok(JsValue::undefined())
}

fn params_to_string(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.toString called on non-object")
    })?;
    let params = this_obj.downcast_ref::<UrlSearchParamsData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.toString called on non-URLSearchParams object")
    })?;

    let query: String = params.params
        .iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                urlencoding::encode(k),
                urlencoding::encode(v)
            )
        })
        .collect::<Vec<_>>()
        .join("&");

    Ok(JsString::from(query.as_str()).into())
}

fn entries(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.entries called on non-object")
    })?;
    let params = this_obj.downcast_ref::<UrlSearchParamsData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.entries called on non-URLSearchParams object")
    })?;

    // Create an array of [key, value] pairs
    let entries: Vec<JsValue> = params.params
        .iter()
        .map(|(k, v)| {
            let pair = boa_engine::object::builtins::JsArray::from_iter(
                vec![
                    JsString::from(k.as_str()).into(),
                    JsString::from(v.as_str()).into(),
                ],
                context,
            );
            pair.into()
        })
        .collect();

    let array = boa_engine::object::builtins::JsArray::from_iter(entries, context);

    Ok(array.into())
}

fn keys(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.keys called on non-object")
    })?;
    let params = this_obj.downcast_ref::<UrlSearchParamsData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.keys called on non-URLSearchParams object")
    })?;

    let keys: Vec<JsValue> = params.params
        .iter()
        .map(|(k, _)| JsString::from(k.as_str()).into())
        .collect();

    let array = boa_engine::object::builtins::JsArray::from_iter(keys, context);

    Ok(array.into())
}

fn values(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.values called on non-object")
    })?;
    let params = this_obj.downcast_ref::<UrlSearchParamsData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.values called on non-URLSearchParams object")
    })?;

    let values: Vec<JsValue> = params.params
        .iter()
        .map(|(_, v)| JsString::from(v.as_str()).into())
        .collect();

    let array = boa_engine::object::builtins::JsArray::from_iter(values, context);

    Ok(array.into())
}

fn for_each(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.forEach called on non-object")
    })?;
    let params = this_obj.downcast_ref::<UrlSearchParamsData>().ok_or_else(|| {
        JsNativeError::typ().with_message("URLSearchParams.prototype.forEach called on non-URLSearchParams object")
    })?;

    let callback = args.get_or_undefined(0);
    if !callback.is_callable() {
        return Err(JsNativeError::typ()
            .with_message("URLSearchParams.prototype.forEach: callback is not callable")
            .into());
    }

    let callback_obj = callback.as_object().unwrap();
    let params_clone: Vec<(String, String)> = params.params.clone();

    for (key, value) in params_clone {
        let args = [
            JsString::from(value.as_str()).into(),
            JsString::from(key.as_str()).into(),
            this.clone(),
        ];
        callback_obj.call(&JsValue::undefined(), &args, context)?;
    }

    Ok(JsValue::undefined())
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::{Context, Source};

    fn create_test_context() -> Context {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context)
            .expect("Failed to initialize browser APIs");
        context
    }

    #[test]
    fn test_url_exists() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("typeof URL")).unwrap();
        assert_eq!(result, JsValue::from(js_string!("function")));
    }

    #[test]
    fn test_url_constructor() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            let url = new URL('https://example.com/path?query=1#hash');
            url.href;
        "#)).unwrap();
        assert_eq!(result, JsValue::from(js_string!("https://example.com/path?query=1#hash")));
    }

    #[test]
    fn test_url_properties() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            let url = new URL('https://user:pass@example.com:8080/path?query=1#hash');
            url.protocol === 'https:' &&
            url.hostname === 'example.com' &&
            url.port === '8080' &&
            url.pathname === '/path' &&
            url.search === '?query=1' &&
            url.hash === '#hash';
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_url_search_params_exists() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("typeof URLSearchParams")).unwrap();
        assert_eq!(result, JsValue::from(js_string!("function")));
    }

    #[test]
    fn test_url_search_params_constructor() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            let params = new URLSearchParams('foo=1&bar=2');
            params.get('foo') === '1' && params.get('bar') === '2';
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_url_search_params_methods() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            let params = new URLSearchParams();
            params.append('foo', '1');
            params.append('bar', '2');
            params.has('foo') && params.get('foo') === '1' && params.toString() === 'foo=1&bar=2';
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
