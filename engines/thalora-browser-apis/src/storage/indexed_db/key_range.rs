//! IDBKeyRange implementation
//!
//! Represents a continuous interval over some data type that is used for keys.
//! Records can be retrieved from object stores and indexes using keys or a range of keys.
//!
//! Spec: https://w3c.github.io/IndexedDB/#keyrange

use super::key::IDBKey;
use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString, JsValue,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor},
    js_string,
    object::JsObject,
    property::Attribute,
    realm::Realm,
};
use boa_gc::{Finalize, Trace};

/// IDBKeyRange represents a continuous interval over some data type used for keys
#[derive(Debug, Clone, Finalize)]
pub struct IDBKeyRange {
    /// Lower bound of the key range (None = unbounded)
    lower: Option<IDBKey>,

    /// Upper bound of the key range (None = unbounded)
    upper: Option<IDBKey>,

    /// Whether the lower bound is open (exclusive)
    lower_open: bool,

    /// Whether the upper bound is open (exclusive)
    upper_open: bool,
}

unsafe impl Trace for IDBKeyRange {
    unsafe fn trace(&self, _tracer: &mut boa_gc::Tracer) {
        // IDBKey doesn't contain GC'd objects
    }

    unsafe fn trace_non_roots(&self) {}

    fn run_finalizer(&self) {}
}

impl JsData for IDBKeyRange {}

impl IDBKeyRange {
    /// Create a new key range
    pub fn new(
        lower: Option<IDBKey>,
        upper: Option<IDBKey>,
        lower_open: bool,
        upper_open: bool,
    ) -> Result<Self, String> {
        // Validate that lower <= upper if both exist
        if let (Some(l), Some(u)) = (&lower, &upper) {
            if l > u {
                return Err("Lower bound cannot be greater than upper bound".to_string());
            }
            if l == u && (lower_open || upper_open) {
                return Err(
                    "Lower and upper bounds cannot be equal when either is open".to_string()
                );
            }
        }

        Ok(Self {
            lower,
            upper,
            lower_open,
            upper_open,
        })
    }

    /// Check if a key is within this range
    pub fn includes(&self, key: &IDBKey) -> bool {
        // Check lower bound
        if let Some(lower) = &self.lower {
            if self.lower_open {
                if key <= lower {
                    return false;
                }
            } else {
                if key < lower {
                    return false;
                }
            }
        }

        // Check upper bound
        if let Some(upper) = &self.upper {
            if self.upper_open {
                if key >= upper {
                    return false;
                }
            } else {
                if key > upper {
                    return false;
                }
            }
        }

        true
    }

    /// IDBKeyRange.only(value)
    /// Creates a new key range containing a single value
    pub(crate) fn only(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let value = args.get_or_undefined(0);
        let key = IDBKey::from_js_value(value, context)?;

        let range = IDBKeyRange {
            lower: Some(key.clone()),
            upper: Some(key),
            lower_open: false,
            upper_open: false,
        };

        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            range,
        );

        Ok(obj.into())
    }

    /// IDBKeyRange.lowerBound(lower, open?)
    /// Creates a new key range with only a lower bound
    pub(crate) fn lower_bound(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let lower_val = args.get_or_undefined(0);
        let open = args.get_or_undefined(1).to_boolean();

        let lower = IDBKey::from_js_value(lower_val, context)?;

        let range = IDBKeyRange {
            lower: Some(lower),
            upper: None,
            lower_open: open,
            upper_open: false,
        };

        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            range,
        );

        Ok(obj.into())
    }

    /// IDBKeyRange.upperBound(upper, open?)
    /// Creates a new key range with only an upper bound
    pub(crate) fn upper_bound(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let upper_val = args.get_or_undefined(0);
        let open = args.get_or_undefined(1).to_boolean();

        let upper = IDBKey::from_js_value(upper_val, context)?;

        let range = IDBKeyRange {
            lower: None,
            upper: Some(upper),
            lower_open: false,
            upper_open: open,
        };

        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            range,
        );

        Ok(obj.into())
    }

    /// IDBKeyRange.bound(lower, upper, lowerOpen?, upperOpen?)
    /// Creates a new key range with both lower and upper bounds
    pub(crate) fn bound(
        _this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let lower_val = args.get_or_undefined(0);
        let upper_val = args.get_or_undefined(1);
        let lower_open = args.get_or_undefined(2).to_boolean();
        let upper_open = args.get_or_undefined(3).to_boolean();

        let lower = IDBKey::from_js_value(lower_val, context)?;
        let upper = IDBKey::from_js_value(upper_val, context)?;

        // Validate bounds
        if lower > upper {
            return Err(JsNativeError::typ()
                .with_message("Lower bound cannot be greater than upper bound")
                .into());
        }
        if lower == upper && (lower_open || upper_open) {
            return Err(JsNativeError::typ()
                .with_message("Lower and upper bounds cannot be equal when either is open")
                .into());
        }

        let range = IDBKeyRange {
            lower: Some(lower),
            upper: Some(upper),
            lower_open,
            upper_open,
        };

        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            context.intrinsics().constructors().object().prototype(),
            range,
        );

        Ok(obj.into())
    }

    /// Get lower bound
    pub(crate) fn get_lower(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBKeyRange object")
        })?;

        let range = obj.downcast_ref::<IDBKeyRange>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBKeyRange object")
        })?;

        match &range.lower {
            Some(key) => key.to_js_value(context),
            None => Ok(JsValue::undefined()),
        }
    }

    /// Get upper bound
    pub(crate) fn get_upper(
        this: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBKeyRange object")
        })?;

        let range = obj.downcast_ref::<IDBKeyRange>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBKeyRange object")
        })?;

        match &range.upper {
            Some(key) => key.to_js_value(context),
            None => Ok(JsValue::undefined()),
        }
    }

    /// Get lowerOpen
    pub(crate) fn get_lower_open(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBKeyRange object")
        })?;

        let range = obj.downcast_ref::<IDBKeyRange>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBKeyRange object")
        })?;

        Ok(JsValue::from(range.lower_open))
    }

    /// Get upperOpen
    pub(crate) fn get_upper_open(
        this: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBKeyRange object")
        })?;

        let range = obj.downcast_ref::<IDBKeyRange>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBKeyRange object")
        })?;

        Ok(JsValue::from(range.upper_open))
    }

    /// IDBKeyRange.includes(key)
    /// Returns whether a given key is within the range
    pub(crate) fn includes_method(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let obj = this.as_object().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBKeyRange object")
        })?;

        let range = obj.downcast_ref::<IDBKeyRange>().ok_or_else(|| {
            JsNativeError::typ().with_message("'this' is not an IDBKeyRange object")
        })?;

        let key_val = args.get_or_undefined(0);
        let key = IDBKey::from_js_value(key_val, context)?;

        Ok(JsValue::from(range.includes(&key)))
    }

    /// Getters
    pub fn lower(&self) -> Option<&IDBKey> {
        self.lower.as_ref()
    }

    pub fn upper(&self) -> Option<&IDBKey> {
        self.upper.as_ref()
    }

    pub fn is_lower_open(&self) -> bool {
        self.lower_open
    }

    pub fn is_upper_open(&self) -> bool {
        self.upper_open
    }
}

