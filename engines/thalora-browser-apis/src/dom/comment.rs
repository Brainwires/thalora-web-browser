//! Comment interface implementation for DOM Level 4
//!
//! The Comment interface represents textual notations within markup.
//! It inherits from CharacterData and provides no additional methods.
//! https://dom.spec.whatwg.org/#interface-comment

use boa_engine::{
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
    string::{StaticJsStrings, JsString},
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult,
};
use boa_gc::{Finalize, Trace};
use super::{character_data::CharacterDataData, node::{NodeData, NodeType}};

/// Comment data structure for comment DOM nodes
#[derive(Debug, Trace, Finalize, JsData)]
pub struct CommentData {
    // Inherit from CharacterData
    character_data: CharacterDataData,
}

impl CommentData {
    /// Create a new Comment object
    pub fn new(data: String) -> Self {
        let character_data = CharacterDataData::new(NodeType::Comment, data);
        Self { character_data }
    }

    /// Get reference to underlying character data
    pub fn character_data(&self) -> &CharacterDataData {
        &self.character_data
    }

    /// Get reference to underlying node data
    pub fn node_data(&self) -> &NodeData {
        self.character_data.node_data()
    }
}

impl CommentData {
    /// `Comment.prototype.data` getter
    fn get_data_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.data called on non-object")
        })?;

        let comment_data = this_obj.downcast_ref::<CommentData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Comment.data called on non-Comment object")
        })?;

        Ok(JsValue::from(js_string!(comment_data.character_data.get_data())))
    }

    fn set_data_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.data called on non-object")
        })?;

        let data_arg = args.get_or_undefined(0);
        let data_string = data_arg.to_string(context)?;

        let comment_data = this_obj.downcast_ref::<CommentData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Comment.data called on non-Comment object")
        })?;

        comment_data.character_data.set_data(data_string.to_std_string_escaped());
        Ok(JsValue::undefined())
    }

    fn get_length_accessor(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.length called on non-object")
        })?;

        let comment_data = this_obj.downcast_ref::<CommentData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Comment.length called on non-Comment object")
        })?;

        Ok(JsValue::from(comment_data.character_data.get_length()))
    }

    fn substring_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.substringData called on non-object")
        })?;

        let offset = args.get_or_undefined(0).to_u32(context)?;
        let count = args.get_or_undefined(1).to_u32(context)?;

        let comment_data = this_obj.downcast_ref::<CommentData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Comment.substringData called on non-Comment object")
        })?;

        match comment_data.character_data.substring_data_impl(offset, count) {
            Ok(result) => Ok(JsValue::from(js_string!(result))),
            Err(err) => Err(JsNativeError::range().with_message(err).into()),
        }
    }

    fn append_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.appendData called on non-object")
        })?;

        let data_string = args.get_or_undefined(0).to_string(context)?;

        let comment_data = this_obj.downcast_ref::<CommentData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Comment.appendData called on non-Comment object")
        })?;

        comment_data.character_data.append_data_impl(data_string.to_std_string_escaped());
        Ok(JsValue::undefined())
    }

    fn insert_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.insertData called on non-object")
        })?;

        let offset = args.get_or_undefined(0).to_u32(context)?;
        let data_string = args.get_or_undefined(1).to_string(context)?;

        let comment_data = this_obj.downcast_ref::<CommentData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Comment.insertData called on non-Comment object")
        })?;

        match comment_data.character_data.insert_data_impl(offset, data_string.to_std_string_escaped()) {
            Ok(_) => Ok(JsValue::undefined()),
            Err(err) => Err(JsNativeError::range().with_message(err).into()),
        }
    }

    fn delete_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.deleteData called on non-object")
        })?;

        let offset = args.get_or_undefined(0).to_u32(context)?;
        let count = args.get_or_undefined(1).to_u32(context)?;

        let comment_data = this_obj.downcast_ref::<CommentData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Comment.deleteData called on non-Comment object")
        })?;

        match comment_data.character_data.delete_data_impl(offset, count) {
            Ok(_) => Ok(JsValue::undefined()),
            Err(err) => Err(JsNativeError::range().with_message(err).into()),
        }
    }

    fn replace_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let this_obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("Comment.replaceData called on non-object")
        })?;

        let offset = args.get_or_undefined(0).to_u32(context)?;
        let count = args.get_or_undefined(1).to_u32(context)?;
        let data_string = args.get_or_undefined(2).to_string(context)?;

        let comment_data = this_obj.downcast_ref::<CommentData>().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("Comment.replaceData called on non-Comment object")
        })?;

        match comment_data.character_data.replace_data_impl(offset, count, data_string.to_std_string_escaped()) {
            Ok(_) => Ok(JsValue::undefined()),
            Err(err) => Err(JsNativeError::range().with_message(err).into()),
        }
    }
}

/// The `Comment` object
#[derive(Debug, Trace, Finalize)]
pub struct Comment;

impl Comment {
    fn get_data_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CommentData::get_data_accessor(this, args, context)
    }

    fn set_data_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CommentData::set_data_accessor(this, args, context)
    }

    fn get_length_accessor(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CommentData::get_length_accessor(this, args, context)
    }

    fn substring_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CommentData::substring_data(this, args, context)
    }

    fn append_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CommentData::append_data(this, args, context)
    }

    fn insert_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CommentData::insert_data(this, args, context)
    }

    fn delete_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CommentData::delete_data(this, args, context)
    }

    fn replace_data(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        CommentData::replace_data(this, args, context)
    }
}

impl IntrinsicObject for Comment {
    fn init(realm: &Realm) {
        let data_get_func = BuiltInBuilder::callable(realm, Self::get_data_accessor)
            .name(js_string!("get data"))
            .build();
        let data_set_func = BuiltInBuilder::callable(realm, Self::set_data_accessor)
            .name(js_string!("set data"))
            .build();
        let length_get_func = BuiltInBuilder::callable(realm, Self::get_length_accessor)
            .name(js_string!("get length"))
            .build();

        let _constructor = BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Set up prototype chain: Comment -> CharacterData -> Node -> EventTarget
            .inherits(Some(realm.intrinsics().constructors().character_data().prototype()))
            // CharacterData methods
            .method(Self::substring_data, js_string!("substringData"), 2)
            .method(Self::append_data, js_string!("appendData"), 1)
            .method(Self::insert_data, js_string!("insertData"), 2)
            .method(Self::delete_data, js_string!("deleteData"), 2)
            .method(Self::replace_data, js_string!("replaceData"), 3)
            // CharacterData properties
            .accessor(
                js_string!("data"),
                Some(data_get_func),
                Some(data_set_func),
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .accessor(
                js_string!("length"),
                Some(length_get_func),
                None,
                Attribute::CONFIGURABLE | Attribute::ENUMERABLE,
            )
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for Comment {
    const NAME: JsString = StaticJsStrings::COMMENT;
}

impl BuiltInConstructor for Comment {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 100;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::comment;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        // Comment constructor should be called with 'new'
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("Constructor Comment requires 'new'")
                .into());
        }

        // Get optional data argument, default to empty string
        let data_arg = args.get_or_undefined(0);
        let data_string = if data_arg.is_undefined() {
            String::new()
        } else {
            data_arg.to_string(context)?.to_std_string_escaped()
        };

        // Create a new Comment object
        let comment_data = CommentData::new(data_string);

        let comment_obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().comment().prototype(),
            comment_data,
        );

        Ok(comment_obj.into())
    }
}
