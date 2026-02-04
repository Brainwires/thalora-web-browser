//! Comprehensive test suite for Observer APIs
//! Tests IntersectionObserver, MutationObserver, ResizeObserver, and PerformanceObserver

use crate::boa_engine::{Context, Source, JsValue};
use crate::boa_engine::string::JsString;

// Helper to initialize context with browser APIs
fn create_test_context() -> Context {
    let mut context = Context::default();
    crate::initialize_browser_apis(&mut context)
        .expect("Failed to initialize browser APIs");
    context
}

// ============================================================================
// IntersectionObserver Tests
// ============================================================================

#[test]
fn test_intersection_observer_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof IntersectionObserver")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_intersection_observer_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new IntersectionObserver(() => {});
        observer !== null && observer !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intersection_observer_observe_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new IntersectionObserver(() => {});
        typeof observer.observe === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intersection_observer_unobserve_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new IntersectionObserver(() => {});
        typeof observer.unobserve === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intersection_observer_disconnect_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new IntersectionObserver(() => {});
        typeof observer.disconnect === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intersection_observer_takerecords_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new IntersectionObserver(() => {});
        typeof observer.takeRecords === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intersection_observer_with_options() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new IntersectionObserver(() => {}, {
            root: null,
            rootMargin: '0px',
            threshold: 0.5
        });
        observer !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// MutationObserver Tests
// ============================================================================

#[test]
fn test_mutation_observer_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof MutationObserver")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_mutation_observer_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new MutationObserver(() => {});
        observer !== null && observer !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_mutation_observer_observe_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new MutationObserver(() => {});
        typeof observer.observe === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_mutation_observer_disconnect_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new MutationObserver(() => {});
        typeof observer.disconnect === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_mutation_observer_takerecords_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new MutationObserver(() => {});
        typeof observer.takeRecords === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_mutation_observer_with_callback() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let called = false;
        let observer = new MutationObserver((mutations) => {
            called = true;
        });
        observer !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// ResizeObserver Tests
// ============================================================================

#[test]
fn test_resize_observer_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof ResizeObserver")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_resize_observer_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new ResizeObserver(() => {});
        observer !== null && observer !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_resize_observer_observe_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new ResizeObserver(() => {});
        typeof observer.observe === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_resize_observer_unobserve_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new ResizeObserver(() => {});
        typeof observer.unobserve === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_resize_observer_disconnect_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new ResizeObserver(() => {});
        typeof observer.disconnect === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_resize_observer_with_callback() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let called = false;
        let observer = new ResizeObserver((entries) => {
            called = true;
        });
        observer !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Property Descriptor Tests
// ============================================================================

#[test]
fn test_intersection_observer_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'IntersectionObserver');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_mutation_observer_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'MutationObserver');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_resize_observer_property_descriptor() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let desc = Object.getOwnPropertyDescriptor(globalThis, 'ResizeObserver');
        desc !== undefined && typeof desc.value === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_all_observers_available() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof IntersectionObserver === 'function' &&
        typeof MutationObserver === 'function' &&
        typeof ResizeObserver === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_observer_callback_signature() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let io = new IntersectionObserver((entries, observer) => {});
        let mo = new MutationObserver((mutations, observer) => {});
        let ro = new ResizeObserver((entries, observer) => {});
        io !== null && mo !== null && ro !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// PerformanceObserver Tests
// ============================================================================

#[test]
fn test_performance_observer_constructor_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes("typeof PerformanceObserver")).unwrap();
    assert_eq!(result, JsValue::from(JsString::from("function")));
}

#[test]
fn test_performance_observer_constructor_basic() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new PerformanceObserver(() => {});
        observer !== null && observer !== undefined;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_requires_callback() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        try {
            new PerformanceObserver();
            false;
        } catch(e) {
            true;
        }
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_observe_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new PerformanceObserver(() => {});
        typeof observer.observe === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_disconnect_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new PerformanceObserver(() => {});
        typeof observer.disconnect === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_takerecords_method() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new PerformanceObserver(() => {});
        typeof observer.takeRecords === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_supported_entry_types() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        Array.isArray(PerformanceObserver.supportedEntryTypes);
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_supported_entry_types_contains_mark() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        PerformanceObserver.supportedEntryTypes.includes('mark');
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_supported_entry_types_contains_measure() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        PerformanceObserver.supportedEntryTypes.includes('measure');
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_observe_with_entry_types() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new PerformanceObserver(() => {});
        observer.observe({ entryTypes: ['mark', 'measure'] });
        true;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_observe_with_type() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new PerformanceObserver(() => {});
        observer.observe({ type: 'mark' });
        true;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_observe_requires_options() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        try {
            let observer = new PerformanceObserver(() => {});
            observer.observe();
            false;
        } catch(e) {
            true;
        }
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_observe_requires_entry_types_or_type() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        try {
            let observer = new PerformanceObserver(() => {});
            observer.observe({});
            false;
        } catch(e) {
            true;
        }
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_observe_cannot_use_both() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        try {
            let observer = new PerformanceObserver(() => {});
            observer.observe({ entryTypes: ['mark'], type: 'measure' });
            false;
        } catch(e) {
            true;
        }
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_disconnect() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new PerformanceObserver(() => {});
        observer.observe({ entryTypes: ['mark'] });
        observer.disconnect();
        true;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_performance_observer_take_records_returns_array() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        let observer = new PerformanceObserver(() => {});
        observer.observe({ entryTypes: ['mark'] });
        let records = observer.takeRecords();
        Array.isArray(records);
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_all_observers_including_performance() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        typeof IntersectionObserver === 'function' &&
        typeof MutationObserver === 'function' &&
        typeof ResizeObserver === 'function' &&
        typeof PerformanceObserver === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// IntersectionObserver Callback Tests (for Turnstile compatibility)