impl IntrinsicObject for IDBKeyRange {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Static methods
            .static_method(Self::only, js_string!("only"), 1)
            .static_method(Self::lower_bound, js_string!("lowerBound"), 1)
            .static_method(Self::upper_bound, js_string!("upperBound"), 1)
            .static_method(Self::bound, js_string!("bound"), 2)
            // Instance properties with getters
            .accessor(
                js_string!("lower"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_lower)
                        .name(js_string!("get lower"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("upper"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_upper)
                        .name(js_string!("get upper"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("lowerOpen"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_lower_open)
                        .name(js_string!("get lowerOpen"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("upperOpen"),
                Some(
                    BuiltInBuilder::callable(realm, Self::get_upper_open)
                        .name(js_string!("get upperOpen"))
                        .build(),
                ),
                None,
                Attribute::CONFIGURABLE,
            )
            // Instance methods
            .method(Self::includes_method, js_string!("includes"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for IDBKeyRange {
    const NAME: JsString = js_string!("IDBKeyRange");
}

impl BuiltInConstructor for IDBKeyRange {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 100; // 4 static methods: only, lowerBound, upperBound, bound
    const PROTOTYPE_STORAGE_SLOTS: usize = 100; // 4 accessors + 1 method (lower, upper, lowerOpen, upperOpen, includes)

    const STANDARD_CONSTRUCTOR: fn(
        &boa_engine::context::intrinsics::StandardConstructors,
    ) -> &StandardConstructor = |constructors| constructors.idb_key_range();

    fn constructor(
        _new_target: &JsValue,
        _args: &[JsValue],
        _context: &mut Context,
    ) -> JsResult<JsValue> {
        // IDBKeyRange cannot be constructed directly
        Err(JsNativeError::typ()
            .with_message("Illegal constructor")
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_range_only() {
        let key = IDBKey::Number(42.0);
        let range = IDBKeyRange::new(Some(key.clone()), Some(key.clone()), false, false).unwrap();

        assert!(range.includes(&IDBKey::Number(42.0)));
        assert!(!range.includes(&IDBKey::Number(41.0)));
        assert!(!range.includes(&IDBKey::Number(43.0)));
    }

    #[test]
    fn test_key_range_lower_bound() {
        let range = IDBKeyRange::new(Some(IDBKey::Number(10.0)), None, false, false).unwrap();

        assert!(range.includes(&IDBKey::Number(10.0)));
        assert!(range.includes(&IDBKey::Number(100.0)));
        assert!(!range.includes(&IDBKey::Number(9.0)));
    }

    #[test]
    fn test_key_range_upper_bound() {
        let range = IDBKeyRange::new(None, Some(IDBKey::Number(100.0)), false, false).unwrap();

        assert!(range.includes(&IDBKey::Number(100.0)));
        assert!(range.includes(&IDBKey::Number(10.0)));
        assert!(!range.includes(&IDBKey::Number(101.0)));
    }

    #[test]
    fn test_key_range_bound() {
        let range = IDBKeyRange::new(
            Some(IDBKey::Number(10.0)),
            Some(IDBKey::Number(100.0)),
            false,
            false,
        )
        .unwrap();

        assert!(range.includes(&IDBKey::Number(10.0)));
        assert!(range.includes(&IDBKey::Number(50.0)));
        assert!(range.includes(&IDBKey::Number(100.0)));
        assert!(!range.includes(&IDBKey::Number(9.0)));
        assert!(!range.includes(&IDBKey::Number(101.0)));
    }

    #[test]
    fn test_key_range_open_bounds() {
        let range = IDBKeyRange::new(
            Some(IDBKey::Number(10.0)),
            Some(IDBKey::Number(100.0)),
            true, // lower open
            true, // upper open
        )
        .unwrap();

        assert!(!range.includes(&IDBKey::Number(10.0))); // Lower bound excluded
        assert!(range.includes(&IDBKey::Number(50.0)));
        assert!(!range.includes(&IDBKey::Number(100.0))); // Upper bound excluded
    }
}
