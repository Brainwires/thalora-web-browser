//! Thalora Browser APIs
//!
//! Web standard APIs extracted from Boa engine fork for use in Thalora browser.

// Re-export Boa engine types needed for API bindings
pub use boa_engine;

// DOM APIs
pub mod dom;

// Fetch & Networking APIs
pub mod fetch;

// Storage APIs
pub mod storage;

// Web Workers
pub mod worker;

// File APIs
pub mod file;

// Event APIs
pub mod events;

// Browser objects
pub mod browser;

// Crypto APIs
pub mod crypto;

// Console API
pub mod console;

// Timer APIs
pub mod timers;

// WebRTC APIs
pub mod webrtc;

// Streams APIs
pub mod streams;

// Observer APIs
pub mod observers;

// Messaging APIs
pub mod messaging;

// Miscellaneous APIs
pub mod misc;

/// Initialize all browser APIs in a Boa context
pub fn initialize_browser_apis(context: &mut boa_engine::Context) -> anyhow::Result<()> {
    use boa_engine::builtins::{IntrinsicObject, BuiltInObject};
    use boa_engine::property::PropertyDescriptor;

    let realm = context.realm().clone();

    // Initialize DOM APIs
    dom::node::Node::init(&realm);
    dom::attr::Attr::init(&realm);
    dom::nodelist::NodeList::init(&realm);
    dom::domtokenlist::DOMTokenList::init(&realm);
    dom::document::Document::init(&realm);
    dom::element::Element::init(&realm);

    // Initialize Browser APIs
    browser::navigator::Navigator::init(&realm);
    browser::window::Window::init(&realm);
    browser::history::History::init(&realm);
    browser::performance::Performance::init(&realm);

    // Initialize Storage APIs
    storage::storage::Storage::init(&realm);
    storage::storage_event::StorageEvent::init(&realm);
    storage::storage_manager::StorageManager::init(&realm);

    // Initialize File APIs
    file::file_system::FileSystemFileHandle::init(&realm);
    file::file_system::FileSystemDirectoryHandle::init(&realm);

    // Register browser APIs as global properties
    let global_object = context.global_object();

    // DOM APIs
    global_object.define_property_or_throw(
        dom::node::Node::NAME,
        PropertyDescriptor::builder()
            .value(dom::node::Node::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        dom::attr::Attr::NAME,
        PropertyDescriptor::builder()
            .value(dom::attr::Attr::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        dom::nodelist::NodeList::NAME,
        PropertyDescriptor::builder()
            .value(dom::nodelist::NodeList::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        dom::domtokenlist::DOMTokenList::NAME,
        PropertyDescriptor::builder()
            .value(dom::domtokenlist::DOMTokenList::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        dom::document::Document::NAME,
        PropertyDescriptor::builder()
            .value(dom::document::Document::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        dom::element::Element::NAME,
        PropertyDescriptor::builder()
            .value(dom::element::Element::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Browser APIs
    global_object.define_property_or_throw(
        browser::navigator::Navigator::NAME,
        PropertyDescriptor::builder()
            .value(browser::navigator::Navigator::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        storage::storage::Storage::NAME,
        PropertyDescriptor::builder()
            .value(storage::storage::Storage::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        storage::storage_event::StorageEvent::NAME,
        PropertyDescriptor::builder()
            .value(storage::storage_event::StorageEvent::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        storage::storage_manager::StorageManager::NAME,
        PropertyDescriptor::builder()
            .value(storage::storage_manager::StorageManager::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // TODO: Register other APIs as needed

    Ok(())
}

// Re-export commonly used Boa types for tests
pub use boa_engine::{Context, JsValue, JsString, Source, js_string, JsResult, JsNativeError};

/// Test infrastructure
#[cfg(test)]
pub use test_utils::*;

#[cfg(test)]
pub use boa_macros::js_str;

#[cfg(test)]
mod test_utils {
    use boa_engine::{Context, JsResult, JsValue, js_string};
    use std::fmt;

    /// Error kinds for test assertions
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum JsNativeErrorKind {
        Type,
        Reference,
        Range,
        Syntax,
        Error,
    }

    /// Test action for declarative testing
    pub enum TestAction {
        /// Run arbitrary JavaScript code
        Run(&'static str),
        /// Assert that code evaluates to true
        Assert(&'static str),
        /// Assert that two values are equal
        AssertEq(&'static str, JsValue),
        /// Assert that code throws an error (with optional message)
        AssertThrows(&'static str, JsNativeErrorKind, Option<&'static str>),
    }

    impl TestAction {
        /// Create a Run action
        pub fn run(code: &'static str) -> Self {
            TestAction::Run(code)
        }

        /// Create an Assert action
        pub fn assert(code: &'static str) -> Self {
            TestAction::Assert(code)
        }

        /// Create an AssertEq action
        pub fn assert_eq<V: Into<JsValue>>(code: &'static str, expected: V) -> Self {
            TestAction::AssertEq(code, expected.into())
        }

        /// Create an AssertThrows action (called assert_native_error in tests)
        pub fn assert_native_error(code: &'static str, kind: JsNativeErrorKind, message: &'static str) -> Self {
            TestAction::AssertThrows(code, kind, Some(message))
        }
    }

    impl fmt::Debug for TestAction {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                TestAction::Run(code) => write!(f, "Run({})", code),
                TestAction::Assert(code) => write!(f, "Assert({})", code),
                TestAction::AssertEq(code, _) => write!(f, "AssertEq({}, ...)", code),
                TestAction::AssertThrows(code, kind, msg) => {
                    if let Some(msg) = msg {
                        write!(f, "AssertThrows({}, {:?}, {})", code, kind, msg)
                    } else {
                        write!(f, "AssertThrows({}, {:?})", code, kind)
                    }
                }
            }
        }
    }

    /// Run a series of test actions
    pub fn run_test_actions<const N: usize>(actions: [TestAction; N]) {
        let mut context = Context::default();

        // Initialize all browser APIs that the test might need
        crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");

        for action in actions {
            match action {
                TestAction::Run(code) => {
                    if let Err(e) = context.eval(boa_engine::Source::from_bytes(code)) {
                        panic!("Failed to run '{}': {:?}", code, e);
                    }
                }
                TestAction::Assert(code) => {
                    match context.eval(boa_engine::Source::from_bytes(code)) {
                        Ok(value) => {
                            if !value.to_boolean() {
                                panic!("Assertion failed: '{}' was not true", code);
                            }
                        }
                        Err(e) => {
                            panic!("Failed to evaluate '{}': {:?}", code, e);
                        }
                    }
                }
                TestAction::AssertEq(code, expected) => {
                    match context.eval(boa_engine::Source::from_bytes(code)) {
                        Ok(value) => {
                            if !js_values_equal(&value, &expected, &mut context) {
                                panic!(
                                    "Assertion failed: '{}' = {:?}, expected {:?}",
                                    code, value, expected
                                );
                            }
                        }
                        Err(e) => {
                            panic!("Failed to evaluate '{}': {:?}", code, e);
                        }
                    }
                }
                TestAction::AssertThrows(code, _kind, _message) => {
                    match context.eval(boa_engine::Source::from_bytes(code)) {
                        Ok(value) => {
                            panic!("Expected '{}' to throw, but got: {:?}", code, value);
                        }
                        Err(_) => {
                            // Success - it threw as expected
                            // TODO: Could validate error kind and message here
                        }
                    }
                }
            }
        }
    }

    /// Compare two JsValues for equality
    fn js_values_equal(a: &JsValue, b: &JsValue, context: &mut Context) -> bool {
        // Handle null and undefined
        if a.is_null() && b.is_null() {
            return true;
        }
        if a.is_undefined() && b.is_undefined() {
            return true;
        }

        // Handle numbers
        if let (Some(a_num), Some(b_num)) = (a.as_number(), b.as_number()) {
            return (a_num - b_num).abs() < f64::EPSILON || (a_num.is_nan() && b_num.is_nan());
        }

        // Handle booleans
        if let (Some(a_bool), Some(b_bool)) = (a.as_boolean(), b.as_boolean()) {
            return a_bool == b_bool;
        }

        // Handle strings
        if let (Some(a_str), Some(b_str)) = (a.as_string(), b.as_string()) {
            return a_str.to_std_string_escaped() == b_str.to_std_string_escaped();
        }

        // Use JavaScript == comparison for everything else
        match a.equals(b, context) {
            Ok(result) => result,
            Err(_) => false,
        }
    }
}
