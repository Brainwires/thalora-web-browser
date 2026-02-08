//! Fetch Web API implementation for Boa
//!
//! Native implementation of the Fetch standard
//! https://fetch.spec.whatwg.org/
//!
//! This implements the complete Fetch interface with real HTTP networking


use boa_engine::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor, json::Json},
    object::{JsObject, builtins::JsPromise, PROTOTYPE, internal_methods::get_prototype_from_constructor},
    value::JsValue,
    Context, JsArgs, JsNativeError, JsResult, js_string,
    realm::Realm, JsData, JsString,
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    property::Attribute,
    job::NativeAsyncJob
};
use boa_gc::{Finalize, Trace};
use std::collections::HashMap;
use rquest;
use url::Url;

/// JavaScript `fetch()` global function implementation.
#[derive(Debug, Copy, Clone)]
pub struct Fetch;

impl IntrinsicObject for Fetch {
    fn init(realm: &Realm) {
        BuiltInBuilder::callable_with_intrinsic::<Self>(realm, fetch)
            .name(js_string!("fetch"))
            .length(1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        // Return the intrinsic `fetch` object stored in the intrinsics.
        intrinsics.objects().fetch().into()
    }
}

impl BuiltInObject for Fetch {
    const NAME: JsString = js_string!("fetch");
}

/// Get the base URL from the JS context by reading window.location.href
/// Used by fetch() and XMLHttpRequest to resolve relative URLs
pub(crate) fn get_base_url_from_context(context: &mut Context) -> Option<String> {
    let global = context.global_object();
    let location = global.get(js_string!("location"), context).ok()?;
    let loc_obj = location.as_object()?;
    let href = loc_obj.get(js_string!("href"), context).ok()?;
    let href_str = href.to_string(context).ok()?.to_std_string_escaped();
    if href_str.is_empty() || href_str == "undefined" || href_str == "about:blank" {
        None
    } else {
        Some(href_str)
    }
}

/// `fetch(input, init)` global function
fn fetch(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let input = args.get_or_undefined(0);
    let init = args.get_or_undefined(1);

    // Parse input (URL or Request object)
    let url_string = if let Some(request_obj) = input.as_object() {
        // If it's a Request object, get its URL
        if let Some(request_data) = request_obj.downcast_ref::<RequestData>() {
            request_data.url.clone()
        } else {
            // Otherwise convert to string
            input.to_string(context)?.to_std_string_escaped()
        }
    } else {
        input.to_string(context)?.to_std_string_escaped()
    };

    // Resolve URL — support relative URLs by resolving against current page origin
    let url_string = match Url::parse(&url_string) {
        Ok(_) => url_string, // Already absolute
        Err(_) => {
            // Try to resolve relative URL against window.location.href
            let base_url = get_base_url_from_context(context);
            if let Some(base) = base_url {
                if let Ok(base_parsed) = Url::parse(&base) {
                    if let Ok(resolved) = base_parsed.join(&url_string) {
                        resolved.to_string()
                    } else {
                        return Err(JsNativeError::typ()
                            .with_message(format!("Invalid URL: {}", url_string))
                            .into());
                    }
                } else {
                    return Err(JsNativeError::typ()
                        .with_message(format!("Invalid URL: {}", url_string))
                        .into());
                }
            } else {
                return Err(JsNativeError::typ()
                    .with_message(format!("Invalid URL: {} (no base URL available)", url_string))
                    .into());
            }
        }
    };

    // Parse init options
    let (method, headers, body) = if !init.is_undefined() {
        parse_fetch_init(init, context)?
    } else {
        ("GET".to_string(), HashMap::new(), None)
    };

    // Create a new pending Promise and return it immediately
    let (promise, resolvers) = JsPromise::new_pending(context);

    eprintln!("DEBUG: fetch() called: {} {} (enqueuing NativeAsyncJob)", method, url_string);

    // Enqueue an async job to perform the actual HTTP request.
    // IMPORTANT: Boa's SimpleJobExecutor uses futures_lite::block_on which does NOT
    // provide a tokio runtime. Pure `.await` on HTTP clients returns Poll::Pending
    // forever because there's no tokio executor to drive the I/O. We use
    // block_on_compat() which spawns a thread with its own tokio runtime.
    context.enqueue_job(
        NativeAsyncJob::new(async move |ctx_ref| {
            // Save URL for the Response object (url_string is moved into block_on_compat)
            let url_for_response = url_string.clone();

            // Execute HTTP request synchronously via block_on_compat.
            // This spawns a thread with its own tokio runtime so the .await
            // on send()/text() actually executes instead of pending forever.
            let http_result = crate::http_blocking::block_on_compat(async move {
                let client = crate::http_blocking::get_shared_client();
                let mut request_builder = client.request(
                    rquest::Method::from_bytes(method.as_bytes()).unwrap_or(rquest::Method::GET),
                    &url_string
                );

                // Add headers
                for (key, value) in headers {
                    request_builder = request_builder.header(&key, &value);
                }

                // Add body if present
                if let Some(body_content) = body {
                    request_builder = request_builder.body(body_content);
                }

                let response = request_builder.send().await
                    .map_err(|e| format!("Fetch request failed: {}", e))?;

                let status = response.status().as_u16();
                let status_text = response.status().canonical_reason().unwrap_or("").to_string();

                let mut response_headers = HashMap::new();
                for (name, value) in response.headers() {
                    if let Ok(value_str) = value.to_str() {
                        response_headers.insert(name.to_string(), value_str.to_string());
                    }
                }

                let body_text = response.text().await
                    .map_err(|e| format!("Failed to read response body: {}", e))?;

                Ok::<_, String>((status, status_text, response_headers, body_text))
            });

            // Resolve or reject the promise with the HTTP result
            let context = &mut ctx_ref.borrow_mut();
            match http_result {
                Ok((status, status_text, response_headers, body_text)) => {
                    let response_data = ResponseData {
                        body: Some(body_text),
                        status,
                        status_text: status_text.clone(),
                        headers: response_headers,
                        url: url_for_response.clone(),
                    };

                    let prototype = context.intrinsics().constructors().response().prototype();
                    let response_obj = JsObject::from_proto_and_data(Some(prototype), response_data);

                    drop(response_obj.set(js_string!("status"), JsValue::from(status), false, context));
                    drop(response_obj.set(js_string!("statusText"), JsValue::from(js_string!(status_text)), false, context));
                    drop(response_obj.set(js_string!("ok"), JsValue::from(status >= 200 && status < 300), false, context));
                    drop(response_obj.set(js_string!("url"), JsValue::from(js_string!(url_for_response)), false, context));

                    resolvers.resolve.call(&JsValue::undefined(), &[response_obj.into()], context)
                }
                Err(error_msg) => {
                    let error = JsNativeError::typ()
                        .with_message(error_msg)
                        .into_opaque(context);
                    resolvers.reject.call(&JsValue::undefined(), &[error.into()], context)
                }
            }
        })
        .into(),
    );

    // Return the Promise immediately
    Ok(promise.into())
}

/// Parse fetch init options
fn parse_fetch_init(
    init: &JsValue,
    context: &mut Context,
) -> JsResult<(String, HashMap<String, String>, Option<String>)> {
    let init_obj = init.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("fetch init must be an object")
    })?;

    // Method — default to GET when missing or undefined/null
    let method = if let Ok(method_val) = init_obj.get(js_string!("method"), context) {
        if method_val.is_undefined() || method_val.is_null() {
            "GET".to_string()
        } else {
            method_val.to_string(context)?.to_std_string_escaped().to_uppercase()
        }
    } else {
        "GET".to_string()
    };

    // Headers
    let mut headers = HashMap::new();
    if let Ok(headers_val) = init_obj.get(js_string!("headers"), context) {
        if let Some(headers_obj) = headers_val.as_object() {
            // Parse headers object - could be Headers object or plain object
            if let Some(headers_data) = headers_obj.downcast_ref::<HeadersData>() {
                // It's a Headers object
                headers.extend(headers_data.headers.clone());
            } else {
                // It's a plain object, iterate over properties
                for property_key in headers_obj.own_property_keys(context)? {
                    let key_name = property_key.to_string();
                    if let Ok(value) = headers_obj.get(property_key, context) {
                        let value_str = value.to_string(context)?.to_std_string_escaped();
                        headers.insert(key_name, value_str);
                    }
                }
            }
        } else if headers_val.is_string() {
            // Handle string headers (less common)
            let headers_str = headers_val.to_string(context)?.to_std_string_escaped();
            // Simple parsing of "key: value" format
            for line in headers_str.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    headers.insert(key.trim().to_string(), value.trim().to_string());
                }
            }
        }
    }

    // Add default User-Agent if not present - use shared constant!
    if !headers.contains_key("User-Agent") {
        headers.insert("User-Agent".to_string(), thalora_constants::USER_AGENT.to_string());
    }

    // Body
    let body = if let Ok(body_val) = init_obj.get(js_string!("body"), context) {
        if !body_val.is_undefined() && !body_val.is_null() {
            Some(body_val.to_string(context)?.to_std_string_escaped())
        } else {
            None
        }
    } else {
        None
    };

    Ok((method, headers, body))
}