// ============================================================================

#[test]
fn test_intersection_observer_calls_callback_on_observe() {
    let mut context = create_test_context();
    // Test that IntersectionObserver calls the callback immediately when observe() is called
    // This is critical for Cloudflare Turnstile which checks visibility
    let result = context.eval(Source::from_bytes(r#"
        let callbackCalled = false;
        let entriesReceived = null;

        const observer = new IntersectionObserver((entries, obs) => {
            callbackCalled = true;
            entriesReceived = entries;
        });

        // Create a target element
        const target = document.createElement('div');
        target.id = 'turnstile-widget';

        // Observe should trigger the callback immediately
        observer.observe(target);

        // Check that callback was called
        callbackCalled === true && entriesReceived !== null;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intersection_observer_entry_has_required_properties() {
    let mut context = create_test_context();
    // Test that IntersectionObserverEntry has all required properties
    let result = context.eval(Source::from_bytes(r#"
        let entry = null;

        const observer = new IntersectionObserver((entries) => {
            if (entries.length > 0) {
                entry = entries[0];
            }
        });

        const target = document.createElement('div');
        observer.observe(target);

        // Check all required properties exist
        entry !== null &&
        typeof entry.target === 'object' &&
        typeof entry.intersectionRatio === 'number' &&
        typeof entry.isIntersecting === 'boolean' &&
        typeof entry.time === 'number' &&
        typeof entry.boundingClientRect === 'object' &&
        typeof entry.intersectionRect === 'object';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_intersection_observer_reports_visible_element() {
    let mut context = create_test_context();
    // Test that IntersectionObserver reports an element as visible
    // when it's in the viewport (default case for headless browser)
    let result = context.eval(Source::from_bytes(r#"
        let isIntersecting = null;
        let ratio = null;

        const observer = new IntersectionObserver((entries) => {
            if (entries.length > 0) {
                isIntersecting = entries[0].isIntersecting;
                ratio = entries[0].intersectionRatio;
            }
        });

        // Create a visible widget element
        const target = document.createElement('div');
        target.id = 'cf-turnstile';
        target.style.width = '300px';
        target.style.height = '65px';

        observer.observe(target);

        // The element should be reported as intersecting
        // (visible in viewport since we're at scroll position 0,0)
        isIntersecting === true;
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// ============================================================================
// Element.checkVisibility() Tests
// ============================================================================

#[test]
fn test_element_check_visibility_exists() {
    let mut context = create_test_context();
    let result = context.eval(Source::from_bytes(r#"
        const div = document.createElement('div');
        typeof div.checkVisibility === 'function';
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_check_visibility_returns_true_for_visible() {
    let mut context = create_test_context();
    // Default elements should be visible
    let result = context.eval(Source::from_bytes(r#"
        const div = document.createElement('div');
        div.checkVisibility();
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

#[test]
fn test_element_check_visibility_accepts_options() {
    let mut context = create_test_context();
    // Test that checkVisibility accepts options without throwing
    let result = context.eval(Source::from_bytes(r#"
        const div = document.createElement('div');
        // Should not throw with various option combinations
        div.checkVisibility({}) &&
        div.checkVisibility({ checkOpacity: true }) &&
        div.checkVisibility({ checkVisibilityCSS: true }) &&
        div.checkVisibility({ opacityProperty: true }) &&
        div.checkVisibility({ visibilityProperty: true });
    "#)).unwrap();
    assert_eq!(result.to_boolean(), true);
}

// Note: Tests for display:none, visibility:hidden, and opacity:0 require
// proper style persistence on Element, which is a separate feature.
// The current style getter creates a new CSSStyleDeclaration each time.
// For Turnstile compatibility, the important thing is that elements
// report as visible by default (which they do).
