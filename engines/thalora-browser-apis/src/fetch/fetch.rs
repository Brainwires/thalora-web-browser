//! Fetch Web API implementation for Boa
//!
//! Native implementation of the Fetch standard
//! https://fetch.spec.whatwg.org/
//!
//! This implements the complete Fetch interface with real HTTP networking

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject, json::Json},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    job::NativeAsyncJob,
    js_string,
    object::{
        JsObject, builtins::JsPromise, internal_methods::get_prototype_from_constructor,
    },
    property::Attribute,
    realm::Realm,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};
use reqwest;
use std::collections::HashMap;
use std::sync::Mutex;
use url::Url;

/// Simple CORS preflight cache: (origin, method) → expiry time.
/// Caches successful preflight results per Access-Control-Max-Age.
static PREFLIGHT_CACHE: std::sync::LazyLock<Mutex<HashMap<(String, String), std::time::Instant>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

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

/// `fetch(input, init)` global function
fn fetch(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
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

    // Validate URL
    let _url = Url::parse(&url_string)
        .map_err(|_| JsNativeError::typ().with_message(format!("Invalid URL: {}", url_string)))?;

    // Parse init options
    let fetch_init = if !init.is_undefined() {
        parse_fetch_init(init, context)?
    } else {
        FetchInit {
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
            mode: "cors".to_string(),
            credentials: "same-origin".to_string(),
        }
    };
    let method = fetch_init.method;
    let headers = fetch_init.headers;
    let body = fetch_init.body;
    let mode = fetch_init.mode;
    let _credentials = fetch_init.credentials;

    // Create a new pending Promise and return it immediately
    let (promise, resolvers) = JsPromise::new_pending(context);

    // Enqueue an async job to perform the actual HTTP request
    context.enqueue_job(
        NativeAsyncJob::new(async move |context| {
            // Perform HTTP request in the background
            let client = reqwest::Client::new();

            // CORS preflight for non-simple cross-origin requests
            // Check preflight cache first
            let cache_key = {
                let origin = Url::parse(&url_string)
                    .ok()
                    .and_then(|u| u.host_str().map(|h| h.to_string()))
                    .unwrap_or_default();
                (origin, method.clone())
            };
            let preflight_cached = PREFLIGHT_CACHE
                .lock()
                .ok()
                .and_then(|cache| {
                    cache
                        .get(&cache_key)
                        .map(|expiry| std::time::Instant::now() < *expiry)
                })
                .unwrap_or(false);

            if mode == "cors" && !is_cors_simple_request(&method, &headers) && !preflight_cached {
                // Collect custom header names for Access-Control-Request-Headers
                let safelisted = ["accept", "accept-language", "content-language", "content-type", "user-agent"];
                let custom_headers: Vec<String> = headers.keys()
                    .filter(|k| !safelisted.contains(&k.to_lowercase().as_str()))
                    .cloned()
                    .collect();

                let mut preflight = client.request(reqwest::Method::OPTIONS, &url_string)
                    .header("Access-Control-Request-Method", &method);

                if !custom_headers.is_empty() {
                    preflight = preflight.header(
                        "Access-Control-Request-Headers",
                        custom_headers.join(", "),
                    );
                }

                // Send preflight and validate the response
                match preflight.send().await {
                    Ok(preflight_resp) => {
                        let status = preflight_resp.status().as_u16();
                        let strict_cors = std::env::var("THALORA_STRICT_CORS")
                            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                            .unwrap_or(true);

                        if status < 200 || status >= 300 {
                            eprintln!("⚠️  CORS preflight returned status {}", status);
                            if strict_cors {
                                return Err(JsNativeError::typ()
                                    .with_message(format!(
                                        "Failed to fetch: CORS preflight returned status {}",
                                        status
                                    ))
                                    .into());
                            }
                        }

                        // Validate Access-Control-Allow-Methods
                        let allowed_methods = preflight_resp
                            .headers()
                            .get("access-control-allow-methods")
                            .and_then(|v| v.to_str().ok())
                            .unwrap_or("");
                        let method_upper = method.to_uppercase();
                        let method_allowed = allowed_methods == "*"
                            || allowed_methods
                                .split(',')
                                .any(|m| m.trim().eq_ignore_ascii_case(&method_upper));
                        if !method_allowed && !matches!(method_upper.as_str(), "GET" | "HEAD" | "POST") {
                            eprintln!(
                                "⚠️  CORS: method {} not in Access-Control-Allow-Methods: {}",
                                method, allowed_methods
                            );
                            if strict_cors {
                                return Err(JsNativeError::typ()
                                    .with_message(format!(
                                        "Failed to fetch: method {} not allowed by CORS",
                                        method
                                    ))
                                    .into());
                            }
                        }

                        // Validate Access-Control-Allow-Headers
                        if !custom_headers.is_empty() {
                            let allowed_headers = preflight_resp
                                .headers()
                                .get("access-control-allow-headers")
                                .and_then(|v| v.to_str().ok())
                                .unwrap_or("");
                            let allowed_set: Vec<String> = allowed_headers
                                .split(',')
                                .map(|h| h.trim().to_lowercase())
                                .collect();
                            let headers_ok = allowed_headers == "*"
                                || custom_headers
                                    .iter()
                                    .all(|h| allowed_set.contains(&h.to_lowercase()));
                            if !headers_ok {
                                eprintln!(
                                    "⚠️  CORS: custom headers {:?} not all in Access-Control-Allow-Headers: {}",
                                    custom_headers, allowed_headers
                                );
                                if strict_cors {
                                    return Err(JsNativeError::typ()
                                        .with_message(
                                            "Failed to fetch: headers not allowed by CORS"
                                                .to_string(),
                                        )
                                        .into());
                                }
                            }
                        }

                        // Cache successful preflight result per Access-Control-Max-Age
                        let max_age = preflight_resp
                            .headers()
                            .get("access-control-max-age")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| v.parse::<u64>().ok())
                            .unwrap_or(5); // Default 5 seconds per spec
                        if max_age > 0 {
                            if let Ok(mut cache) = PREFLIGHT_CACHE.lock() {
                                cache.insert(
                                    cache_key.clone(),
                                    std::time::Instant::now()
                                        + std::time::Duration::from_secs(max_age),
                                );
                                // Evict expired entries periodically (keep cache bounded)
                                if cache.len() > 100 {
                                    let now = std::time::Instant::now();
                                    cache.retain(|_, expiry| *expiry > now);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("⚠️  CORS preflight request failed: {}", e);
                        let strict_cors = std::env::var("THALORA_STRICT_CORS")
                            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                            .unwrap_or(true);
                        if strict_cors {
                            return Err(JsNativeError::typ()
                                .with_message(format!(
                                    "Failed to fetch: CORS preflight failed: {}",
                                    e
                                ))
                                .into());
                        }
                    }
                }
            }

            let mut request_builder = client.request(
                reqwest::Method::from_bytes(method.as_bytes()).unwrap_or(reqwest::Method::GET),
                &url_string,
            );

            // Add headers
            for (key, value) in &headers {
                request_builder = request_builder.header(key, value);
            }

            // Add body if present
            if let Some(body_content) = body {
                request_builder = request_builder.body(body_content);
            }

            // Execute the request
            let response_result = request_builder.send().await;

            let context = &mut context.borrow_mut();

            match response_result {
                Ok(response) => {
                    // Extract response data
                    let status = response.status().as_u16();
                    let status_text = response
                        .status()
                        .canonical_reason()
                        .unwrap_or("")
                        .to_string();

                    // Convert headers
                    let mut response_headers = HashMap::new();
                    for (name, value) in response.headers() {
                        if let Ok(value_str) = value.to_str() {
                            response_headers.insert(name.to_string(), value_str.to_string());
                        }
                    }

                    // Determine response type based on CORS headers and request mode
                    let has_acao = response_headers.contains_key("access-control-allow-origin");
                    let response_type = if mode == "no-cors" {
                        "opaque"
                    } else if has_acao {
                        "cors"
                    } else {
                        "basic"
                    };

                    // Get response body
                    let body_result = response.text().await;
                    match body_result {
                        Ok(body_text) => {
                            // Create Response object and resolve the promise
                            let response_data = ResponseData {
                                body: Some(body_text),
                                status,
                                status_text: status_text.clone(),
                                headers: response_headers,
                                url: url_string.clone(),
                            };

                            let response_obj = JsObject::from_proto_and_data(None, response_data);

                            // Add properties to the Response object
                            drop(response_obj.set(
                                js_string!("status"),
                                JsValue::from(status),
                                false,
                                context,
                            ));
                            drop(response_obj.set(
                                js_string!("statusText"),
                                JsValue::from(js_string!(status_text)),
                                false,
                                context,
                            ));
                            drop(response_obj.set(
                                js_string!("ok"),
                                JsValue::from(status >= 200 && status < 300),
                                false,
                                context,
                            ));
                            drop(response_obj.set(
                                js_string!("url"),
                                JsValue::from(js_string!(url_string)),
                                false,
                                context,
                            ));
                            // Set response type per Fetch spec
                            drop(response_obj.set(
                                js_string!("type"),
                                JsValue::from(js_string!(response_type)),
                                false,
                                context,
                            ));
                            // Redirected flag
                            drop(response_obj.set(
                                js_string!("redirected"),
                                JsValue::from(false),
                                false,
                                context,
                            ));
                            // Body used flag
                            drop(response_obj.set(
                                js_string!("bodyUsed"),
                                JsValue::from(false),
                                false,
                                context,
                            ));

                            resolvers.resolve.call(
                                &JsValue::undefined(),
                                &[response_obj.into()],
                                context,
                            )
                        }
                        Err(e) => {
                            // Reject promise with body read error
                            let error = JsNativeError::typ()
                                .with_message(format!("Failed to read response body: {}", e))
                                .into_opaque(context);
                            resolvers
                                .reject
                                .call(&JsValue::undefined(), &[error.into()], context)
                        }
                    }
                }
                Err(e) => {
                    // Reject promise with network error
                    let error = JsNativeError::typ()
                        .with_message(format!("Fetch request failed: {}", e))
                        .into_opaque(context);
                    resolvers
                        .reject
                        .call(&JsValue::undefined(), &[error.into()], context)
                }
            }
        })
        .into(),
    );

    // Return the Promise immediately
    Ok(promise.into())
}

/// Parse fetch init options
/// Parsed fetch init options
struct FetchInit {
    method: String,
    headers: HashMap<String, String>,
    body: Option<String>,
    mode: String,
    credentials: String,
}

fn parse_fetch_init(
    init: &JsValue,
    context: &mut Context,
) -> JsResult<FetchInit> {
    let init_obj = init
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("fetch init must be an object"))?;

    // Method
    let method = if let Ok(method_val) = init_obj.get(js_string!("method"), context) {
        method_val
            .to_string(context)?
            .to_std_string_escaped()
            .to_uppercase()
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
        headers.insert(
            "User-Agent".to_string(),
            thalora_constants::USER_AGENT.to_string(),
        );
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

    // Mode (cors, no-cors, same-origin, navigate)
    let mode = if let Ok(mode_val) = init_obj.get(js_string!("mode"), context) {
        if !mode_val.is_undefined() {
            mode_val.to_string(context)?.to_std_string_escaped()
        } else {
            "cors".to_string()
        }
    } else {
        "cors".to_string()
    };

    // Credentials (omit, same-origin, include)
    let credentials = if let Ok(cred_val) = init_obj.get(js_string!("credentials"), context) {
        if !cred_val.is_undefined() {
            cred_val.to_string(context)?.to_std_string_escaped()
        } else {
            "same-origin".to_string()
        }
    } else {
        "same-origin".to_string()
    };

    Ok(FetchInit { method, headers, body, mode, credentials })
}

/// Check if a request is a CORS "simple request" that doesn't need preflight.
fn is_cors_simple_request(method: &str, headers: &HashMap<String, String>) -> bool {
    // Simple methods
    if !matches!(method, "GET" | "HEAD" | "POST") {
        return false;
    }

    // CORS-safelisted headers (case-insensitive check)
    let safelisted = ["accept", "accept-language", "content-language", "content-type"];
    let simple_content_types = [
        "application/x-www-form-urlencoded",
        "multipart/form-data",
        "text/plain",
    ];

    for (key, value) in headers {
        let key_lower = key.to_lowercase();
        if key_lower == "user-agent" {
            continue; // We always add this, it's fine
        }
        if !safelisted.contains(&key_lower.as_str()) {
            return false; // Non-safelisted header = not simple
        }
        if key_lower == "content-type" {
            let ct_lower = value.to_lowercase();
            if !simple_content_types.iter().any(|&t| ct_lower.starts_with(t)) {
                return false;
            }
        }
    }

    true
}

/// Check if two URLs have the same origin (scheme + host + port).
fn same_origin(url_a: &str, url_b: &str) -> bool {
    let parse = |u: &str| -> Option<(String, String, u16)> {
        let p = url::Url::parse(u).ok()?;
        let scheme = p.scheme().to_string();
        let host = p.host_str()?.to_string();
        let port = p.port_or_known_default().unwrap_or(if scheme == "https" { 443 } else { 80 });
        Some((scheme, host, port))
    };
    match (parse(url_a), parse(url_b)) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    }
}

/// JavaScript `Request` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub struct Request;

impl IntrinsicObject for Request {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm).build();
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
        let _init = args.get_or_undefined(1);

        // Parse URL
        let url = input.to_string(context)?.to_std_string_escaped();

        // Validate URL
        if Url::parse(&url).is_err() {
            return Err(JsNativeError::typ().with_message("Invalid URL").into());
        }

        // Create the Request object
        let proto =
            get_prototype_from_constructor(new_target, StandardConstructors::request, context)?;
        let request_data = RequestData {
            url,
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: None,
            mode: "cors".to_string(),
            credentials: "same-origin".to_string(),
        };
        let request_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            request_data,
        );

        Ok(request_obj.into())
    }
}