/// JavaScript `Request` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub struct Request;

impl IntrinsicObject for Request {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().request().constructor()
    }
}

impl BuiltInObject for Request {
    const NAME: JsString = js_string!("Request");
}

impl BuiltInConstructor for Request {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::request;

    /// `new Request(input, init)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Request constructor requires 'new'")
                .into());
        }

        let input = args.get_or_undefined(0);
        let init = args.get_or_undefined(1);

        // Parse URL — resolve relative URLs against page origin
        let url_raw = input.to_string(context)?.to_std_string_escaped();
        let url = match Url::parse(&url_raw) {
            Ok(parsed) => parsed.to_string(),
            Err(_) => {
                // Try to resolve as relative URL
                let base_url = get_base_url_from_context(context);
                if let Some(base) = base_url {
                    if let Ok(base_parsed) = Url::parse(&base) {
                        if let Ok(resolved) = base_parsed.join(&url_raw) {
                            resolved.to_string()
                        } else {
                            return Err(JsNativeError::typ()
                                .with_message("Invalid URL")
                                .into());
                        }
                    } else {
                        return Err(JsNativeError::typ()
                            .with_message("Invalid URL")
                            .into());
                    }
                } else {
                    return Err(JsNativeError::typ()
                        .with_message("Invalid URL")
                        .into());
                }
            }
        };

        // Create the Request object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::request, context)?;
        let request_data = RequestData {
            url,
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
        };
        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            request_data,
        );

        Ok(request_obj.into())
    }
}

