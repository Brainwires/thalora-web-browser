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

// Web Locks API
pub mod locks;

/// Initialize all browser APIs in a Boa context
pub fn initialize_browser_apis(context: &mut boa_engine::Context) -> JsResult<()> {
    use boa_engine::builtins::{IntrinsicObject, BuiltInObject};
    use boa_engine::property::PropertyDescriptor;

    let realm = context.realm().clone();

    // Initialize Console API (must be early for debugging)
    console::console::Console::init(context);

    // Initialize Timer APIs (foundational for async operations)
    timers::timers::Timers::init(context);

    // Initialize Event APIs (foundational for DOM)
    events::event::Event::init(&realm);
    events::event_target::EventTarget::init(&realm);

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

    // Initialize Web Locks API
    locks::lock_manager::LockManager::init(&realm);

    // Initialize File APIs
    file::blob::Blob::init(&realm);
    file::file::File::init(&realm);
    file::file_reader::FileReader::init(&realm);
    file::file_system::FileSystemFileHandle::init(&realm);
    file::file_system::FileSystemDirectoryHandle::init(&realm);

    // Initialize Crypto API
    crypto::crypto::Crypto::init(context);

    // Initialize Fetch APIs
    fetch::fetch::Fetch::init(&realm);
    fetch::fetch::Request::init(&realm);
    fetch::fetch::Response::init(&realm);
    fetch::fetch::Headers::init(&realm);

    // Register browser APIs as global properties
    let global_object = context.global_object();

    // Browser APIs - Navigator needs to be registered
    global_object.define_property_or_throw(
        browser::navigator::Navigator::NAME,
        PropertyDescriptor::builder()
            .value(browser::navigator::Navigator::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Event APIs
    global_object.define_property_or_throw(
        events::event::Event::NAME,
        PropertyDescriptor::builder()
            .value(events::event::Event::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        events::event_target::EventTarget::NAME,
        PropertyDescriptor::builder()
            .value(events::event_target::EventTarget::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Fetch APIs
    global_object.define_property_or_throw(
        js_string!("fetch"),
        PropertyDescriptor::builder()
            .value(fetch::fetch::Fetch::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        fetch::fetch::Request::NAME,
        PropertyDescriptor::builder()
            .value(fetch::fetch::Request::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        fetch::fetch::Response::NAME,
        PropertyDescriptor::builder()
            .value(fetch::fetch::Response::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        fetch::fetch::Headers::NAME,
        PropertyDescriptor::builder()
            .value(fetch::fetch::Headers::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // File APIs
    global_object.define_property_or_throw(
        file::blob::Blob::NAME,
        PropertyDescriptor::builder()
            .value(file::blob::Blob::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        file::file::File::NAME,
        PropertyDescriptor::builder()
            .value(file::file::File::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        file::file_reader::FileReader::NAME,
        PropertyDescriptor::builder()
            .value(file::file_reader::FileReader::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

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

    // Register LockManager constructor
    global_object.define_property_or_throw(
        locks::lock_manager::LockManager::NAME,
        PropertyDescriptor::builder()
            .value(locks::lock_manager::LockManager::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Create global browser environment objects (moved from Boa test harness)
    use boa_engine::builtins::BuiltInConstructor;

    // Create a global document instance
    let document_constructor = dom::document::Document::get(context.intrinsics());
    let document_args = [];
    let document_instance = dom::document::Document::constructor(
        &document_constructor.clone().into(),
        &document_args,
        context,
    ).expect("failed to create document instance");

    global_object.set(boa_engine::js_string!("document"), document_instance, false, context)
        .expect("failed to set global document");

    // Create a global window instance
    let window_constructor = browser::window::Window::get(context.intrinsics());
    let window_args = [];
    let window_instance = browser::window::Window::constructor(
        &window_constructor.clone().into(),
        &window_args,
        context,
    ).expect("failed to create window instance");

    global_object.set(boa_engine::js_string!("window"), window_instance.clone(), false, context)
        .expect("failed to set global window");

    // Create navigator instance manually with proper prototype and data
    let navigator_proto = global_object.get(browser::navigator::Navigator::NAME, context)?
        .as_object()
        .and_then(|ctor| ctor.get(boa_engine::js_string!("prototype"), context).ok())
        .and_then(|proto| proto.as_object().map(|obj| obj.clone()))
        .ok_or_else(|| boa_engine::JsNativeError::typ().with_message("Navigator prototype not found"))?;

    let mut navigator_data = browser::navigator::Navigator::new();

    // Create StorageManager instance for navigator.storage
    let storage_manager_proto = global_object.get(storage::storage_manager::StorageManager::NAME, context)?
        .as_object()
        .and_then(|ctor| ctor.get(boa_engine::js_string!("prototype"), context).ok())
        .and_then(|proto| proto.as_object().map(|obj| obj.clone()))
        .ok_or_else(|| boa_engine::JsNativeError::typ().with_message("StorageManager prototype not found"))?;

    let storage_manager_data = storage::storage_manager::StorageManager::new();
    let storage_manager_obj = boa_engine::JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        storage_manager_proto,
        storage_manager_data,
    );

    // Create LockManager instance for navigator.locks
    let lock_manager_proto = global_object.get(locks::lock_manager::LockManager::NAME, context)?
        .as_object()
        .and_then(|ctor| ctor.get(boa_engine::js_string!("prototype"), context).ok())
        .and_then(|proto| proto.as_object().map(|obj| obj.clone()))
        .ok_or_else(|| boa_engine::JsNativeError::typ().with_message("LockManager prototype not found"))?;

    let lock_manager_data = locks::lock_manager::LockManager::new();
    let lock_manager_obj = boa_engine::JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        lock_manager_proto,
        lock_manager_data,
    );

    // Set lock_manager on navigator_data BEFORE creating the object
    eprintln!("DEBUG: Setting lock_manager on Navigator data structure");
    navigator_data.set_lock_manager(lock_manager_obj.clone());

    // Now create the navigator object with the data that includes lock_manager
    let navigator_obj = boa_engine::JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        navigator_proto,
        navigator_data,
    );

    // Set navigator.storage (this works with .set() because storage doesn't have a special getter)
    navigator_obj.set(
        boa_engine::js_string!("storage"),
        storage_manager_obj,
        false,
        context
    ).expect("failed to set navigator.storage");

    let check_storage = navigator_obj.get(boa_engine::js_string!("storage"), context).unwrap();
    eprintln!("DEBUG: After setting navigator.storage, get('storage') = {:?}", check_storage);

    let navigator_instance: boa_engine::JsValue = navigator_obj.into();

    // Now set navigator on the window object (after setting storage and locks)
    if let Some(window_obj) = window_instance.as_object() {
        if let Some(window_data) = window_obj.downcast_ref::<browser::window::WindowData>() {
            if let Some(nav_obj) = navigator_instance.as_object() {
                window_data.set_navigator(nav_obj.clone());
            }
        }
    }

    // Also set as global navigator
    global_object.set(boa_engine::js_string!("navigator"), navigator_instance.clone(), false, context)
        .expect("failed to set global navigator");

    // Create localStorage and sessionStorage instances
    // Storage constructor cannot be called directly, so we create instances manually
    let storage_proto = global_object.get(storage::storage::Storage::NAME, context)?
        .as_object()
        .and_then(|ctor| ctor.get(boa_engine::js_string!("prototype"), context).ok())
        .and_then(|proto| proto.as_object().map(|obj| obj.clone()))
        .ok_or_else(|| boa_engine::JsNativeError::typ().with_message("Storage prototype not found"))?;

    // Create localStorage instance with persistence enabled
    let local_storage_data = storage::storage::Storage::new("localStorage");
    let local_storage_obj = boa_engine::JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        storage_proto.clone(),
        local_storage_data,
    );
    let local_storage: boa_engine::JsValue = local_storage_obj.into();

    // Create sessionStorage instance with persistence enabled
    let session_storage_data = storage::storage::Storage::new("sessionStorage");
    let session_storage_obj = boa_engine::JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        storage_proto,
        session_storage_data,
    );
    let session_storage: boa_engine::JsValue = session_storage_obj.into();

    // Set on window object
    if let Some(window_obj) = window_instance.as_object() {
        window_obj.set(boa_engine::js_string!("localStorage"), local_storage.clone(), false, context)
            .expect("failed to set window.localStorage");
        window_obj.set(boa_engine::js_string!("sessionStorage"), session_storage.clone(), false, context)
            .expect("failed to set window.sessionStorage");
    }

    // Set as globals
    global_object.set(boa_engine::js_string!("localStorage"), local_storage, false, context)
        .expect("failed to set global localStorage");
    global_object.set(boa_engine::js_string!("sessionStorage"), session_storage, false, context)
        .expect("failed to set global sessionStorage");

    // Register file picker global functions
    use boa_engine::object::FunctionObjectBuilder;

    let show_open_file_picker_fn = FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_fn_ptr(file::file_system::show_open_file_picker),
    )
    .name("showOpenFilePicker")
    .length(0)
    .build();
    global_object.set(
        boa_engine::js_string!("showOpenFilePicker"),
        show_open_file_picker_fn,
        false,
        context,
    )?;

    let show_save_file_picker_fn = FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_fn_ptr(file::file_system::show_save_file_picker),
    )
    .name("showSaveFilePicker")
    .length(0)
    .build();
    global_object.set(
        boa_engine::js_string!("showSaveFilePicker"),
        show_save_file_picker_fn,
        false,
        context,
    )?;

    let show_directory_picker_fn = FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_fn_ptr(file::file_system::show_directory_picker),
    )
    .name("showDirectoryPicker")
    .length(0)
    .build();
    global_object.set(
        boa_engine::js_string!("showDirectoryPicker"),
        show_directory_picker_fn,
        false,
        context,
    )?;

    // Add 'self' reference to global scope (Worker/browser compatibility)
    global_object.set(js_string!("self"), global_object.clone(), false, context)?;

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
        crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs in test context");

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
#[cfg(test)]
mod debug_navigator_locks {
    use crate::{Context, Source, JsValue, JsString};

    #[test]
    fn debug_locks() {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).unwrap();
        
        println!("=== Testing LockManager ===");
        
        // 1. Does LockManager constructor exist?
        let result = context.eval(Source::from_bytes("typeof LockManager")).unwrap();
        println!("1. typeof LockManager = {:?}", result);
        
        // 2. Does LockManager.prototype.request exist?
        let result = context.eval(Source::from_bytes("typeof LockManager.prototype.request")).unwrap();
        println!("2. typeof LockManager.prototype.request = {:?}", result);
        
        // 3. Does global navigator exist?
        let result = context.eval(Source::from_bytes("typeof navigator")).unwrap();
        println!("3. typeof navigator = {:?}", result);
        
        // 4. Does navigator.locks exist?
        let result = context.eval(Source::from_bytes("typeof navigator.locks")).unwrap();
        println!("4. typeof navigator.locks = {:?}", result);
        
        // 5. Are navigator and window.navigator the same object?
        let result = context.eval(Source::from_bytes("navigator === window.navigator")).unwrap();
        println!("5. navigator === window.navigator: {:?}", result);
        
        // 6. Does window.navigator have locks?
        let result = context.eval(Source::from_bytes("typeof window.navigator.locks")).unwrap();
        println!("6. typeof window.navigator.locks = {:?}", result);
        
        // 7. Check what properties navigator has
        let result = context.eval(Source::from_bytes("Object.keys(navigator)")).unwrap();
        println!("7. Object.keys(navigator) = {:?}", result);

        // 8. Does navigator.storage work?
        let result = context.eval(Source::from_bytes("typeof navigator.storage")).unwrap();
        println!("8. typeof navigator.storage = {:?}", result);

        // 9. Does window.navigator.storage work?
        let result = context.eval(Source::from_bytes("typeof window.navigator.storage")).unwrap();
        println!("9. typeof window.navigator.storage = {:?}", result);
    }
}