impl Request {}

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
                    _ => {
                        if status >= 200 && status < 300 {
                            "OK".to_string()
                        } else {
                            "".to_string()
                        }
                    }
                };

                (status, status_text)
            } else {
                (200, "OK".to_string())
            }
        } else {
            (200, "OK".to_string())
        };

        // Create the Response object
        let proto =
            get_prototype_from_constructor(new_target, StandardConstructors::response, context)?;
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
            JsNativeError::typ()
                .with_message("Response.prototype.status getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Response.prototype.status getter called on non-Response object")
        })?;

        Ok(JsValue::from(data.status))
    }

    /// `get Response.prototype.statusText` getter
    fn get_status_text(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Response.prototype.statusText getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Response.prototype.statusText getter called on non-Response object")
        })?;

        Ok(JsValue::from(js_string!(data.status_text.clone())))
    }

    /// `get Response.prototype.ok` getter
    fn get_ok(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.ok getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Response.prototype.ok getter called on non-Response object")
        })?;

        Ok(JsValue::from(data.status >= 200 && data.status < 300))
    }

    /// `get Response.prototype.url` getter
    fn get_url(this: &JsValue, _: &[JsValue], _: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Response.prototype.url getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<ResponseData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Response.prototype.url getter called on non-Response object")
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
            Ok(JsPromise::resolve(JsValue::from(js_string!(body.clone())), context)?.into())
        } else {
            Ok(JsPromise::resolve(JsValue::from(js_string!("")), context)?.into())
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
            match Json::parse(
                &JsValue::undefined(),
                &[JsValue::from(js_string!(body.clone()))],
                context,
            ) {
                Ok(json_value) => Ok(JsPromise::resolve(json_value, context)?.into()),
                Err(e) => {
                    let error = JsNativeError::syntax()
                        .with_message(format!("Failed to parse JSON: {}", e));
                    Ok(JsPromise::reject(error, context)?.into())
                }
            }
        } else {
            let error = JsNativeError::typ().with_message("Response body is null");
            Ok(JsPromise::reject(error, context)?.into())
        }
    }
}

/// JavaScript `Headers` constructor implementation.
#[derive(Debug, Copy, Clone)]
pub struct Headers;

impl IntrinsicObject for Headers {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm).build();
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
                                        let name = pair_obj
                                            .get(0, context)?
                                            .to_string(context)?
                                            .to_std_string_escaped()
                                            .to_lowercase();
                                        let value = pair_obj
                                            .get(1, context)?
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
        let proto =
            get_prototype_from_constructor(new_target, StandardConstructors::headers, context)?;
        let headers_data = HeadersData { headers };
        let headers_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            headers_data,
        );

        Ok(headers_obj.into())
    }
}

impl Headers {}

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
    #[unsafe_ignore_trace]
    mode: String,
    #[unsafe_ignore_trace]
    credentials: String,
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
