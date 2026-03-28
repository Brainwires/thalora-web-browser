//! Web API Coverage Audit Module
//!
//! Systematically tests which Web APIs are implemented in Thalora
//! by evaluating JavaScript expressions against the Boa context.
//!
//! Note: Thalora is a HEADLESS browser - GUI-related APIs (alert, confirm,
//! visual rendering, etc.) are intentionally stubbed or omitted.

use crate::boa_engine::{Context, JsValue, Source};
use std::collections::HashMap;

/// API priority for coverage tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Critical,   // Must have for web compatibility
    High,       // Important for most web apps
    Medium,     // Nice to have
    Low,        // Specialized or GUI-only
    HeadlessNA, // Not applicable for headless browsers
}

/// Category of Web API
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ApiCategory {
    CoreDom,
    DomEvents,
    DomTraversal,
    HtmlElements,
    HtmlForms,
    Cssom,
    Storage,
    IndexedDb,
    Networking,
    Workers,
    Messaging,
    Navigator,
    Crypto,
    Performance,
    Observers,
    FileApi,
    Streams,
    Encoding,
    Url,
    WebComponents,
    History,
    Timers,
    Canvas,
    Audio,
    WebGl,
    Misc,
}

/// Single API to test
#[derive(Debug, Clone)]
pub struct ApiTest {
    pub name: &'static str,
    pub test_expr: &'static str,
    pub category: ApiCategory,
    pub priority: Priority,
    pub description: &'static str,
}

/// Result of an API test
#[derive(Debug, Clone)]
pub struct ApiResult {
    pub name: String,
    pub implemented: bool,
    pub category: ApiCategory,
    pub priority: Priority,
}