impl Request {
}

/// JavaScript `Response` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub struct Response;

impl IntrinsicObject for Response {
    fn init(realm: &Realm) {
        // Create getter functions
        let status_getter = BuiltInBuilder::callable(realm, Self::get_status)
            .name(js_string!("get status"))
            .build();
        let status_text_getter = BuiltInBuilder::callable(realm, Self::get_status_text)
            .name(js_string!("get statusText"))
            .build();
        let ok_getter = BuiltInBuilder::callable(realm, Self::get_ok)
            .name(js_string!("get ok"))
            .build();
        let url_getter = BuiltInBuilder::callable(realm, Self::get_url)
            .name(js_string!("get url"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::text, js_string!("text"), 0)
            .method(Self::json, js_string!("json"), 0)
            .accessor(
                js_string!("status"),
                Some(status_getter),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("statusText"),
                Some(status_text_getter),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("ok"),
                Some(ok_getter),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("url"),
                Some(url_getter),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().response().constructor()
    }
}

impl BuiltInObject for Response {
    const NAME: JsString = js_string!("Response");
}

impl BuiltInConstructor for Response {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::response;

    /// `new Response(body, init)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Response constructor requires 'new'")
                .into());
        }

        let body = args.get_or_undefined(0);
        let init = args.get_or_undefined(1);

        // Parse body
        let body_text = if !body.is_undefined() && !body.is_null() {
            Some(body.to_string(context)?.to_std_string_escaped())
        } else {
            None
        };

        // Parse status and statusText from init
        let (status, status_text) = if !init.is_undefined() && !init.is_null() {
            if let Some(init_obj) = init.as_object() {
                let status = match init_obj.get(js_string!("status"), context) {
                    Ok(status_val) if !status_val.is_undefined() => {
                        match status_val.to_number(context) {
                            Ok(num) => num as u16,
                            Err(_) => 200,
                        }
                    }
                    _ => 200,
                };

                let status_text = match init_obj.get(js_string!("statusText"), context) {
                    Ok(status_text_val) if !status_text_val.is_undefined() => {
                        match status_text_val.to_string(context) {
                            Ok(s) => s.to_std_string_escaped(),
                            Err(_) => "OK".to_string(),
                        }
                    }
                    _ => if status >= 200 && status < 300 { "OK".to_string() } else { "".to_string() },
                };

                (status, status_text)
            } else {
                (200, "OK".to_string())
            }
        } else {
            (200, "OK".to_string())
        };

        // Create the Response object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::response, context)?;
        let response_data = ResponseData {
            body: body_text,
            status,
            status_text: status_text.clone(),
            headers: HashMap::new(),
            url: String::new(),
        };
        let response_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            response_data,
        );

        Ok(response_obj.into())
    }
}

