//! DOMParser implementation for Boa
//!
//! Implements the DOMParser interface as defined in:
//! https://html.spec.whatwg.org/multipage/dynamic-markup-insertion.html#domparser

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, js_string,
};
use boa_gc::{Finalize, Trace};

/// JavaScript `DOMParser` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct DOMParser;

impl IntrinsicObject for DOMParser {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .method(parse_from_string, js_string!("parseFromString"), 2)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for DOMParser {
    const NAME: JsString = StaticJsStrings::DOM_PARSER;
}

impl BuiltInConstructor for DOMParser {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::dom_parser;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("DOMParser constructor requires 'new'")
                .into());
        }

        let proto = get_prototype_from_constructor(new_target, StandardConstructors::dom_parser, context)?;
        let parser_data = DOMParserData::new();
        let parser_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            proto,
            parser_data,
        );

        Ok(parser_obj.upcast().into())
    }
}

/// Internal data for DOMParser instances
#[derive(Debug, Trace, Finalize, JsData)]
pub struct DOMParserData {}

impl DOMParserData {
    pub fn new() -> Self {
        Self {}
    }
}

/// `DOMParser.prototype.parseFromString(string, mimeType)`
fn parse_from_string(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let _string = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let mime_type = args.get_or_undefined(1).to_string(context)?.to_std_string_escaped();

    // Validate mime type
    match mime_type.as_str() {
        "text/html" | "text/xml" | "application/xml" | "application/xhtml+xml" | "image/svg+xml" => {}
        _ => {
            return Err(JsNativeError::typ()
                .with_message(format!("Invalid MIME type: {}", mime_type))
                .into());
        }
    }

    // Create and return a new Document
    let document_constructor = context.intrinsics().constructors().document().constructor();
    crate::dom::document::Document::constructor(
        &document_constructor.clone().into(),
        &[],
        context,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use boa_engine::Source;

    fn create_test_context() -> Context {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
        context
    }

    #[test]
    fn test_dom_parser_exists() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes("typeof DOMParser === 'function'")).unwrap();
        assert_eq!(result.to_boolean(), true);
    }

    #[test]
    fn test_dom_parser_parse_from_string() {
        let mut context = create_test_context();
        let result = context.eval(Source::from_bytes(r#"
            const parser = new DOMParser();
            const doc = parser.parseFromString('<html></html>', 'text/html');
            typeof doc === 'object';
        "#)).unwrap();
        assert_eq!(result.to_boolean(), true);
    }
}