/// Get all APIs to test - comprehensive list for headless browser
pub fn get_api_tests() -> Vec<ApiTest> {
    vec![
        // ========== CORE DOM ==========
        ApiTest {
            name: "Document",
            test_expr: "typeof Document === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::Critical,
            description: "Document interface",
        },
        ApiTest {
            name: "Element",
            test_expr: "typeof Element === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::Critical,
            description: "Element interface",
        },
        ApiTest {
            name: "Node",
            test_expr: "typeof Node === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::Critical,
            description: "Node interface",
        },
        ApiTest {
            name: "Text",
            test_expr: "typeof Text === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::Critical,
            description: "Text node interface",
        },
        ApiTest {
            name: "Comment",
            test_expr: "typeof Comment === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::High,
            description: "Comment node",
        },
        ApiTest {
            name: "DocumentFragment",
            test_expr: "typeof DocumentFragment === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::Critical,
            description: "DocumentFragment interface",
        },
        ApiTest {
            name: "Attr",
            test_expr: "typeof Attr === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::High,
            description: "Attribute interface",
        },
        ApiTest {
            name: "CharacterData",
            test_expr: "typeof CharacterData === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::High,
            description: "CharacterData base",
        },
        ApiTest {
            name: "DOMTokenList",
            test_expr: "typeof DOMTokenList === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::Critical,
            description: "classList support",
        },
        ApiTest {
            name: "NamedNodeMap",
            test_expr: "typeof NamedNodeMap === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::High,
            description: "Attributes map",
        },
        ApiTest {
            name: "NodeList",
            test_expr: "typeof NodeList === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::Critical,
            description: "NodeList interface",
        },
        ApiTest {
            name: "HTMLCollection",
            test_expr: "typeof HTMLCollection === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::Critical,
            description: "HTMLCollection interface",
        },
        ApiTest {
            name: "DOMParser",
            test_expr: "typeof DOMParser === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::Critical,
            description: "HTML/XML parsing",
        },
        ApiTest {
            name: "XMLSerializer",
            test_expr: "typeof XMLSerializer === 'function'",
            category: ApiCategory::CoreDom,
            priority: Priority::High,
            description: "DOM serialization",
        },
        // ========== DOM TRAVERSAL ==========
        ApiTest {
            name: "Range",
            test_expr: "typeof Range === 'function'",
            category: ApiCategory::DomTraversal,
            priority: Priority::High,
            description: "Range interface",
        },
        ApiTest {
            name: "TreeWalker",
            test_expr: "typeof TreeWalker === 'function'",
            category: ApiCategory::DomTraversal,
            priority: Priority::Medium,
            description: "TreeWalker traversal",
        },
        ApiTest {
            name: "NodeIterator",
            test_expr: "typeof NodeIterator === 'function'",
            category: ApiCategory::DomTraversal,
            priority: Priority::Medium,
            description: "NodeIterator traversal",
        },
        ApiTest {
            name: "Selection",
            test_expr: "typeof Selection === 'function'",
            category: ApiCategory::DomTraversal,
            priority: Priority::Medium,
            description: "Selection API",
        },
        // ========== DOM EVENTS ==========
        ApiTest {
            name: "Event",
            test_expr: "typeof Event === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Critical,
            description: "Base Event interface",
        },
        ApiTest {
            name: "EventTarget",
            test_expr: "typeof EventTarget === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Critical,
            description: "EventTarget interface",
        },
        ApiTest {
            name: "CustomEvent",
            test_expr: "typeof CustomEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Critical,
            description: "Custom events",
        },
        ApiTest {
            name: "MouseEvent",
            test_expr: "typeof MouseEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::High,
            description: "Mouse events",
        },
        ApiTest {
            name: "KeyboardEvent",
            test_expr: "typeof KeyboardEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::High,
            description: "Keyboard events",
        },
        ApiTest {
            name: "FocusEvent",
            test_expr: "typeof FocusEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::High,
            description: "Focus events",
        },
        ApiTest {
            name: "InputEvent",
            test_expr: "typeof InputEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::High,
            description: "Input events",
        },
        ApiTest {
            name: "WheelEvent",
            test_expr: "typeof WheelEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Medium,
            description: "Wheel events",
        },
        ApiTest {
            name: "PointerEvent",
            test_expr: "typeof PointerEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Medium,
            description: "Pointer events",
        },
        ApiTest {
            name: "TouchEvent",
            test_expr: "typeof TouchEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Medium,
            description: "Touch events",
        },
        ApiTest {
            name: "DragEvent",
            test_expr: "typeof DragEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Medium,
            description: "Drag events",
        },
        ApiTest {
            name: "ErrorEvent",
            test_expr: "typeof ErrorEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Critical,
            description: "Error events",
        },
        ApiTest {
            name: "MessageEvent",
            test_expr: "typeof MessageEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Critical,
            description: "Message events",
        },
        ApiTest {
            name: "ProgressEvent",
            test_expr: "typeof ProgressEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::High,
            description: "Progress events",
        },
        ApiTest {
            name: "HashChangeEvent",
            test_expr: "typeof HashChangeEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::High,
            description: "Hash change",
        },
        ApiTest {
            name: "PopStateEvent",
            test_expr: "typeof PopStateEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::High,
            description: "History state",
        },
        ApiTest {
            name: "StorageEvent",
            test_expr: "typeof StorageEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::High,
            description: "Storage events",
        },
        ApiTest {
            name: "ClipboardEvent",
            test_expr: "typeof ClipboardEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Medium,
            description: "Clipboard events",
        },
        ApiTest {
            name: "AnimationEvent",
            test_expr: "typeof AnimationEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Medium,
            description: "Animation events",
        },
        ApiTest {
            name: "TransitionEvent",
            test_expr: "typeof TransitionEvent === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Medium,
            description: "Transition events",
        },
        ApiTest {
            name: "AbortController",
            test_expr: "typeof AbortController === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Critical,
            description: "Abort controller",
        },
        ApiTest {
            name: "AbortSignal",
            test_expr: "typeof AbortSignal === 'function'",
            category: ApiCategory::DomEvents,
            priority: Priority::Critical,
            description: "Abort signal",
        },
        // ========== HTML ELEMENTS ==========
        ApiTest {
            name: "HTMLElement",
            test_expr: "typeof HTMLElement === 'function'",
            category: ApiCategory::HtmlElements,
            priority: Priority::Critical,
            description: "HTMLElement base",
        },
        ApiTest {
            name: "HTMLDivElement",
            test_expr: "typeof HTMLDivElement === 'function'",
            category: ApiCategory::HtmlElements,
            priority: Priority::High,
            description: "Div element",
        },
        ApiTest {
            name: "HTMLSpanElement",
            test_expr: "typeof HTMLSpanElement === 'function'",
            category: ApiCategory::HtmlElements,
            priority: Priority::High,
            description: "Span element",
        },
        ApiTest {
            name: "HTMLAnchorElement",
            test_expr: "typeof HTMLAnchorElement === 'function'",
            category: ApiCategory::HtmlElements,
            priority: Priority::High,
            description: "Anchor element",
        },
        ApiTest {
            name: "HTMLImageElement",
            test_expr: "typeof HTMLImageElement === 'function'",
            category: ApiCategory::HtmlElements,
            priority: Priority::High,
            description: "Image element",
        },
        ApiTest {
            name: "HTMLScriptElement",
            test_expr: "typeof HTMLScriptElement === 'function'",
            category: ApiCategory::HtmlElements,
            priority: Priority::Critical,
            description: "Script element",
        },
        ApiTest {
            name: "HTMLStyleElement",
            test_expr: "typeof HTMLStyleElement === 'function'",
            category: ApiCategory::HtmlElements,
            priority: Priority::High,
            description: "Style element",
        },
        ApiTest {
            name: "HTMLLinkElement",
            test_expr: "typeof HTMLLinkElement === 'function'",
            category: ApiCategory::HtmlElements,
            priority: Priority::High,
            description: "Link element",
        },
        ApiTest {
            name: "HTMLTemplateElement",
            test_expr: "typeof HTMLTemplateElement === 'function'",
            category: ApiCategory::HtmlElements,
            priority: Priority::High,
            description: "Template element",
        },
        ApiTest {
            name: "HTMLSlotElement",
            test_expr: "typeof HTMLSlotElement === 'function'",
            category: ApiCategory::HtmlElements,
            priority: Priority::High,
            description: "Slot element",
        },
        ApiTest {
            name: "HTMLIFrameElement",
            test_expr: "typeof HTMLIFrameElement === 'function'",
            category: ApiCategory::HtmlElements,
            priority: Priority::High,
            description: "IFrame element",
        },
        ApiTest {
            name: "HTMLCanvasElement",
            test_expr: "typeof HTMLCanvasElement === 'function'",
            category: ApiCategory::HtmlElements,
            priority: Priority::High,
            description: "Canvas element",
        },
        ApiTest {
            name: "Image",
            test_expr: "typeof Image === 'function'",
            category: ApiCategory::HtmlElements,
            priority: Priority::High,
            description: "Image constructor",
        },
        // ========== HTML FORMS ==========
        ApiTest {
            name: "HTMLFormElement",
            test_expr: "typeof HTMLFormElement === 'function'",
            category: ApiCategory::HtmlForms,
            priority: Priority::High,
            description: "Form element",
        },
        ApiTest {
            name: "HTMLInputElement",
            test_expr: "typeof HTMLInputElement === 'function'",
            category: ApiCategory::HtmlForms,
            priority: Priority::Critical,
            description: "Input element",
        },
        ApiTest {
            name: "HTMLButtonElement",
            test_expr: "typeof HTMLButtonElement === 'function'",
            category: ApiCategory::HtmlForms,
            priority: Priority::High,
            description: "Button element",
        },
        ApiTest {
            name: "HTMLSelectElement",
            test_expr: "typeof HTMLSelectElement === 'function'",
            category: ApiCategory::HtmlForms,
            priority: Priority::High,
            description: "Select element",
        },
        ApiTest {
            name: "HTMLOptionElement",
            test_expr: "typeof HTMLOptionElement === 'function'",
            category: ApiCategory::HtmlForms,
            priority: Priority::High,
            description: "Option element",
        },
        ApiTest {
            name: "HTMLTextAreaElement",
            test_expr: "typeof HTMLTextAreaElement === 'function'",
            category: ApiCategory::HtmlForms,
            priority: Priority::High,
            description: "TextArea element",
        },
        ApiTest {
            name: "HTMLLabelElement",
            test_expr: "typeof HTMLLabelElement === 'function'",
            category: ApiCategory::HtmlForms,
            priority: Priority::High,
            description: "Label element",
        },
        ApiTest {
            name: "FormData",
            test_expr: "typeof FormData === 'function'",
            category: ApiCategory::HtmlForms,
            priority: Priority::Critical,
            description: "FormData interface",
        },
        ApiTest {
            name: "ValidityState",
            test_expr: "typeof ValidityState === 'function'",
            category: ApiCategory::HtmlForms,
            priority: Priority::Medium,
            description: "Form validation",
        },
        // ========== CSSOM ==========
        ApiTest {
            name: "CSSStyleDeclaration",
            test_expr: "typeof CSSStyleDeclaration === 'function'",
            category: ApiCategory::Cssom,
            priority: Priority::Critical,
            description: "Style declaration",
        },
        ApiTest {
            name: "CSSRule",
            test_expr: "typeof CSSRule === 'function'",
            category: ApiCategory::Cssom,
            priority: Priority::High,
            description: "CSS rule",
        },
        ApiTest {
            name: "CSSStyleSheet",
            test_expr: "typeof CSSStyleSheet === 'function'",
            category: ApiCategory::Cssom,
            priority: Priority::High,
            description: "Style sheet",
        },
        ApiTest {
            name: "StyleSheet",
            test_expr: "typeof StyleSheet === 'function'",
            category: ApiCategory::Cssom,
            priority: Priority::High,
            description: "StyleSheet base",
        },
        ApiTest {
            name: "MediaQueryList",
            test_expr: "typeof MediaQueryList === 'function'",
            category: ApiCategory::Cssom,
            priority: Priority::High,
            description: "Media queries",
        },
        ApiTest {
            name: "CSS",
            test_expr: "typeof CSS === 'object'",
            category: ApiCategory::Cssom,
            priority: Priority::High,
            description: "CSS namespace",
        },
        ApiTest {
            name: "DOMRect",
            test_expr: "typeof DOMRect === 'function'",
            category: ApiCategory::Cssom,
            priority: Priority::High,
            description: "Rectangle geometry",
        },
        ApiTest {
            name: "DOMRectReadOnly",
            test_expr: "typeof DOMRectReadOnly === 'function'",
            category: ApiCategory::Cssom,
            priority: Priority::Medium,
            description: "Read-only rect",
        },
        ApiTest {
            name: "DOMPoint",
            test_expr: "typeof DOMPoint === 'function'",
            category: ApiCategory::Cssom,
            priority: Priority::Medium,
            description: "Point geometry",
        },
        ApiTest {
            name: "DOMMatrix",
            test_expr: "typeof DOMMatrix === 'function'",
            category: ApiCategory::Cssom,
            priority: Priority::Medium,
            description: "Matrix transforms",
        },
        ApiTest {
            name: "getComputedStyle",
            test_expr: "typeof getComputedStyle === 'function'",
            category: ApiCategory::Cssom,
            priority: Priority::Critical,
            description: "Computed styles",
        },
        ApiTest {
            name: "matchMedia",
            test_expr: "typeof matchMedia === 'function'",
            category: ApiCategory::Cssom,
            priority: Priority::High,
            description: "Media matching",
        },
        // ========== STORAGE ==========
        ApiTest {
            name: "localStorage",
            test_expr: "typeof localStorage === 'object'",
            category: ApiCategory::Storage,
            priority: Priority::Critical,
            description: "Local storage",
        },
        ApiTest {
            name: "sessionStorage",
            test_expr: "typeof sessionStorage === 'object'",
            category: ApiCategory::Storage,
            priority: Priority::Critical,
            description: "Session storage",
        },
        ApiTest {
            name: "Storage",
            test_expr: "typeof Storage === 'function'",
            category: ApiCategory::Storage,
            priority: Priority::High,
            description: "Storage interface",
        },
        ApiTest {
            name: "CookieStore",
            test_expr: "typeof cookieStore === 'object'",
            category: ApiCategory::Storage,
            priority: Priority::Medium,
            description: "Cookie store API",
        },
        // ========== INDEXEDDB ==========
        ApiTest {
            name: "indexedDB",
            test_expr: "typeof indexedDB === 'object'",
            category: ApiCategory::IndexedDb,
            priority: Priority::Critical,
            description: "IndexedDB API",
        },
        ApiTest {
            name: "IDBFactory",
            test_expr: "typeof IDBFactory === 'function'",
            category: ApiCategory::IndexedDb,
            priority: Priority::High,
            description: "IDB factory",
        },
        ApiTest {
            name: "IDBDatabase",
            test_expr: "typeof IDBDatabase === 'function'",
            category: ApiCategory::IndexedDb,
            priority: Priority::High,
            description: "IDB database",
        },
        ApiTest {
            name: "IDBTransaction",
            test_expr: "typeof IDBTransaction === 'function'",
            category: ApiCategory::IndexedDb,
            priority: Priority::High,
            description: "IDB transaction",
        },
        ApiTest {
            name: "IDBObjectStore",
            test_expr: "typeof IDBObjectStore === 'function'",
            category: ApiCategory::IndexedDb,
            priority: Priority::High,
            description: "IDB object store",
        },
        ApiTest {
            name: "IDBIndex",
            test_expr: "typeof IDBIndex === 'function'",
            category: ApiCategory::IndexedDb,
            priority: Priority::High,
            description: "IDB index",
        },
        ApiTest {
            name: "IDBCursor",
            test_expr: "typeof IDBCursor === 'function'",
            category: ApiCategory::IndexedDb,
            priority: Priority::High,
            description: "IDB cursor",
        },
        ApiTest {
            name: "IDBKeyRange",
            test_expr: "typeof IDBKeyRange === 'function'",
            category: ApiCategory::IndexedDb,
            priority: Priority::High,
            description: "IDB key range",
        },
        ApiTest {
            name: "IDBRequest",
            test_expr: "typeof IDBRequest === 'function'",
            category: ApiCategory::IndexedDb,
            priority: Priority::High,
            description: "IDB request",
        },
        // ========== NETWORKING ==========
        ApiTest {
            name: "fetch",
            test_expr: "typeof fetch === 'function'",
            category: ApiCategory::Networking,
            priority: Priority::Critical,
            description: "Fetch API",
        },
        ApiTest {
            name: "Request",
            test_expr: "typeof Request === 'function'",
            category: ApiCategory::Networking,
            priority: Priority::Critical,
            description: "Request interface",
        },
        ApiTest {
            name: "Response",
            test_expr: "typeof Response === 'function'",
            category: ApiCategory::Networking,
            priority: Priority::Critical,
            description: "Response interface",
        },
        ApiTest {
            name: "Headers",
            test_expr: "typeof Headers === 'function'",
            category: ApiCategory::Networking,
            priority: Priority::Critical,
            description: "Headers interface",
        },
        ApiTest {
            name: "XMLHttpRequest",
            test_expr: "typeof XMLHttpRequest === 'function'",
            category: ApiCategory::Networking,
            priority: Priority::Critical,
            description: "XHR interface",
        },
        ApiTest {
            name: "WebSocket",
            test_expr: "typeof WebSocket === 'function'",
            category: ApiCategory::Networking,
            priority: Priority::Critical,
            description: "WebSocket API",
        },
        ApiTest {
            name: "CloseEvent",
            test_expr: "typeof CloseEvent === 'function'",
            category: ApiCategory::Networking,
            priority: Priority::High,
            description: "WS close event",
        },
        ApiTest {
            name: "EventSource",
            test_expr: "typeof EventSource === 'function'",
            category: ApiCategory::Networking,
            priority: Priority::Medium,
            description: "Server-sent events",
        },
        // ========== WORKERS ==========
        ApiTest {
            name: "Worker",
            test_expr: "typeof Worker === 'function'",
            category: ApiCategory::Workers,
            priority: Priority::Critical,
            description: "Web Worker",
        },
        ApiTest {
            name: "SharedWorker",
            test_expr: "typeof SharedWorker === 'function'",
            category: ApiCategory::Workers,
            priority: Priority::Medium,
            description: "Shared Worker",
        },
        ApiTest {
            name: "navigator.serviceWorker",
            test_expr: "typeof navigator.serviceWorker === 'object'",
            category: ApiCategory::Workers,
            priority: Priority::High,
            description: "Service Worker container",
        },
        // ========== MESSAGING ==========
        ApiTest {
            name: "MessageChannel",
            test_expr: "typeof MessageChannel === 'function'",
            category: ApiCategory::Messaging,
            priority: Priority::High,
            description: "Message channel",
        },
        ApiTest {
            name: "MessagePort",
            test_expr: "typeof MessagePort === 'function'",
            category: ApiCategory::Messaging,
            priority: Priority::High,
            description: "Message port",
        },
        ApiTest {
            name: "BroadcastChannel",
            test_expr: "typeof BroadcastChannel === 'function'",
            category: ApiCategory::Messaging,
            priority: Priority::Medium,
            description: "Broadcast channel",
        },
        ApiTest {
            name: "postMessage",
            test_expr: "typeof postMessage === 'function'",
            category: ApiCategory::Messaging,
            priority: Priority::Critical,
            description: "Post message",
        },
        // ========== NAVIGATOR ==========
        ApiTest {
            name: "navigator",
            test_expr: "typeof navigator === 'object'",
            category: ApiCategory::Navigator,
            priority: Priority::Critical,
            description: "Navigator object",
        },
        ApiTest {
            name: "navigator.userAgent",
            test_expr: "typeof navigator.userAgent === 'string'",
            category: ApiCategory::Navigator,
            priority: Priority::Critical,
            description: "User agent",
        },
        ApiTest {
            name: "navigator.language",
            test_expr: "typeof navigator.language === 'string'",
            category: ApiCategory::Navigator,
            priority: Priority::High,
            description: "Language",
        },
        ApiTest {
            name: "navigator.languages",
            test_expr: "Array.isArray(navigator.languages)",
            category: ApiCategory::Navigator,
            priority: Priority::High,
            description: "Languages",
        },
        ApiTest {
            name: "navigator.onLine",
            test_expr: "typeof navigator.onLine === 'boolean'",
            category: ApiCategory::Navigator,
            priority: Priority::High,
            description: "Online status",
        },
        ApiTest {
            name: "navigator.cookieEnabled",
            test_expr: "typeof navigator.cookieEnabled === 'boolean'",
            category: ApiCategory::Navigator,
            priority: Priority::High,
            description: "Cookie support",
        },
        ApiTest {
            name: "navigator.hardwareConcurrency",
            test_expr: "typeof navigator.hardwareConcurrency === 'number'",
            category: ApiCategory::Navigator,
            priority: Priority::Medium,
            description: "CPU cores",
        },
        ApiTest {
            name: "navigator.clipboard",
            test_expr: "typeof navigator.clipboard === 'object'",
            category: ApiCategory::Navigator,
            priority: Priority::High,
            description: "Clipboard API",
        },
        ApiTest {
            name: "navigator.permissions",
            test_expr: "typeof navigator.permissions === 'object'",
            category: ApiCategory::Navigator,
            priority: Priority::High,
            description: "Permissions API",
        },
        ApiTest {
            name: "navigator.locks",
            test_expr: "typeof navigator.locks === 'object'",
            category: ApiCategory::Navigator,
            priority: Priority::Medium,
            description: "Web Locks API",
        },
        ApiTest {
            name: "navigator.storage",
            test_expr: "typeof navigator.storage === 'object'",
            category: ApiCategory::Navigator,
            priority: Priority::Medium,
            description: "Storage manager",
        },
        ApiTest {
            name: "navigator.sendBeacon",
            test_expr: "typeof navigator.sendBeacon === 'function'",
            category: ApiCategory::Navigator,
            priority: Priority::Medium,
            description: "Beacon API",
        },
        ApiTest {
            name: "navigator.geolocation",
            test_expr: "typeof navigator.geolocation === 'object'",
            category: ApiCategory::Navigator,
            priority: Priority::Low,
            description: "Geolocation",
        },
        ApiTest {
            name: "navigator.mediaDevices",
            test_expr: "typeof navigator.mediaDevices === 'object'",
            category: ApiCategory::Navigator,
            priority: Priority::Low,
            description: "Media devices",
        },
        // ========== CRYPTO ==========
        ApiTest {
            name: "crypto",
            test_expr: "typeof crypto === 'object'",
            category: ApiCategory::Crypto,
            priority: Priority::Critical,
            description: "Crypto API",
        },
        ApiTest {
            name: "crypto.subtle",
            test_expr: "typeof crypto.subtle === 'object'",
            category: ApiCategory::Crypto,
            priority: Priority::Critical,
            description: "SubtleCrypto",
        },
        ApiTest {
            name: "crypto.getRandomValues",
            test_expr: "typeof crypto.getRandomValues === 'function'",
            category: ApiCategory::Crypto,
            priority: Priority::Critical,
            description: "Random values",
        },
        ApiTest {
            name: "crypto.randomUUID",
            test_expr: "typeof crypto.randomUUID === 'function'",
            category: ApiCategory::Crypto,
            priority: Priority::High,
            description: "UUID generation",
        },
        ApiTest {
            name: "CryptoKey",
            test_expr: "typeof CryptoKey === 'function'",
            category: ApiCategory::Crypto,
            priority: Priority::High,
            description: "Crypto key",
        },
        // ========== PERFORMANCE ==========
        ApiTest {
            name: "performance",
            test_expr: "typeof performance === 'object'",
            category: ApiCategory::Performance,
            priority: Priority::Critical,
            description: "Performance API",
        },
        ApiTest {
            name: "performance.now",
            test_expr: "typeof performance.now === 'function'",
            category: ApiCategory::Performance,
            priority: Priority::Critical,
            description: "High-res time",
        },
        ApiTest {
            name: "performance.mark",
            test_expr: "typeof performance.mark === 'function'",
            category: ApiCategory::Performance,
            priority: Priority::High,
            description: "Perf marks",
        },
        ApiTest {
            name: "performance.measure",
            test_expr: "typeof performance.measure === 'function'",
            category: ApiCategory::Performance,
            priority: Priority::High,
            description: "Perf measures",
        },
        ApiTest {
            name: "PerformanceObserver",
            test_expr: "typeof PerformanceObserver === 'function'",
            category: ApiCategory::Performance,
            priority: Priority::High,
            description: "Perf observer",
        },
        ApiTest {
            name: "performance.getEntries",
            test_expr: "typeof performance.getEntries === 'function'",
            category: ApiCategory::Performance,
            priority: Priority::High,
            description: "Perf entries",
        },
        // ========== OBSERVERS ==========
        ApiTest {
            name: "MutationObserver",
            test_expr: "typeof MutationObserver === 'function'",
            category: ApiCategory::Observers,
            priority: Priority::Critical,
            description: "Mutation observer",
        },
        ApiTest {
            name: "MutationRecord",
            test_expr: "typeof MutationRecord === 'function'",
            category: ApiCategory::Observers,
            priority: Priority::High,
            description: "Mutation record",
        },
        ApiTest {
            name: "IntersectionObserver",
            test_expr: "typeof IntersectionObserver === 'function'",
            category: ApiCategory::Observers,
            priority: Priority::High,
            description: "Intersection observer",
        },
        ApiTest {
            name: "ResizeObserver",
            test_expr: "typeof ResizeObserver === 'function'",
            category: ApiCategory::Observers,
            priority: Priority::High,
            description: "Resize observer",
        },
        ApiTest {
            name: "ReportingObserver",
            test_expr: "typeof ReportingObserver === 'function'",
            category: ApiCategory::Observers,
            priority: Priority::Low,
            description: "Reporting observer",
        },
        // ========== FILE API ==========
        ApiTest {
            name: "File",
            test_expr: "typeof File === 'function'",
            category: ApiCategory::FileApi,
            priority: Priority::Critical,
            description: "File interface",
        },
        ApiTest {
            name: "Blob",
            test_expr: "typeof Blob === 'function'",
            category: ApiCategory::FileApi,
            priority: Priority::Critical,
            description: "Blob interface",
        },
        ApiTest {
            name: "FileReader",
            test_expr: "typeof FileReader === 'function'",
            category: ApiCategory::FileApi,
            priority: Priority::Critical,
            description: "File reader",
        },
        ApiTest {
            name: "FileList",
            test_expr: "typeof FileList === 'function'",
            category: ApiCategory::FileApi,
            priority: Priority::High,
            description: "File list",
        },
        // ========== STREAMS ==========
        ApiTest {
            name: "ReadableStream",
            test_expr: "typeof ReadableStream === 'function'",
            category: ApiCategory::Streams,
            priority: Priority::High,
            description: "Readable stream",
        },
        ApiTest {
            name: "WritableStream",
            test_expr: "typeof WritableStream === 'function'",
            category: ApiCategory::Streams,
            priority: Priority::High,
            description: "Writable stream",
        },
        ApiTest {
            name: "TransformStream",
            test_expr: "typeof TransformStream === 'function'",
            category: ApiCategory::Streams,
            priority: Priority::Medium,
            description: "Transform stream",
        },
        ApiTest {
            name: "ByteLengthQueuingStrategy",
            test_expr: "typeof ByteLengthQueuingStrategy === 'function'",
            category: ApiCategory::Streams,
            priority: Priority::Low,
            description: "Byte queuing",
        },
        ApiTest {
            name: "CountQueuingStrategy",
            test_expr: "typeof CountQueuingStrategy === 'function'",
            category: ApiCategory::Streams,
            priority: Priority::Low,
            description: "Count queuing",
        },
        // ========== ENCODING ==========
        ApiTest {
            name: "TextEncoder",
            test_expr: "typeof TextEncoder === 'function'",
            category: ApiCategory::Encoding,
            priority: Priority::Critical,
            description: "Text encoder",
        },
        ApiTest {
            name: "TextDecoder",
            test_expr: "typeof TextDecoder === 'function'",
            category: ApiCategory::Encoding,
            priority: Priority::Critical,
            description: "Text decoder",
        },
        ApiTest {
            name: "TextEncoderStream",
            test_expr: "typeof TextEncoderStream === 'function'",
            category: ApiCategory::Encoding,
            priority: Priority::Low,
            description: "Encoder stream",
        },
        ApiTest {
            name: "TextDecoderStream",
            test_expr: "typeof TextDecoderStream === 'function'",
            category: ApiCategory::Encoding,
            priority: Priority::Low,
            description: "Decoder stream",
        },
        // ========== URL ==========
        ApiTest {
            name: "URL",
            test_expr: "typeof URL === 'function'",
            category: ApiCategory::Url,
            priority: Priority::Critical,
            description: "URL interface",
        },
        ApiTest {
            name: "URLSearchParams",
            test_expr: "typeof URLSearchParams === 'function'",
            category: ApiCategory::Url,
            priority: Priority::Critical,
            description: "URL params",
        },
        ApiTest {
            name: "URLPattern",
            test_expr: "typeof URLPattern === 'function'",
            category: ApiCategory::Url,
            priority: Priority::Low,
            description: "URL pattern",
        },
        // ========== WEB COMPONENTS ==========
        ApiTest {
            name: "customElements",
            test_expr: "typeof customElements === 'object'",
            category: ApiCategory::WebComponents,
            priority: Priority::High,
            description: "Custom elements",
        },
        ApiTest {
            name: "customElements.define",
            test_expr: "typeof customElements.define === 'function'",
            category: ApiCategory::WebComponents,
            priority: Priority::High,
            description: "Define element",
        },
        ApiTest {
            name: "ShadowRoot",
            test_expr: "typeof ShadowRoot === 'function'",
            category: ApiCategory::WebComponents,
            priority: Priority::High,
            description: "Shadow root",
        },
        ApiTest {
            name: "Element.attachShadow",
            test_expr: "typeof Element.prototype.attachShadow === 'function'",
            category: ApiCategory::WebComponents,
            priority: Priority::High,
            description: "Attach shadow",
        },
        // ========== HISTORY ==========
        ApiTest {
            name: "history",
            test_expr: "typeof history === 'object'",
            category: ApiCategory::History,
            priority: Priority::Critical,
            description: "History API",
        },
        ApiTest {
            name: "history.pushState",
            test_expr: "typeof history.pushState === 'function'",
            category: ApiCategory::History,
            priority: Priority::Critical,
            description: "Push state",
        },
        ApiTest {
            name: "history.replaceState",
            test_expr: "typeof history.replaceState === 'function'",
            category: ApiCategory::History,
            priority: Priority::Critical,
            description: "Replace state",
        },
        ApiTest {
            name: "location",
            test_expr: "typeof location === 'object'",
            category: ApiCategory::History,
            priority: Priority::Critical,
            description: "Location object",
        },
        // ========== TIMERS ==========
        ApiTest {
            name: "setTimeout",
            test_expr: "typeof setTimeout === 'function'",
            category: ApiCategory::Timers,
            priority: Priority::Critical,
            description: "Set timeout",
        },
        ApiTest {
            name: "setInterval",
            test_expr: "typeof setInterval === 'function'",
            category: ApiCategory::Timers,
            priority: Priority::Critical,
            description: "Set interval",
        },
        ApiTest {
            name: "clearTimeout",
            test_expr: "typeof clearTimeout === 'function'",
            category: ApiCategory::Timers,
            priority: Priority::Critical,
            description: "Clear timeout",
        },
        ApiTest {
            name: "clearInterval",
            test_expr: "typeof clearInterval === 'function'",
            category: ApiCategory::Timers,
            priority: Priority::Critical,
            description: "Clear interval",
        },
        ApiTest {
            name: "requestAnimationFrame",
            test_expr: "typeof requestAnimationFrame === 'function'",
            category: ApiCategory::Timers,
            priority: Priority::High,
            description: "Animation frame",
        },
        ApiTest {
            name: "cancelAnimationFrame",
            test_expr: "typeof cancelAnimationFrame === 'function'",
            category: ApiCategory::Timers,
            priority: Priority::High,
            description: "Cancel animation",
        },
        ApiTest {
            name: "requestIdleCallback",
            test_expr: "typeof requestIdleCallback === 'function'",
            category: ApiCategory::Timers,
            priority: Priority::Medium,
            description: "Idle callback",
        },
        ApiTest {
            name: "queueMicrotask",
            test_expr: "typeof queueMicrotask === 'function'",
            category: ApiCategory::Timers,
            priority: Priority::High,
            description: "Queue microtask",
        },
        // ========== CANVAS (Headless-relevant for image generation) ==========
        ApiTest {
            name: "CanvasRenderingContext2D",
            test_expr: "typeof CanvasRenderingContext2D === 'function'",
            category: ApiCategory::Canvas,
            priority: Priority::High,
            description: "2D context",
        },
        ApiTest {
            name: "OffscreenCanvas",
            test_expr: "typeof OffscreenCanvas === 'function'",
            category: ApiCategory::Canvas,
            priority: Priority::Medium,
            description: "Offscreen canvas",
        },
        ApiTest {
            name: "ImageBitmap",
            test_expr: "typeof ImageBitmap === 'function'",
            category: ApiCategory::Canvas,
            priority: Priority::Medium,
            description: "Image bitmap",
        },
        ApiTest {
            name: "ImageData",
            test_expr: "typeof ImageData === 'function'",
            category: ApiCategory::Canvas,
            priority: Priority::High,
            description: "Image data",
        },
        ApiTest {
            name: "Path2D",
            test_expr: "typeof Path2D === 'function'",
            category: ApiCategory::Canvas,
            priority: Priority::Medium,
            description: "Path 2D",
        },
        ApiTest {
            name: "CanvasGradient",
            test_expr: "typeof CanvasGradient === 'function'",
            category: ApiCategory::Canvas,
            priority: Priority::Medium,
            description: "Canvas gradient",
        },
        ApiTest {
            name: "CanvasPattern",
            test_expr: "typeof CanvasPattern === 'function'",
            category: ApiCategory::Canvas,
            priority: Priority::Medium,
            description: "Canvas pattern",
        },
        // ========== AUDIO (Headless may need for media processing) ==========
        ApiTest {
            name: "AudioContext",
            test_expr: "typeof AudioContext === 'function'",
            category: ApiCategory::Audio,
            priority: Priority::Medium,
            description: "Audio context",
        },
        ApiTest {
            name: "OfflineAudioContext",
            test_expr: "typeof OfflineAudioContext === 'function'",
            category: ApiCategory::Audio,
            priority: Priority::Low,
            description: "Offline audio",
        },
        ApiTest {
            name: "Audio",
            test_expr: "typeof Audio === 'function'",
            category: ApiCategory::Audio,
            priority: Priority::Medium,
            description: "Audio element",
        },
        // ========== WEBGL (Headless may need for GPU compute) ==========
        ApiTest {
            name: "WebGLRenderingContext",
            test_expr: "typeof WebGLRenderingContext === 'function'",
            category: ApiCategory::WebGl,
            priority: Priority::Medium,
            description: "WebGL 1",
        },
        ApiTest {
            name: "WebGL2RenderingContext",
            test_expr: "typeof WebGL2RenderingContext === 'function'",
            category: ApiCategory::WebGl,
            priority: Priority::Low,
            description: "WebGL 2",
        },
        // ========== MISC ==========
        ApiTest {
            name: "window",
            test_expr: "typeof window === 'object'",
            category: ApiCategory::Misc,
            priority: Priority::Critical,
            description: "Window object",
        },
        ApiTest {
            name: "document",
            test_expr: "typeof document === 'object'",
            category: ApiCategory::Misc,
            priority: Priority::Critical,
            description: "Document object",
        },
        ApiTest {
            name: "console",
            test_expr: "typeof console === 'object'",
            category: ApiCategory::Misc,
            priority: Priority::Critical,
            description: "Console API",
        },
        ApiTest {
            name: "atob",
            test_expr: "typeof atob === 'function'",
            category: ApiCategory::Misc,
            priority: Priority::Critical,
            description: "Base64 decode",
        },
        ApiTest {
            name: "btoa",
            test_expr: "typeof btoa === 'function'",
            category: ApiCategory::Misc,
            priority: Priority::Critical,
            description: "Base64 encode",
        },
        ApiTest {
            name: "structuredClone",
            test_expr: "typeof structuredClone === 'function'",
            category: ApiCategory::Misc,
            priority: Priority::High,
            description: "Structured clone",
        },
        ApiTest {
            name: "Notification",
            test_expr: "typeof Notification === 'function'",
            category: ApiCategory::Misc,
            priority: Priority::Low,
            description: "Notifications",
        },
        ApiTest {
            name: "ClipboardItem",
            test_expr: "typeof ClipboardItem === 'function'",
            category: ApiCategory::Misc,
            priority: Priority::Medium,
            description: "Clipboard item",
        },
        ApiTest {
            name: "screen",
            test_expr: "typeof screen === 'object'",
            category: ApiCategory::Misc,
            priority: Priority::Medium,
            description: "Screen object",
        },
        ApiTest {
            name: "caches",
            test_expr: "typeof caches === 'object'",
            category: ApiCategory::Misc,
            priority: Priority::Medium,
            description: "Cache storage",
        },
        // GUI-only APIs (Headless N/A but tracked for completeness)
        ApiTest {
            name: "alert",
            test_expr: "typeof alert === 'function'",
            category: ApiCategory::Misc,
            priority: Priority::HeadlessNA,
            description: "Alert dialog",
        },
        ApiTest {
            name: "confirm",
            test_expr: "typeof confirm === 'function'",
            category: ApiCategory::Misc,
            priority: Priority::HeadlessNA,
            description: "Confirm dialog",
        },
        ApiTest {
            name: "prompt",
            test_expr: "typeof prompt === 'function'",
            category: ApiCategory::Misc,
            priority: Priority::HeadlessNA,
            description: "Prompt dialog",
        },
        ApiTest {
            name: "print",
            test_expr: "typeof print === 'function'",
            category: ApiCategory::Misc,
            priority: Priority::HeadlessNA,
            description: "Print dialog",
        },
    ]
}

