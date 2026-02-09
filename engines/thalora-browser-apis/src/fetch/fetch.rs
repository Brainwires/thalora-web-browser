//! Fetch Web API implementation for Boa
//!
//! Native implementation of the Fetch standard
//! https://fetch.spec.whatwg.org/
//!
//! This implements the complete Fetch interface with real HTTP networking


use boa_engine::{
    builtins::{IntrinsicObject, BuiltInBuilder, BuiltInObject, BuiltInConstructor, json::Json},
    object::{JsObject, builtins::{JsPromise, JsArray}, internal_methods::get_prototype_from_constructor},
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
                    eprintln!("DEBUG: fetch() response: {} {} ({} bytes body) for {}", status, status_text, body_text.len(), url_for_response);
                    let response_data = ResponseData {
                        body: Some(body_text),
                        status,
                        status_text: status_text.clone(),
                        headers: response_headers,
                        url: url_for_response.clone(),
                        body_used: false,
                    };

                    let prototype = context.intrinsics().constructors().response().prototype();
                    let response_obj = JsObject::from_proto_and_data_with_shared_shape(
                        context.root_shape(),
                        prototype,
                        response_data,
                    );
                    let response_obj = response_obj.upcast();

                    // Set own properties as fallback — prototype accessor resolution can
                    // be unreliable depending on how the object was created in Boa.
                    // These drop() calls ignore errors if the prototype accessors block the set.
                    drop(response_obj.set(js_string!("status"), JsValue::from(status), false, context));
                    drop(response_obj.set(js_string!("statusText"), JsValue::from(js_string!(status_text)), false, context));
                    drop(response_obj.set(js_string!("ok"), JsValue::from(status >= 200 && status < 300), false, context));
                    drop(response_obj.set(js_string!("url"), JsValue::from(js_string!(url_for_response)), false, context));

                    resolvers.resolve.call(&JsValue::undefined(), &[response_obj.into()], context)
                }
                Err(error_msg) => {
                    eprintln!("DEBUG: fetch() FAILED: {} for {}", error_msg, url_for_response);
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

// ============================================================================
// Request
// ============================================================================

/// JavaScript `Request` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub struct Request;

impl IntrinsicObject for Request {
    fn init(realm: &Realm) {
        let url_getter = BuiltInBuilder::callable(realm, Self::get_url)
            .name(js_string!("get url"))
            .build();
        let method_getter = BuiltInBuilder::callable(realm, Self::get_method)
            .name(js_string!("get method"))
            .build();
        let headers_getter = BuiltInBuilder::callable(realm, Self::get_headers)
            .name(js_string!("get headers"))
            .build();
        let body_getter = BuiltInBuilder::callable(realm, Self::get_body)
            .name(js_string!("get body"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::clone_request, js_string!("clone"), 0)
            .accessor(
                js_string!("url"),
                Some(url_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("method"),
                Some(method_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("headers"),
                Some(headers_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("body"),
                Some(body_getter),
                None,
                Attribute::CONFIGURABLE,
            )
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

        // Parse init options if provided
        let (method, headers, body) = if !init.is_undefined() && !init.is_null() {
            parse_fetch_init(init, context)?
        } else {
            ("GET".to_string(), HashMap::new(), None)
        };

        // Create the Request object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::request, context)?;
        let request_data = RequestData {
            url,
            method,
            headers,
            body,
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
    /// `get Request.prototype.url`
    fn get_url(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Request.prototype.url getter called on non-object")
        })?;
        let data = this_obj.downcast_ref::<RequestData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Request.prototype.url getter called on non-Request object")
        })?;
        Ok(JsValue::from(js_string!(data.url.clone())))
    }

    /// `get Request.prototype.method`
    fn get_method(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Request.prototype.method getter called on non-object")
        })?;
        let data = this_obj.downcast_ref::<RequestData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Request.prototype.method getter called on non-Request object")
        })?;
        Ok(JsValue::from(js_string!(data.method.clone())))
    }

    /// `get Request.prototype.headers` — creates Headers from RequestData, cached as __headers__
    fn get_headers(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Request.prototype.headers getter called on non-object")
        })?;

        // Check for cached headers first
        let cached = this_obj.get(js_string!("__headers__"), context)?;
        if !cached.is_undefined() {
            return Ok(cached);
        }

        // Clone headers map while borrow is active, then drop borrow
        let headers_map = {
            let data = this_obj.downcast_ref::<RequestData>().ok_or_else(|| {
                JsNativeError::typ().with_message("Request.prototype.headers getter called on non-Request object")
            })?;
            data.headers.clone()
        };

        let headers_obj = create_headers_from_hashmap(&headers_map, context)?;
        let headers_val: JsValue = headers_obj.into();

        // Cache it for identity: request.headers === request.headers
        this_obj.set(js_string!("__headers__"), headers_val.clone(), false, context)?;

        Ok(headers_val)
    }

    /// `get Request.prototype.body`
    fn get_body(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Request.prototype.body getter called on non-object")
        })?;
        let data = this_obj.downcast_ref::<RequestData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Request.prototype.body getter called on non-Request object")
        })?;
        match &data.body {
            Some(body) => Ok(JsValue::from(js_string!(body.clone()))),
            None => Ok(JsValue::null()),
        }
    }

    /// `Request.prototype.clone()`
    fn clone_request(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Request.prototype.clone called on non-object")
        })?;
        let data = this_obj.downcast_ref::<RequestData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Request.prototype.clone called on non-Request object")
        })?;

        let cloned_data = RequestData {
            url: data.url.clone(),
            method: data.method.clone(),
            headers: data.headers.clone(),
            body: data.body.clone(),
        };
        drop(data);

        let prototype = context.intrinsics().constructors().request().prototype();
        let cloned_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            cloned_data,
        );

        Ok(cloned_obj.into())
    }
}