impl Response {
    /// `get Response.prototype.status` getter
    fn get_status(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.status getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.status getter called on non-Response object")
        })?;

        Ok(JsValue::from(data.status))
    }

    /// `get Response.prototype.statusText` getter
    fn get_status_text(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.statusText getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.statusText getter called on non-Response object")
        })?;

        Ok(JsValue::from(js_string!(data.status_text.clone())))
    }

    /// `get Response.prototype.ok` getter
    fn get_ok(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.ok getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.ok getter called on non-Response object")
        })?;

        Ok(JsValue::from(data.status >= 200 && data.status < 300))
    }

    /// `get Response.prototype.url` getter
    fn get_url(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.url getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.url getter called on non-Response object")
        })?;

        Ok(JsValue::from(js_string!(data.url.clone())))
    }

    /// `Response.prototype.text()` method
    fn text(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let response_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.text called on non-object")
        })?;

        let response_data = response_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.text called on non-Response object")
        })?;

        // Create and return a resolved Promise with the body text
        if let Some(ref body) = response_data.body {
            Ok(JsPromise::resolve(JsValue::from(js_string!(body.clone())), context).into())
        } else {
            Ok(JsPromise::resolve(JsValue::from(js_string!("")), context).into())
        }
    }

    /// `Response.prototype.json()` method
    fn json(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let response_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.json called on non-object")
        })?;

        let response_data = response_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.json called on non-Response object")
        })?;

        // Parse JSON from body text
        if let Some(ref body) = response_data.body {
            match Json::parse(&JsValue::undefined(), &[JsValue::from(js_string!(body.clone()))], context) {
                Ok(json_value) => Ok(JsPromise::resolve(json_value, context).into()),
                Err(e) => {
                    let error = JsNativeError::syntax()
                        .with_message(format!("Failed to parse JSON: {}", e));
                    Ok(JsPromise::reject(error, context).into())
                }
            }
        } else {
            let error = JsNativeError::typ()
                .with_message("Response body is null");
            Ok(JsPromise::reject(error, context).into())
        }
    }
}

/// JavaScript `Headers` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub struct Headers;

impl IntrinsicObject for Headers {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        intrinsics.constructors().headers().constructor()
    }
}

impl BuiltInObject for Headers {
    const NAME: JsString = js_string!("Headers");
}

impl BuiltInConstructor for Headers {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::headers;

    /// `new Headers(init)`
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // If NewTarget is undefined, throw a TypeError
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Headers constructor requires 'new'")
                .into());
        }

        let init = args.get_or_undefined(0);
        let mut headers = HashMap::new();

        // Parse init parameter (can be array of [name, value] pairs, object, or another Headers)
        if !init.is_undefined() && !init.is_null() {
            if let Some(init_obj) = init.as_object() {
                // Check if it's a Headers object
                if let Some(headers_data) = init_obj.downcast_ref::<HeadersData>() {
                    // Copy headers from existing Headers object
                    headers = headers_data.headers.clone();
                } else {
                    // Check if it's an array of [name, value] pairs
                    let length_prop = init_obj.get(js_string!("length"), context)?;
                    if !length_prop.is_undefined() {
                        // Array-like: iterate and extract [name, value] pairs
                        let length = length_prop.to_length(context)?;
                        for i in 0..length {
                            let pair = init_obj.get(i, context)?;
                            if let Some(pair_obj) = pair.as_object() {
                                let pair_len = pair_obj.get(js_string!("length"), context)?;
                                if let Some(len) = pair_len.to_length(context).ok() {
                                    if len >= 2 {
                                        let name = pair_obj.get(0, context)?
                                            .to_string(context)?
                                            .to_std_string_escaped()
                                            .to_lowercase();
                                        let value = pair_obj.get(1, context)?
                                            .to_string(context)?
                                            .to_std_string_escaped();
                                        headers.insert(name, value);
                                    }
                                }
                            }
                        }
                    } else {
                        // Regular object: iterate over own properties
                        let keys = init_obj.own_property_keys(context)?;
                        for key in keys {
                            let value = init_obj.get(key.clone(), context)?;
                            if !value.is_undefined() {
                                // PropertyKey::to_string() returns a std::string::String
                                let name = key.to_string().to_lowercase();
                                let value_str = value.to_string(context)?.to_std_string_escaped();
                                headers.insert(name, value_str);
                            }
                        }
                    }
                }
            }
        }

        // Create the Headers object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::headers, context)?;
        let headers_data = HeadersData { headers };
        let headers_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            headers_data,
        );

        Ok(headers_obj.into())
    }
}

impl Headers {
}

/// Internal data for Request instances
#[derive(Debug, Trace, Finalize, JsData)]
struct RequestData {
    #[unsafe_ignore_trace]
    url: String,
    #[unsafe_ignore_trace]
    method: String,
    #[unsafe_ignore_trace]
    headers: HashMap<String, String>,
    #[unsafe_ignore_trace]
    body: Option<String>,
}

/// Internal data for Response instances
#[derive(Debug, Trace, Finalize, JsData)]
struct ResponseData {
    #[unsafe_ignore_trace]
    body: Option<String>,
    #[unsafe_ignore_trace]
    status: u16,
    #[unsafe_ignore_trace]
    status_text: String,
    #[unsafe_ignore_trace]
    headers: HashMap<String, String>,
    #[unsafe_ignore_trace]
    url: String,
}

/// Internal data for Headers instances
#[derive(Debug, Trace, Finalize, JsData)]
struct HeadersData {
    #[unsafe_ignore_trace]
    headers: HashMap<String, String>,
}