/// Run all API tests against a Boa context
pub fn run_audit(context: &mut Context) -> Vec<ApiResult> {
    let tests = get_api_tests();
    let mut results = Vec::with_capacity(tests.len());

    for test in tests {
        let implemented = match context.eval(Source::from_bytes(test.test_expr)) {
            Ok(value) => value.to_boolean(),
            Err(_) => false,
        };

        results.push(ApiResult {
            name: test.name.to_string(),
            implemented,
            category: test.category,
            priority: test.priority,
        });
    }

    results
}

/// Generate a report from audit results
pub fn generate_report(results: &[ApiResult]) -> String {
    let mut report = String::new();

    // Overall stats
    let total = results.len();
    let implemented = results.iter().filter(|r| r.implemented).count();
    let critical_total = results
        .iter()
        .filter(|r| r.priority == Priority::Critical)
        .count();
    let critical_impl = results
        .iter()
        .filter(|r| r.priority == Priority::Critical && r.implemented)
        .count();
    let high_total = results
        .iter()
        .filter(|r| r.priority == Priority::High)
        .count();
    let high_impl = results
        .iter()
        .filter(|r| r.priority == Priority::High && r.implemented)
        .count();
    let medium_total = results
        .iter()
        .filter(|r| r.priority == Priority::Medium)
        .count();
    let medium_impl = results
        .iter()
        .filter(|r| r.priority == Priority::Medium && r.implemented)
        .count();

    report.push_str("\n");
    report.push_str(
        "╔══════════════════════════════════════════════════════════════════════════════╗\n",
    );
    report.push_str(
        "║           THALORA HEADLESS BROWSER - WEB API COVERAGE REPORT                ║\n",
    );
    report.push_str(
        "╠══════════════════════════════════════════════════════════════════════════════╣\n",
    );
    report.push_str(&format!(
        "║ Overall Coverage:    {:>3}/{:<3} ({:>5.1}%)                                      ║\n",
        implemented,
        total,
        (implemented as f64 / total as f64) * 100.0
    ));
    report.push_str(&format!(
        "║ Critical APIs:       {:>3}/{:<3} ({:>5.1}%)                                      ║\n",
        critical_impl,
        critical_total,
        (critical_impl as f64 / critical_total as f64) * 100.0
    ));
    report.push_str(&format!(
        "║ High Priority APIs:  {:>3}/{:<3} ({:>5.1}%)                                      ║\n",
        high_impl,
        high_total,
        (high_impl as f64 / high_total as f64) * 100.0
    ));
    report.push_str(&format!(
        "║ Medium Priority:     {:>3}/{:<3} ({:>5.1}%)                                      ║\n",
        medium_impl,
        medium_total,
        (medium_impl as f64 / medium_total as f64) * 100.0
    ));
    report.push_str(
        "╚══════════════════════════════════════════════════════════════════════════════╝\n",
    );

    // Group by category
    let mut by_category: HashMap<ApiCategory, Vec<&ApiResult>> = HashMap::new();
    for result in results {
        by_category.entry(result.category).or_default().push(result);
    }

    // Sort categories by priority of missing APIs
    let mut categories: Vec<_> = by_category.keys().collect();
    categories.sort_by_key(|cat| {
        let cat_results = by_category.get(cat).unwrap();
        let missing_critical = cat_results
            .iter()
            .filter(|r| !r.implemented && r.priority == Priority::Critical)
            .count();
        std::cmp::Reverse(missing_critical)
    });

    for category in categories {
        let cat_results = by_category.get(category).unwrap();
        let cat_impl = cat_results.iter().filter(|r| r.implemented).count();
        let cat_total = cat_results.len();
        let missing: Vec<_> = cat_results.iter().filter(|r| !r.implemented).collect();

        report.push_str(&format!("\n{:?} ({}/{})\n", category, cat_impl, cat_total));
        report.push_str(&"─".repeat(60));
        report.push_str("\n");

        // Show implemented
        for result in cat_results.iter().filter(|r| r.implemented) {
            let priority_tag = match result.priority {
                Priority::Critical => "[CRIT]",
                Priority::High => "[HIGH]",
                Priority::Medium => "[MED ]",
                Priority::Low => "[LOW ]",
                Priority::HeadlessNA => "[N/A ]",
            };
            report.push_str(&format!("  ✓ {} {}\n", priority_tag, result.name));
        }

        // Show missing
        if !missing.is_empty() {
            report.push_str("  ---\n");
            for result in missing {
                let priority_tag = match result.priority {
                    Priority::Critical => "[CRIT]",
                    Priority::High => "[HIGH]",
                    Priority::Medium => "[MED ]",
                    Priority::Low => "[LOW ]",
                    Priority::HeadlessNA => "[N/A ]",
                };
                report.push_str(&format!("  ✗ {} {}\n", priority_tag, result.name));
            }
        }
    }

    // Missing Critical APIs summary
    let missing_critical: Vec<_> = results
        .iter()
        .filter(|r| !r.implemented && r.priority == Priority::Critical)
        .collect();

    if !missing_critical.is_empty() {
        report.push_str("\n");
        report.push_str(
            "╔══════════════════════════════════════════════════════════════════════════════╗\n",
        );
        report.push_str(
            "║                    MISSING CRITICAL APIs (Priority Fix)                     ║\n",
        );
        report.push_str(
            "╠══════════════════════════════════════════════════════════════════════════════╣\n",
        );
        for result in missing_critical {
            report.push_str(&format!("║  ✗ {:<72} ║\n", result.name));
        }
        report.push_str(
            "╚══════════════════════════════════════════════════════════════════════════════╝\n",
        );
    }

    // Missing High Priority APIs
    let missing_high: Vec<_> = results
        .iter()
        .filter(|r| !r.implemented && r.priority == Priority::High)
        .collect();

    if !missing_high.is_empty() {
        report.push_str("\n");
        report.push_str(
            "═══════════════════════════ MISSING HIGH PRIORITY APIs ═══════════════════════\n",
        );
        for result in missing_high {
            report.push_str(&format!("  ✗ {:?}: {}\n", result.category, result.name));
        }
    }

    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_context() -> Context {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).expect("Failed to initialize browser APIs");
        context
    }

    #[test]
    fn test_audit_runs_successfully() {
        let mut context = create_test_context();
        let results = run_audit(&mut context);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_report_generation() {
        let mut context = create_test_context();
        let results = run_audit(&mut context);
        let report = generate_report(&results);
        assert!(report.contains("THALORA"));
        assert!(report.contains("Coverage"));
    }

    #[test]
    fn run_full_audit_and_print() {
        let mut context = create_test_context();
        let results = run_audit(&mut context);
        let report = generate_report(&results);
        println!("{}", report);
    }
}