// ============================================================================
// Response
// ============================================================================

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
        let headers_getter = BuiltInBuilder::callable(realm, Self::get_headers)
            .name(js_string!("get headers"))
            .build();
        let type_getter = BuiltInBuilder::callable(realm, Self::get_type)
            .name(js_string!("get type"))
            .build();
        let redirected_getter = BuiltInBuilder::callable(realm, Self::get_redirected)
            .name(js_string!("get redirected"))
            .build();
        let body_used_getter = BuiltInBuilder::callable(realm, Self::get_body_used)
            .name(js_string!("get bodyUsed"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::text, js_string!("text"), 0)
            .method(Self::json, js_string!("json"), 0)
            .method(Self::clone_response, js_string!("clone"), 0)
            .static_method(Self::static_error, js_string!("error"), 0)
            .static_method(Self::static_redirect, js_string!("redirect"), 1)
            .static_method(Self::static_json, js_string!("json"), 1)
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
            .accessor(
                js_string!("headers"),
                Some(headers_getter),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("type"),
                Some(type_getter),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("redirected"),
                Some(redirected_getter),
                None,
                Attribute::READONLY | Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("bodyUsed"),
                Some(body_used_getter),
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

        // Parse status, statusText, and headers from init
        let (status, status_text, init_headers) = if !init.is_undefined() && !init.is_null() {
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

                // Parse headers from init
                let mut hdrs = HashMap::new();
                if let Ok(headers_val) = init_obj.get(js_string!("headers"), context) {
                    if let Some(headers_obj) = headers_val.as_object() {
                        if let Some(headers_data) = headers_obj.downcast_ref::<HeadersData>() {
                            hdrs = headers_data.headers.clone();
                        } else {
                            for property_key in headers_obj.own_property_keys(context)? {
                                let key_name = property_key.to_string().to_lowercase();
                                if let Ok(value) = headers_obj.get(property_key, context) {
                                    let value_str = value.to_string(context)?.to_std_string_escaped();
                                    hdrs.insert(key_name, value_str);
                                }
                            }
                        }
                    }
                }

                (status, status_text, hdrs)
            } else {
                (200, "OK".to_string(), HashMap::new())
            }
        } else {
            (200, "OK".to_string(), HashMap::new())
        };

        // Create the Response object
        let proto = get_prototype_from_constructor(new_target, StandardConstructors::response, context)?;
        let response_data = ResponseData {
            body: body_text,
            status,
            status_text: status_text.clone(),
            headers: init_headers,
            url: String::new(),
            body_used: false,
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

    /// `get Response.prototype.headers` — creates Headers from ResponseData, cached as __headers__
    fn get_headers(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.headers getter called on non-object")
        })?;

        // Check for cached headers first
        let cached = this_obj.get(js_string!("__headers__"), context)?;
        if !cached.is_undefined() {
            return Ok(cached);
        }

        // Clone headers map while borrow is active, then drop the borrow
        let headers_map = {
            let data = this_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
                JsNativeError::typ().with_message("Response.prototype.headers getter called on non-Response object")
            })?;
            data.headers.clone()
        };

        let headers_obj = create_headers_from_hashmap(&headers_map, context)?;
        let headers_val: JsValue = headers_obj.into();

        // Cache it for identity: response.headers === response.headers
        this_obj.set(js_string!("__headers__"), headers_val.clone(), false, context)?;

        Ok(headers_val)
    }

    /// `get Response.prototype.type` — returns "basic"
    fn get_type(_this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(js_string!("basic")))
    }

    /// `get Response.prototype.redirected` — returns false
    fn get_redirected(_this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        Ok(JsValue::from(false))
    }

    /// `get Response.prototype.bodyUsed` getter
    fn get_body_used(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.bodyUsed getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.bodyUsed getter called on non-Response object")
        })?;

        Ok(JsValue::from(data.body_used))
    }

    /// `Response.prototype.text()` method
    fn text(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let response_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.text called on non-object")
        })?;

        // Check bodyUsed
        {
            let data = response_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
                JsNativeError::typ().with_message("Response.text called on non-Response object")
            })?;
            if data.body_used {
                let error = JsNativeError::typ()
                    .with_message("body stream already read");
                return Ok(JsPromise::reject(error, context).into());
            }
        }

        // Mark body as used and read body
        let body_text = {
            let mut data = response_obj.downcast_mut::<ResponseData>().ok_or_else(|| {
                JsNativeError::typ().with_message("Response.text called on non-Response object")
            })?;
            data.body_used = true;
            data.body.clone()
        };

        // Create and return a resolved Promise with the body text
        if let Some(body) = body_text {
            Ok(JsPromise::resolve(JsValue::from(js_string!(body)), context).into())
        } else {
            Ok(JsPromise::resolve(JsValue::from(js_string!("")), context).into())
        }
    }

    /// `Response.prototype.json()` method
    fn json(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let response_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.json called on non-object")
        })?;

        // Check bodyUsed
        {
            let data = response_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
                JsNativeError::typ().with_message("Response.json called on non-Response object")
            })?;
            if data.body_used {
                let error = JsNativeError::typ()
                    .with_message("body stream already read");
                return Ok(JsPromise::reject(error, context).into());
            }
        }

        // Mark body as used and read body
        let body_text = {
            let mut data = response_obj.downcast_mut::<ResponseData>().ok_or_else(|| {
                JsNativeError::typ().with_message("Response.json called on non-Response object")
            })?;
            data.body_used = true;
            data.body.clone()
        };

        // Parse JSON from body text
        if let Some(body) = body_text {
            match Json::parse(&JsValue::undefined(), &[JsValue::from(js_string!(body))], context) {
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

    /// `Response.prototype.clone()` method
    fn clone_response(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.clone called on non-object")
        })?;

        let data = this_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.clone called on non-Response object")
        })?;

        if data.body_used {
            return Err(JsNativeError::typ()
                .with_message("Failed to execute 'clone' on 'Response': body is already used")
                .into());
        }

        let cloned_data = ResponseData {
            body: data.body.clone(),
            status: data.status,
            status_text: data.status_text.clone(),
            headers: data.headers.clone(),
            url: data.url.clone(),
            body_used: false,
        };
        drop(data);

        let prototype = context.intrinsics().constructors().response().prototype();
        let cloned_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            cloned_data,
        );

        Ok(cloned_obj.into())
    }

    /// `Response.error()` static method — returns a network error Response
    fn static_error(_: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let response_data = ResponseData {
            body: None,
            status: 0,
            status_text: String::new(),
            headers: HashMap::new(),
            url: String::new(),
            body_used: false,
        };

        let prototype = context.intrinsics().constructors().response().prototype();
        let response_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            response_data,
        );

        Ok(response_obj.into())
    }

    /// `Response.redirect(url, status)` static method
    fn static_redirect(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let url = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let status = if let Some(status_val) = args.get(1) {
            if status_val.is_undefined() {
                302u16
            } else {
                status_val.to_number(context)? as u16
            }
        } else {
            302
        };

        // Validate redirect status
        if !matches!(status, 301 | 302 | 303 | 307 | 308) {
            return Err(JsNativeError::range()
                .with_message(format!("Failed to execute 'redirect' on 'Response': Invalid status code {}", status))
                .into());
        }

        let mut headers = HashMap::new();
        headers.insert("location".to_string(), url);

        let response_data = ResponseData {
            body: None,
            status,
            status_text: String::new(),
            headers,
            url: String::new(),
            body_used: false,
        };

        let prototype = context.intrinsics().constructors().response().prototype();
        let response_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            response_data,
        );

        Ok(response_obj.into())
    }

    /// `Response.json(data, init)` static method — creates a Response with JSON body
    fn static_json(_: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let data = args.get_or_undefined(0);
        let init = args.get_or_undefined(1);

        // JSON.stringify the data via global JSON object
        let global = context.global_object();
        let json_obj = global.get(js_string!("JSON"), context)?;
        let json_obj = json_obj.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("JSON is not an object")
        })?;
        let stringify_fn = json_obj.get(js_string!("stringify"), context)?;
        let stringify_fn = stringify_fn.as_callable().ok_or_else(|| {
            JsNativeError::typ().with_message("JSON.stringify is not callable")
        })?;
        let json_result = stringify_fn.call(&JsValue::undefined(), &[data.clone()], context)?;
        let body_text = json_result.to_string(context)?.to_std_string_escaped();

        // Parse status from init
        let (status, status_text) = if !init.is_undefined() && !init.is_null() {
            if let Some(init_obj) = init.as_object() {
                let st = match init_obj.get(js_string!("status"), context) {
                    Ok(v) if !v.is_undefined() => v.to_number(context)? as u16,
                    _ => 200,
                };
                let st_text = match init_obj.get(js_string!("statusText"), context) {
                    Ok(v) if !v.is_undefined() => v.to_string(context)?.to_std_string_escaped(),
                    _ => "OK".to_string(),
                };
                (st, st_text)
            } else {
                (200, "OK".to_string())
            }
        } else {
            (200, "OK".to_string())
        };

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        let response_data = ResponseData {
            body: Some(body_text),
            status,
            status_text,
            headers,
            url: String::new(),
            body_used: false,
        };

        let prototype = context.intrinsics().constructors().response().prototype();
        let response_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            response_data,
        );

        Ok(response_obj.into())
    }
}

