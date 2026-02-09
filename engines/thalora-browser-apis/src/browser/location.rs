//! Location Web API implementation for Boa
//!
//! The Location interface represents the location (URL) of the object it is linked to.
//! Changes done on it are reflected on the object it relates to.
//!
//! https://html.spec.whatwg.org/multipage/history.html#the-location-interface

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString,
};
use boa_gc::{Finalize, Trace};

/// JavaScript `Location` object implementation
#[derive(Debug, Copy, Clone)]
pub struct Location;

/// Internal data for Location objects
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct LocationData {
    #[unsafe_ignore_trace]
    href: std::sync::Arc<std::sync::Mutex<String>>,
}

impl LocationData {
    /// Create a new LocationData with default values
    pub fn new() -> Self {
        Self {
            href: std::sync::Arc::new(std::sync::Mutex::new("about:blank".to_string())),
        }
    }

    /// Create LocationData with specific href
    pub fn with_href(href: String) -> Self {
        Self {
            href: std::sync::Arc::new(std::sync::Mutex::new(href)),
        }
    }

    /// Set the href (for navigation URL updates)
    pub fn set_href(&self, href: &str) {
        *self.href.lock().unwrap() = href.to_string();
    }

    /// Get the current href
    pub fn get_href(&self) -> String {
        self.href.lock().unwrap().clone()
    }

    /// Parse URL components from href
    fn parse_url(&self) -> ParsedUrl {
        let href = self.get_href();
        // Simple URL parsing - in production would use url crate
        if let Some(protocol_end) = href.find("://") {
            let protocol = &href[..protocol_end + 1];
            let rest = &href[protocol_end + 3..];

            let (host, path_search_hash) = if let Some(slash_pos) = rest.find('/') {
                (&rest[..slash_pos], &rest[slash_pos..])
            } else {
                (rest, "")
            };

            let (pathname, search, hash) = Self::split_path(path_search_hash);

            ParsedUrl {
                href: href.clone(),
                protocol: protocol.to_string(),
                host: host.to_string(),
                hostname: host.split(':').next().unwrap_or(host).to_string(),
                port: host.split(':').nth(1).unwrap_or("").to_string(),
                pathname: pathname.to_string(),
                search: search.to_string(),
                hash: hash.to_string(),
                origin: format!("{}://{}", protocol.trim_end_matches(':'), host),
            }
        } else {
            // Fallback for invalid URLs
            ParsedUrl {
                href: href.clone(),
                protocol: "".to_string(),
                host: "".to_string(),
                hostname: "".to_string(),
                port: "".to_string(),
                pathname: href.clone(),
                search: "".to_string(),
                hash: "".to_string(),
                origin: "null".to_string(),
            }
        }
    }

    fn split_path(path: &str) -> (&str, &str, &str) {
        if let Some(hash_pos) = path.find('#') {
            let (before_hash, hash) = path.split_at(hash_pos);
            if let Some(search_pos) = before_hash.find('?') {
                let (pathname, search) = before_hash.split_at(search_pos);
                (pathname, search, hash)
            } else {
                (before_hash, "", hash)
            }
        } else if let Some(search_pos) = path.find('?') {
            let (pathname, search) = path.split_at(search_pos);
            (pathname, search, "")
        } else {
            (path, "", "")
        }
    }
}

struct ParsedUrl {
    href: String,
    protocol: String,
    host: String,
    hostname: String,
    port: String,
    pathname: String,
    search: String,
    hash: String,
    origin: String,
}

impl IntrinsicObject for Location {
    fn init(realm: &Realm) {
        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .accessor(
                js_string!("href"),
                Some(BuiltInBuilder::callable(realm, Self::get_href).name(js_string!("get href")).build()),
                Some(BuiltInBuilder::callable(realm, Self::set_href).name(js_string!("set href")).build()),
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("protocol"),
                Some(BuiltInBuilder::callable(realm, Self::get_protocol).name(js_string!("get protocol")).build()),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("host"),
                Some(BuiltInBuilder::callable(realm, Self::get_host).name(js_string!("get host")).build()),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("hostname"),
                Some(BuiltInBuilder::callable(realm, Self::get_hostname).name(js_string!("get hostname")).build()),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("port"),
                Some(BuiltInBuilder::callable(realm, Self::get_port).name(js_string!("get port")).build()),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("pathname"),
                Some(BuiltInBuilder::callable(realm, Self::get_pathname).name(js_string!("get pathname")).build()),
                Some(BuiltInBuilder::callable(realm, Self::set_pathname).name(js_string!("set pathname")).build()),
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("search"),
                Some(BuiltInBuilder::callable(realm, Self::get_search).name(js_string!("get search")).build()),
                Some(BuiltInBuilder::callable(realm, Self::set_search).name(js_string!("set search")).build()),
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("hash"),
                Some(BuiltInBuilder::callable(realm, Self::get_hash).name(js_string!("get hash")).build()),
                Some(BuiltInBuilder::callable(realm, Self::set_hash).name(js_string!("set hash")).build()),
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("origin"),
                Some(BuiltInBuilder::callable(realm, Self::get_origin).name(js_string!("get origin")).build()),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .method(Self::assign, js_string!("assign"), 1)
            .method(Self::replace_method, js_string!("replace"), 1)
            .method(Self::reload, js_string!("reload"), 0)
            .method(Self::to_string_method, js_string!("toString"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Location {
    const NAME: JsString = StaticJsStrings::LOCATION;
}

impl BuiltInConstructor for Location {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100; // 9 accessors
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::location;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Constructor Location requires 'new'")
                .into());
        }

        let location_data = LocationData::new();

        let prototype = Self::get(context.intrinsics())
            .get(js_string!("prototype"), context)?
            .as_object()
            .ok_or_else(|| JsNativeError::typ().with_message("Location.prototype is not an object"))?
            .clone();

        let location_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            location_data,
        );

        Ok(location_obj.into())
    }
}

impl Location {
    /// `location.href` getter
    fn get_href(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Location.href getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<LocationData>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Location object")
        })?;