// ============================================================================
// Headers
// ============================================================================

/// JavaScript `Headers` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub struct Headers;

impl IntrinsicObject for Headers {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(Self::get, js_string!("get"), 1)
            .method(Self::set, js_string!("set"), 2)
            .method(Self::has, js_string!("has"), 1)
            .method(Self::delete, js_string!("delete"), 1)
            .method(Self::append, js_string!("append"), 2)
            .method(Self::for_each, js_string!("forEach"), 1)
            .method(Self::entries, js_string!("entries"), 0)
            .method(Self::keys, js_string!("keys"), 0)
            .method(Self::values, js_string!("values"), 0)
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
    /// `Headers.prototype.get(name)` — returns value or null, case-insensitive
    fn get(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.get called on non-object")
        })?;
        let data = this_obj.downcast_ref::<HeadersData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.get called on non-Headers object")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped().to_lowercase();

        match data.headers.get(&name) {
            Some(value) => Ok(JsValue::from(js_string!(value.clone()))),
            None => Ok(JsValue::null()),
        }
    }

    /// `Headers.prototype.set(name, value)` — sets header, case-insensitive
    fn set(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.set called on non-object")
        })?;
        let mut data = this_obj.downcast_mut::<HeadersData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.set called on non-Headers object")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped().to_lowercase();
        let value = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();
        let value = value.trim().to_string();

        data.headers.insert(name, value);

        Ok(JsValue::undefined())
    }

    /// `Headers.prototype.has(name)` — returns boolean, case-insensitive
    fn has(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.has called on non-object")
        })?;
        let data = this_obj.downcast_ref::<HeadersData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.has called on non-Headers object")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped().to_lowercase();

        Ok(JsValue::from(data.headers.contains_key(&name)))
    }

    /// `Headers.prototype.delete(name)` — removes header, case-insensitive
    fn delete(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.delete called on non-object")
        })?;
        let mut data = this_obj.downcast_mut::<HeadersData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.delete called on non-Headers object")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped().to_lowercase();

        data.headers.remove(&name);

        Ok(JsValue::undefined())
    }

    /// `Headers.prototype.append(name, value)` — appends value (comma-separated if exists)
    fn append(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.append called on non-object")
        })?;
        let mut data = this_obj.downcast_mut::<HeadersData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.append called on non-Headers object")
        })?;

        let name = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped().to_lowercase();
        let value = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();
        let value = value.trim().to_string();

        if let Some(existing) = data.headers.get_mut(&name) {
            existing.push_str(", ");
            existing.push_str(&value);
        } else {
            data.headers.insert(name, value);
        }

        Ok(JsValue::undefined())
    }

    /// `Headers.prototype.forEach(callback, thisArg)` — iterates sorted entries as (value, name, headers)
    fn for_each(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.forEach called on non-object")
        })?;

        let callback = args.get_or_undefined(0);
        if !callback.is_callable() {
            return Err(JsNativeError::typ()
                .with_message("Headers.prototype.forEach: callback is not callable")
                .into());
        }
        let callback_obj = callback.as_object().unwrap();

        let this_arg = args.get_or_undefined(1).clone();

        // Clone and sort entries while holding borrow
        let sorted_entries = {
            let data = this_obj.downcast_ref::<HeadersData>().ok_or_else(|| {
                JsNativeError::typ().with_message("Headers.prototype.forEach called on non-Headers object")
            })?;
            let mut entries: Vec<(String, String)> = data.headers.iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            entries
        };

        for (name, value) in sorted_entries {
            let call_args = [
                JsValue::from(js_string!(value)),
                JsValue::from(js_string!(name)),
                this.clone(),
            ];
            callback_obj.call(&this_arg, &call_args, context)?;
        }

        Ok(JsValue::undefined())
    }

    /// `Headers.prototype.entries()` — returns array of [name, value] pairs, sorted by name
    fn entries(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.entries called on non-object")
        })?;
        let data = this_obj.downcast_ref::<HeadersData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.entries called on non-Headers object")
        })?;

        let mut entries: Vec<(String, String)> = data.headers.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let pairs: Vec<JsValue> = entries.iter()
            .map(|(k, v)| {
                let pair = JsArray::from_iter(
                    vec![
                        JsValue::from(js_string!(k.clone())),
                        JsValue::from(js_string!(v.clone())),
                    ],
                    context,
                );
                pair.into()
            })
            .collect();

        Ok(JsArray::from_iter(pairs, context).into())
    }

    /// `Headers.prototype.keys()` — returns array of names, sorted
    fn keys(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.keys called on non-object")
        })?;
        let data = this_obj.downcast_ref::<HeadersData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.keys called on non-Headers object")
        })?;

        let mut keys: Vec<String> = data.headers.keys().cloned().collect();
        keys.sort();

        let values: Vec<JsValue> = keys.iter()
            .map(|k| JsValue::from(js_string!(k.clone())))
            .collect();

        Ok(JsArray::from_iter(values, context).into())
    }

    /// `Headers.prototype.values()` — returns array of values, sorted by name
    fn values(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.values called on non-object")
        })?;
        let data = this_obj.downcast_ref::<HeadersData>().ok_or_else(|| {
            JsNativeError::typ().with_message("Headers.prototype.values called on non-Headers object")
        })?;

        let mut entries: Vec<(String, String)> = data.headers.iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        entries.sort_by(|a, b| a.0.cmp(&b.0));

        let values: Vec<JsValue> = entries.iter()
            .map(|(_, v)| JsValue::from(js_string!(v.clone())))
            .collect();

        Ok(JsArray::from_iter(values, context).into())
    }
}

// ============================================================================
// Helpers
// ============================================================================

/// Create a Headers JsObject from a HashMap
fn create_headers_from_hashmap(map: &HashMap<String, String>, context: &mut Context) -> JsResult<JsObject> {
    let headers_data = HeadersData {
        headers: map.clone(),
    };
    let prototype = context.intrinsics().constructors().headers().prototype();
    let headers_obj = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        prototype,
        headers_data,
    );
    Ok(headers_obj.upcast())
}

// ============================================================================
// Internal Data Types
// ============================================================================

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
    #[unsafe_ignore_trace]
    body_used: bool,
}

/// Internal data for Headers instances
#[derive(Debug, Trace, Finalize, JsData)]
pub(crate) struct HeadersData {
    #[unsafe_ignore_trace]
    pub(crate) headers: HashMap<String, String>,
}