        Ok(JsValue::from(js_string!(data.get_href())))
    }

    /// `location.protocol` getter
    fn get_protocol(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Location.protocol getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<LocationData>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Location object")
        })?;

        let parsed = data.parse_url();
        Ok(JsValue::from(js_string!(parsed.protocol)))
    }

    /// `location.host` getter
    fn get_host(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Location.host getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<LocationData>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Location object")
        })?;

        let parsed = data.parse_url();
        Ok(JsValue::from(js_string!(parsed.host)))
    }

    /// `location.hostname` getter
    fn get_hostname(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Location.hostname getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<LocationData>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Location object")
        })?;

        let parsed = data.parse_url();
        Ok(JsValue::from(js_string!(parsed.hostname)))
    }

    /// `location.port` getter
    fn get_port(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Location.port getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<LocationData>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Location object")
        })?;

        let parsed = data.parse_url();
        Ok(JsValue::from(js_string!(parsed.port)))
    }

    /// `location.pathname` getter
    fn get_pathname(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Location.pathname getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<LocationData>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Location object")
        })?;

        let parsed = data.parse_url();
        Ok(JsValue::from(js_string!(parsed.pathname)))
    }

    /// `location.search` getter
    fn get_search(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Location.search getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<LocationData>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Location object")
        })?;

        let parsed = data.parse_url();
        Ok(JsValue::from(js_string!(parsed.search)))
    }

    /// `location.hash` getter
    fn get_hash(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Location.hash getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<LocationData>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Location object")
        })?;

        let parsed = data.parse_url();
        Ok(JsValue::from(js_string!(parsed.hash)))
    }

    /// `location.origin` getter
    fn get_origin(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Location.origin getter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<LocationData>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Location object")
        })?;

        let parsed = data.parse_url();
        Ok(JsValue::from(js_string!(parsed.origin)))
    }

    /// `location.href` setter — navigates to the given URL
    fn set_href(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Location.href setter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<LocationData>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Location object")
        })?;

        let url = args.get_or_undefined(0).to_string(context)?;
        data.set_href(&url.to_std_string_escaped());
        Ok(JsValue::undefined())
    }

    /// `location.hash` setter
    fn set_hash(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Location.hash setter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<LocationData>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Location object")
        })?;

        let hash = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let parsed = data.parse_url();
        let new_hash = if hash.starts_with('#') { hash } else { format!("#{}", hash) };
        let new_href = format!("{}://{}{}{}{}",
            parsed.protocol.trim_end_matches(':'), parsed.host, parsed.pathname, parsed.search, new_hash);
        data.set_href(&new_href);
        Ok(JsValue::undefined())
    }

    /// `location.search` setter
    fn set_search(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Location.search setter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<LocationData>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Location object")
        })?;

        let search = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let parsed = data.parse_url();
        let new_search = if search.starts_with('?') { search } else { format!("?{}", search) };
        let new_href = format!("{}://{}{}{}{}",
            parsed.protocol.trim_end_matches(':'), parsed.host, parsed.pathname, new_search, parsed.hash);
        data.set_href(&new_href);
        Ok(JsValue::undefined())
    }

    /// `location.pathname` setter
    fn set_pathname(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Location.pathname setter called on non-object")
        })?;

        let data = this_obj.downcast_ref::<LocationData>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not a Location object")
        })?;

        let pathname = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let parsed = data.parse_url();
        let new_href = format!("{}://{}{}{}{}",
            parsed.protocol.trim_end_matches(':'), parsed.host, pathname, parsed.search, parsed.hash);
        data.set_href(&new_href);
        Ok(JsValue::undefined())
    }

    /// `location.assign(url)` — navigate to URL (adds to history)
    fn assign(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::set_href(this, args, context)
    }

    /// `location.replace(url)` — navigate to URL (replaces current entry in history)
    fn replace_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::set_href(this, args, context)
    }

    /// `location.reload()` — reloads the current document
    fn reload(_this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        // In a headless browser, reload is a no-op
        Ok(JsValue::undefined())
    }

    /// `location.toString()` — returns the href
    fn to_string_method(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        Self::get_href(this, args, context)
    }
}
