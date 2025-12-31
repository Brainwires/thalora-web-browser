//! Web API Coverage Audit Tool
//!
//! This tool systematically tests which Web APIs are implemented in Thalora
//! by comparing against the standard Web API specifications.

use std::collections::{HashMap, HashSet};

/// Categories of Web APIs for organized reporting
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ApiCategory {
    // Core DOM
    Dom,
    DomEvents,
    DomTraversal,
    DomMutation,

    // HTML Elements
    HtmlElements,
    HtmlForms,
    HtmlMedia,
    HtmlCanvas,

    // CSS/Styling
    Cssom,
    CssTypedOm,

    // Storage
    WebStorage,
    IndexedDb,
    CacheApi,

    // Networking
    Fetch,
    Xhr,
    WebSockets,

    // Workers
    WebWorkers,
    ServiceWorkers,
    SharedWorkers,

    // Messaging
    ChannelMessaging,
    BroadcastChannel,

    // Device/Browser
    Navigator,
    Geolocation,
    DeviceOrientation,
    Battery,
    Vibration,

    // Media
    WebAudio,
    MediaDevices,
    WebRtc,

    // Graphics
    Canvas2d,
    WebGl,
    WebGl2,

    // Security
    Crypto,
    Permissions,
    CredentialManagement,

    // Performance
    PerformanceApi,
    IntersectionObserver,
    ResizeObserver,
    MutationObserver,

    // Misc
    Clipboard,
    Notifications,
    FileApi,
    Streams,
    Encoding,
    Url,
    WebComponents,
    History,

    // Not categorized
    Other,
}

/// Represents a Web API to test
#[derive(Debug, Clone)]
pub struct WebApi {
    pub name: String,
    pub category: ApiCategory,
    pub test_expression: String,
    pub description: String,
    pub priority: Priority,
    pub spec_url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Priority {
    Critical,   // Must have for basic web compatibility
    High,       // Important for most web apps
    Medium,     // Nice to have, used by many sites
    Low,        // Specialized, less common
}

/// Result of testing an API
#[derive(Debug, Clone)]
pub struct ApiTestResult {
    pub api: WebApi,
    pub implemented: bool,
    pub partial: bool,
    pub notes: Option<String>,
}

/// Generate the comprehensive list of Web APIs to test
pub fn get_web_apis() -> Vec<WebApi> {
    let mut apis = Vec::new();

    // =========================================================================
    // CORE DOM APIs
    // =========================================================================

    // Global constructors
    apis.push(WebApi {
        name: "Document".into(),
        category: ApiCategory::Dom,
        test_expression: "typeof Document === 'function'".into(),
        description: "Document interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-document".into()),
    });

    apis.push(WebApi {
        name: "Element".into(),
        category: ApiCategory::Dom,
        test_expression: "typeof Element === 'function'".into(),
        description: "Element interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-element".into()),
    });

    apis.push(WebApi {
        name: "Node".into(),
        category: ApiCategory::Dom,
        test_expression: "typeof Node === 'function'".into(),
        description: "Node interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-node".into()),
    });

    apis.push(WebApi {
        name: "Text".into(),
        category: ApiCategory::Dom,
        test_expression: "typeof Text === 'function'".into(),
        description: "Text node interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-text".into()),
    });

    apis.push(WebApi {
        name: "Comment".into(),
        category: ApiCategory::Dom,
        test_expression: "typeof Comment === 'function'".into(),
        description: "Comment node interface".into(),
        priority: Priority::High,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-comment".into()),
    });

    apis.push(WebApi {
        name: "DocumentFragment".into(),
        category: ApiCategory::Dom,
        test_expression: "typeof DocumentFragment === 'function'".into(),
        description: "DocumentFragment interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-documentfragment".into()),
    });

    apis.push(WebApi {
        name: "Attr".into(),
        category: ApiCategory::Dom,
        test_expression: "typeof Attr === 'function'".into(),
        description: "Attr interface".into(),
        priority: Priority::High,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-attr".into()),
    });

    apis.push(WebApi {
        name: "CharacterData".into(),
        category: ApiCategory::Dom,
        test_expression: "typeof CharacterData === 'function'".into(),
        description: "CharacterData interface".into(),
        priority: Priority::High,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-characterdata".into()),
    });

    apis.push(WebApi {
        name: "DOMImplementation".into(),
        category: ApiCategory::Dom,
        test_expression: "typeof document.implementation === 'object'".into(),
        description: "DOMImplementation interface".into(),
        priority: Priority::Medium,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-domimplementation".into()),
    });

    apis.push(WebApi {
        name: "DOMTokenList".into(),
        category: ApiCategory::Dom,
        test_expression: "typeof DOMTokenList === 'function'".into(),
        description: "DOMTokenList for classList etc".into(),
        priority: Priority::Critical,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-domtokenlist".into()),
    });

    apis.push(WebApi {
        name: "NamedNodeMap".into(),
        category: ApiCategory::Dom,
        test_expression: "typeof NamedNodeMap === 'function'".into(),
        description: "NamedNodeMap for attributes".into(),
        priority: Priority::High,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-namednodemap".into()),
    });

    apis.push(WebApi {
        name: "NodeList".into(),
        category: ApiCategory::Dom,
        test_expression: "typeof NodeList === 'function'".into(),
        description: "NodeList interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-nodelist".into()),
    });

    apis.push(WebApi {
        name: "HTMLCollection".into(),
        category: ApiCategory::Dom,
        test_expression: "typeof HTMLCollection === 'function'".into(),
        description: "HTMLCollection interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-htmlcollection".into()),
    });

    // DOM Traversal
    apis.push(WebApi {
        name: "Range".into(),
        category: ApiCategory::DomTraversal,
        test_expression: "typeof Range === 'function'".into(),
        description: "Range interface for selections".into(),
        priority: Priority::High,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-range".into()),
    });

    apis.push(WebApi {
        name: "TreeWalker".into(),
        category: ApiCategory::DomTraversal,
        test_expression: "typeof TreeWalker === 'function'".into(),
        description: "TreeWalker for DOM traversal".into(),
        priority: Priority::Medium,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-treewalker".into()),
    });

    apis.push(WebApi {
        name: "NodeIterator".into(),
        category: ApiCategory::DomTraversal,
        test_expression: "typeof NodeIterator === 'function'".into(),
        description: "NodeIterator for DOM traversal".into(),
        priority: Priority::Medium,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-nodeiterator".into()),
    });

    apis.push(WebApi {
        name: "Selection".into(),
        category: ApiCategory::DomTraversal,
        test_expression: "typeof Selection === 'function' || typeof window.getSelection === 'function'".into(),
        description: "Selection API".into(),
        priority: Priority::High,
        spec_url: Some("https://w3c.github.io/selection-api/#selection-interface".into()),
    });

    // =========================================================================
    // DOM EVENTS
    // =========================================================================

    apis.push(WebApi {
        name: "Event".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof Event === 'function'".into(),
        description: "Base Event interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-event".into()),
    });

    apis.push(WebApi {
        name: "EventTarget".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof EventTarget === 'function'".into(),
        description: "EventTarget interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-eventtarget".into()),
    });

    apis.push(WebApi {
        name: "CustomEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof CustomEvent === 'function'".into(),
        description: "CustomEvent interface".into(),
        priority: Priority::High,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-customevent".into()),
    });

    apis.push(WebApi {
        name: "MouseEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof MouseEvent === 'function'".into(),
        description: "MouseEvent interface".into(),
        priority: Priority::High,
        spec_url: Some("https://w3c.github.io/uievents/#interface-mouseevent".into()),
    });

    apis.push(WebApi {
        name: "KeyboardEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof KeyboardEvent === 'function'".into(),
        description: "KeyboardEvent interface".into(),
        priority: Priority::High,
        spec_url: Some("https://w3c.github.io/uievents/#interface-keyboardevent".into()),
    });

    apis.push(WebApi {
        name: "FocusEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof FocusEvent === 'function'".into(),
        description: "FocusEvent interface".into(),
        priority: Priority::High,
        spec_url: Some("https://w3c.github.io/uievents/#interface-focusevent".into()),
    });

    apis.push(WebApi {
        name: "InputEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof InputEvent === 'function'".into(),
        description: "InputEvent interface".into(),
        priority: Priority::High,
        spec_url: Some("https://w3c.github.io/uievents/#interface-inputevent".into()),
    });

    apis.push(WebApi {
        name: "WheelEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof WheelEvent === 'function'".into(),
        description: "WheelEvent interface".into(),
        priority: Priority::Medium,
        spec_url: Some("https://w3c.github.io/uievents/#interface-wheelevent".into()),
    });

    apis.push(WebApi {
        name: "TouchEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof TouchEvent === 'function'".into(),
        description: "TouchEvent interface".into(),
        priority: Priority::Medium,
        spec_url: Some("https://w3c.github.io/touch-events/#touchevent-interface".into()),
    });

    apis.push(WebApi {
        name: "PointerEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof PointerEvent === 'function'".into(),
        description: "PointerEvent interface".into(),
        priority: Priority::Medium,
        spec_url: Some("https://w3c.github.io/pointerevents/#pointerevent-interface".into()),
    });

    apis.push(WebApi {
        name: "DragEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof DragEvent === 'function'".into(),
        description: "DragEvent interface".into(),
        priority: Priority::Medium,
        spec_url: Some("https://html.spec.whatwg.org/multipage/dnd.html#the-dragevent-interface".into()),
    });

    apis.push(WebApi {
        name: "ErrorEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof ErrorEvent === 'function'".into(),
        description: "ErrorEvent interface".into(),
        priority: Priority::High,
        spec_url: Some("https://html.spec.whatwg.org/multipage/webappapis.html#errorevent".into()),
    });

    apis.push(WebApi {
        name: "MessageEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof MessageEvent === 'function'".into(),
        description: "MessageEvent interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://html.spec.whatwg.org/multipage/comms.html#messageevent".into()),
    });

    apis.push(WebApi {
        name: "ProgressEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof ProgressEvent === 'function'".into(),
        description: "ProgressEvent interface".into(),
        priority: Priority::High,
        spec_url: Some("https://xhr.spec.whatwg.org/#interface-progressevent".into()),
    });

    apis.push(WebApi {
        name: "HashChangeEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof HashChangeEvent === 'function'".into(),
        description: "HashChangeEvent interface".into(),
        priority: Priority::Medium,
        spec_url: Some("https://html.spec.whatwg.org/multipage/browsing-the-web.html#hashchangeevent".into()),
    });

    apis.push(WebApi {
        name: "PopStateEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof PopStateEvent === 'function'".into(),
        description: "PopStateEvent interface".into(),
        priority: Priority::High,
        spec_url: Some("https://html.spec.whatwg.org/multipage/browsing-the-web.html#popstateevent".into()),
    });

    apis.push(WebApi {
        name: "StorageEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof StorageEvent === 'function'".into(),
        description: "StorageEvent interface".into(),
        priority: Priority::High,
        spec_url: Some("https://html.spec.whatwg.org/multipage/webstorage.html#storageevent".into()),
    });

    apis.push(WebApi {
        name: "ClipboardEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof ClipboardEvent === 'function'".into(),
        description: "ClipboardEvent interface".into(),
        priority: Priority::Medium,
        spec_url: Some("https://w3c.github.io/clipboard-apis/#clipboard-event-interfaces".into()),
    });

    apis.push(WebApi {
        name: "AnimationEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof AnimationEvent === 'function'".into(),
        description: "AnimationEvent interface".into(),
        priority: Priority::Medium,
        spec_url: Some("https://drafts.csswg.org/css-animations/#interface-animationevent".into()),
    });

    apis.push(WebApi {
        name: "TransitionEvent".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof TransitionEvent === 'function'".into(),
        description: "TransitionEvent interface".into(),
        priority: Priority::Medium,
        spec_url: Some("https://drafts.csswg.org/css-transitions/#interface-transitionevent".into()),
    });

    // AbortController/AbortSignal
    apis.push(WebApi {
        name: "AbortController".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof AbortController === 'function'".into(),
        description: "AbortController interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-abortcontroller".into()),
    });

    apis.push(WebApi {
        name: "AbortSignal".into(),
        category: ApiCategory::DomEvents,
        test_expression: "typeof AbortSignal === 'function'".into(),
        description: "AbortSignal interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-abortsignal".into()),
    });

    // =========================================================================
    // MUTATION OBSERVER
    // =========================================================================

    apis.push(WebApi {
        name: "MutationObserver".into(),
        category: ApiCategory::MutationObserver,
        test_expression: "typeof MutationObserver === 'function'".into(),
        description: "MutationObserver interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-mutationobserver".into()),
    });

    apis.push(WebApi {
        name: "MutationRecord".into(),
        category: ApiCategory::MutationObserver,
        test_expression: "typeof MutationRecord === 'function'".into(),
        description: "MutationRecord interface".into(),
        priority: Priority::High,
        spec_url: Some("https://dom.spec.whatwg.org/#interface-mutationrecord".into()),
    });

    // =========================================================================
    // HTML ELEMENTS
    // =========================================================================

    apis.push(WebApi {
        name: "HTMLElement".into(),
        category: ApiCategory::HtmlElements,
        test_expression: "typeof HTMLElement === 'function'".into(),
        description: "HTMLElement base interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://html.spec.whatwg.org/multipage/dom.html#htmlelement".into()),
    });

    apis.push(WebApi {
        name: "HTMLDivElement".into(),
        category: ApiCategory::HtmlElements,
        test_expression: "typeof HTMLDivElement === 'function'".into(),
        description: "HTMLDivElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLSpanElement".into(),
        category: ApiCategory::HtmlElements,
        test_expression: "typeof HTMLSpanElement === 'function'".into(),
        description: "HTMLSpanElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLAnchorElement".into(),
        category: ApiCategory::HtmlElements,
        test_expression: "typeof HTMLAnchorElement === 'function'".into(),
        description: "HTMLAnchorElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLImageElement".into(),
        category: ApiCategory::HtmlElements,
        test_expression: "typeof HTMLImageElement === 'function'".into(),
        description: "HTMLImageElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLScriptElement".into(),
        category: ApiCategory::HtmlElements,
        test_expression: "typeof HTMLScriptElement === 'function'".into(),
        description: "HTMLScriptElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLStyleElement".into(),
        category: ApiCategory::HtmlElements,
        test_expression: "typeof HTMLStyleElement === 'function'".into(),
        description: "HTMLStyleElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLLinkElement".into(),
        category: ApiCategory::HtmlElements,
        test_expression: "typeof HTMLLinkElement === 'function'".into(),
        description: "HTMLLinkElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLHeadElement".into(),
        category: ApiCategory::HtmlElements,
        test_expression: "typeof HTMLHeadElement === 'function'".into(),
        description: "HTMLHeadElement interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLBodyElement".into(),
        category: ApiCategory::HtmlElements,
        test_expression: "typeof HTMLBodyElement === 'function'".into(),
        description: "HTMLBodyElement interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLTemplateElement".into(),
        category: ApiCategory::HtmlElements,
        test_expression: "typeof HTMLTemplateElement === 'function'".into(),
        description: "HTMLTemplateElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLSlotElement".into(),
        category: ApiCategory::HtmlElements,
        test_expression: "typeof HTMLSlotElement === 'function'".into(),
        description: "HTMLSlotElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    // Forms
    apis.push(WebApi {
        name: "HTMLFormElement".into(),
        category: ApiCategory::HtmlForms,
        test_expression: "typeof HTMLFormElement === 'function'".into(),
        description: "HTMLFormElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLInputElement".into(),
        category: ApiCategory::HtmlForms,
        test_expression: "typeof HTMLInputElement === 'function'".into(),
        description: "HTMLInputElement interface".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLButtonElement".into(),
        category: ApiCategory::HtmlForms,
        test_expression: "typeof HTMLButtonElement === 'function'".into(),
        description: "HTMLButtonElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLSelectElement".into(),
        category: ApiCategory::HtmlForms,
        test_expression: "typeof HTMLSelectElement === 'function'".into(),
        description: "HTMLSelectElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLOptionElement".into(),
        category: ApiCategory::HtmlForms,
        test_expression: "typeof HTMLOptionElement === 'function'".into(),
        description: "HTMLOptionElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLTextAreaElement".into(),
        category: ApiCategory::HtmlForms,
        test_expression: "typeof HTMLTextAreaElement === 'function'".into(),
        description: "HTMLTextAreaElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLLabelElement".into(),
        category: ApiCategory::HtmlForms,
        test_expression: "typeof HTMLLabelElement === 'function'".into(),
        description: "HTMLLabelElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "FormData".into(),
        category: ApiCategory::HtmlForms,
        test_expression: "typeof FormData === 'function'".into(),
        description: "FormData interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://xhr.spec.whatwg.org/#interface-formdata".into()),
    });

    apis.push(WebApi {
        name: "ValidityState".into(),
        category: ApiCategory::HtmlForms,
        test_expression: "typeof ValidityState === 'function'".into(),
        description: "ValidityState interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    // Media Elements
    apis.push(WebApi {
        name: "HTMLMediaElement".into(),
        category: ApiCategory::HtmlMedia,
        test_expression: "typeof HTMLMediaElement === 'function'".into(),
        description: "HTMLMediaElement base interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLVideoElement".into(),
        category: ApiCategory::HtmlMedia,
        test_expression: "typeof HTMLVideoElement === 'function'".into(),
        description: "HTMLVideoElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "HTMLAudioElement".into(),
        category: ApiCategory::HtmlMedia,
        test_expression: "typeof HTMLAudioElement === 'function'".into(),
        description: "HTMLAudioElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "Audio".into(),
        category: ApiCategory::HtmlMedia,
        test_expression: "typeof Audio === 'function'".into(),
        description: "Audio constructor".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "Image".into(),
        category: ApiCategory::HtmlMedia,
        test_expression: "typeof Image === 'function'".into(),
        description: "Image constructor".into(),
        priority: Priority::High,
        spec_url: None,
    });

    // Canvas
    apis.push(WebApi {
        name: "HTMLCanvasElement".into(),
        category: ApiCategory::HtmlCanvas,
        test_expression: "typeof HTMLCanvasElement === 'function'".into(),
        description: "HTMLCanvasElement interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "OffscreenCanvas".into(),
        category: ApiCategory::HtmlCanvas,
        test_expression: "typeof OffscreenCanvas === 'function'".into(),
        description: "OffscreenCanvas interface".into(),
        priority: Priority::Medium,
        spec_url: Some("https://html.spec.whatwg.org/multipage/canvas.html#the-offscreencanvas-interface".into()),
    });

    apis.push(WebApi {
        name: "ImageBitmap".into(),
        category: ApiCategory::HtmlCanvas,
        test_expression: "typeof ImageBitmap === 'function'".into(),
        description: "ImageBitmap interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "ImageData".into(),
        category: ApiCategory::HtmlCanvas,
        test_expression: "typeof ImageData === 'function'".into(),
        description: "ImageData interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "Path2D".into(),
        category: ApiCategory::HtmlCanvas,
        test_expression: "typeof Path2D === 'function'".into(),
        description: "Path2D interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "CanvasGradient".into(),
        category: ApiCategory::HtmlCanvas,
        test_expression: "typeof CanvasGradient === 'function'".into(),
        description: "CanvasGradient interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "CanvasPattern".into(),
        category: ApiCategory::HtmlCanvas,
        test_expression: "typeof CanvasPattern === 'function'".into(),
        description: "CanvasPattern interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    // =========================================================================
    // CSSOM
    // =========================================================================

    apis.push(WebApi {
        name: "CSSStyleDeclaration".into(),
        category: ApiCategory::Cssom,
        test_expression: "typeof CSSStyleDeclaration === 'function'".into(),
        description: "CSSStyleDeclaration interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://drafts.csswg.org/cssom/#the-cssstyledeclaration-interface".into()),
    });

    apis.push(WebApi {
        name: "CSSRule".into(),
        category: ApiCategory::Cssom,
        test_expression: "typeof CSSRule === 'function'".into(),
        description: "CSSRule interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "CSSStyleSheet".into(),
        category: ApiCategory::Cssom,
        test_expression: "typeof CSSStyleSheet === 'function'".into(),
        description: "CSSStyleSheet interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "StyleSheet".into(),
        category: ApiCategory::Cssom,
        test_expression: "typeof StyleSheet === 'function'".into(),
        description: "StyleSheet interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "StyleSheetList".into(),
        category: ApiCategory::Cssom,
        test_expression: "typeof StyleSheetList === 'function'".into(),
        description: "StyleSheetList interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "MediaQueryList".into(),
        category: ApiCategory::Cssom,
        test_expression: "typeof MediaQueryList === 'function'".into(),
        description: "MediaQueryList interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "CSS".into(),
        category: ApiCategory::Cssom,
        test_expression: "typeof CSS === 'object'".into(),
        description: "CSS namespace object".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "DOMRect".into(),
        category: ApiCategory::Cssom,
        test_expression: "typeof DOMRect === 'function'".into(),
        description: "DOMRect interface".into(),
        priority: Priority::High,
        spec_url: Some("https://drafts.fxtf.org/geometry/#DOMRect".into()),
    });

    apis.push(WebApi {
        name: "DOMRectReadOnly".into(),
        category: ApiCategory::Cssom,
        test_expression: "typeof DOMRectReadOnly === 'function'".into(),
        description: "DOMRectReadOnly interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "DOMPoint".into(),
        category: ApiCategory::Cssom,
        test_expression: "typeof DOMPoint === 'function'".into(),
        description: "DOMPoint interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "DOMMatrix".into(),
        category: ApiCategory::Cssom,
        test_expression: "typeof DOMMatrix === 'function'".into(),
        description: "DOMMatrix interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    // =========================================================================
    // STORAGE APIs
    // =========================================================================

    apis.push(WebApi {
        name: "localStorage".into(),
        category: ApiCategory::WebStorage,
        test_expression: "typeof localStorage === 'object'".into(),
        description: "localStorage API".into(),
        priority: Priority::Critical,
        spec_url: Some("https://html.spec.whatwg.org/multipage/webstorage.html#dom-localstorage".into()),
    });

    apis.push(WebApi {
        name: "sessionStorage".into(),
        category: ApiCategory::WebStorage,
        test_expression: "typeof sessionStorage === 'object'".into(),
        description: "sessionStorage API".into(),
        priority: Priority::Critical,
        spec_url: Some("https://html.spec.whatwg.org/multipage/webstorage.html#dom-sessionstorage".into()),
    });

    apis.push(WebApi {
        name: "Storage".into(),
        category: ApiCategory::WebStorage,
        test_expression: "typeof Storage === 'function'".into(),
        description: "Storage interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    // IndexedDB
    apis.push(WebApi {
        name: "indexedDB".into(),
        category: ApiCategory::IndexedDb,
        test_expression: "typeof indexedDB === 'object'".into(),
        description: "IndexedDB API".into(),
        priority: Priority::Critical,
        spec_url: Some("https://w3c.github.io/IndexedDB/".into()),
    });

    apis.push(WebApi {
        name: "IDBFactory".into(),
        category: ApiCategory::IndexedDb,
        test_expression: "typeof IDBFactory === 'function'".into(),
        description: "IDBFactory interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "IDBDatabase".into(),
        category: ApiCategory::IndexedDb,
        test_expression: "typeof IDBDatabase === 'function'".into(),
        description: "IDBDatabase interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "IDBTransaction".into(),
        category: ApiCategory::IndexedDb,
        test_expression: "typeof IDBTransaction === 'function'".into(),
        description: "IDBTransaction interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "IDBObjectStore".into(),
        category: ApiCategory::IndexedDb,
        test_expression: "typeof IDBObjectStore === 'function'".into(),
        description: "IDBObjectStore interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "IDBIndex".into(),
        category: ApiCategory::IndexedDb,
        test_expression: "typeof IDBIndex === 'function'".into(),
        description: "IDBIndex interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "IDBCursor".into(),
        category: ApiCategory::IndexedDb,
        test_expression: "typeof IDBCursor === 'function'".into(),
        description: "IDBCursor interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "IDBKeyRange".into(),
        category: ApiCategory::IndexedDb,
        test_expression: "typeof IDBKeyRange === 'function'".into(),
        description: "IDBKeyRange interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "IDBRequest".into(),
        category: ApiCategory::IndexedDb,
        test_expression: "typeof IDBRequest === 'function'".into(),
        description: "IDBRequest interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    // Cache API
    apis.push(WebApi {
        name: "caches".into(),
        category: ApiCategory::CacheApi,
        test_expression: "typeof caches === 'object'".into(),
        description: "Cache Storage API".into(),
        priority: Priority::High,
        spec_url: Some("https://w3c.github.io/ServiceWorker/#cache-storage".into()),
    });

    apis.push(WebApi {
        name: "Cache".into(),
        category: ApiCategory::CacheApi,
        test_expression: "typeof Cache === 'function'".into(),
        description: "Cache interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "CacheStorage".into(),
        category: ApiCategory::CacheApi,
        test_expression: "typeof CacheStorage === 'function'".into(),
        description: "CacheStorage interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    // =========================================================================
    // FETCH API
    // =========================================================================

    apis.push(WebApi {
        name: "fetch".into(),
        category: ApiCategory::Fetch,
        test_expression: "typeof fetch === 'function'".into(),
        description: "Fetch API".into(),
        priority: Priority::Critical,
        spec_url: Some("https://fetch.spec.whatwg.org/".into()),
    });

    apis.push(WebApi {
        name: "Request".into(),
        category: ApiCategory::Fetch,
        test_expression: "typeof Request === 'function'".into(),
        description: "Request interface".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "Response".into(),
        category: ApiCategory::Fetch,
        test_expression: "typeof Response === 'function'".into(),
        description: "Response interface".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "Headers".into(),
        category: ApiCategory::Fetch,
        test_expression: "typeof Headers === 'function'".into(),
        description: "Headers interface".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    // XMLHttpRequest
    apis.push(WebApi {
        name: "XMLHttpRequest".into(),
        category: ApiCategory::Xhr,
        test_expression: "typeof XMLHttpRequest === 'function'".into(),
        description: "XMLHttpRequest interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://xhr.spec.whatwg.org/".into()),
    });

    apis.push(WebApi {
        name: "XMLHttpRequestEventTarget".into(),
        category: ApiCategory::Xhr,
        test_expression: "typeof XMLHttpRequestEventTarget === 'function'".into(),
        description: "XMLHttpRequestEventTarget interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "XMLHttpRequestUpload".into(),
        category: ApiCategory::Xhr,
        test_expression: "typeof XMLHttpRequestUpload === 'function'".into(),
        description: "XMLHttpRequestUpload interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    // WebSocket
    apis.push(WebApi {
        name: "WebSocket".into(),
        category: ApiCategory::WebSockets,
        test_expression: "typeof WebSocket === 'function'".into(),
        description: "WebSocket interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://websockets.spec.whatwg.org/".into()),
    });

    apis.push(WebApi {
        name: "CloseEvent".into(),
        category: ApiCategory::WebSockets,
        test_expression: "typeof CloseEvent === 'function'".into(),
        description: "CloseEvent interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    // =========================================================================
    // WEB WORKERS
    // =========================================================================

    apis.push(WebApi {
        name: "Worker".into(),
        category: ApiCategory::WebWorkers,
        test_expression: "typeof Worker === 'function'".into(),
        description: "Worker interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://html.spec.whatwg.org/multipage/workers.html#worker".into()),
    });

    apis.push(WebApi {
        name: "SharedWorker".into(),
        category: ApiCategory::SharedWorkers,
        test_expression: "typeof SharedWorker === 'function'".into(),
        description: "SharedWorker interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "navigator.serviceWorker".into(),
        category: ApiCategory::ServiceWorkers,
        test_expression: "typeof navigator.serviceWorker === 'object'".into(),
        description: "ServiceWorkerContainer".into(),
        priority: Priority::High,
        spec_url: Some("https://w3c.github.io/ServiceWorker/".into()),
    });

    // =========================================================================
    // MESSAGING
    // =========================================================================

    apis.push(WebApi {
        name: "MessageChannel".into(),
        category: ApiCategory::ChannelMessaging,
        test_expression: "typeof MessageChannel === 'function'".into(),
        description: "MessageChannel interface".into(),
        priority: Priority::High,
        spec_url: Some("https://html.spec.whatwg.org/multipage/web-messaging.html#message-channels".into()),
    });

    apis.push(WebApi {
        name: "MessagePort".into(),
        category: ApiCategory::ChannelMessaging,
        test_expression: "typeof MessagePort === 'function'".into(),
        description: "MessagePort interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "BroadcastChannel".into(),
        category: ApiCategory::BroadcastChannel,
        test_expression: "typeof BroadcastChannel === 'function'".into(),
        description: "BroadcastChannel interface".into(),
        priority: Priority::Medium,
        spec_url: Some("https://html.spec.whatwg.org/multipage/web-messaging.html#broadcasting-to-other-browsing-contexts".into()),
    });

    // =========================================================================
    // NAVIGATOR / DEVICE APIs
    // =========================================================================

    apis.push(WebApi {
        name: "navigator".into(),
        category: ApiCategory::Navigator,
        test_expression: "typeof navigator === 'object'".into(),
        description: "Navigator object".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "navigator.userAgent".into(),
        category: ApiCategory::Navigator,
        test_expression: "typeof navigator.userAgent === 'string'".into(),
        description: "User agent string".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "navigator.language".into(),
        category: ApiCategory::Navigator,
        test_expression: "typeof navigator.language === 'string'".into(),
        description: "Browser language".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "navigator.languages".into(),
        category: ApiCategory::Navigator,
        test_expression: "Array.isArray(navigator.languages)".into(),
        description: "Preferred languages".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "navigator.onLine".into(),
        category: ApiCategory::Navigator,
        test_expression: "typeof navigator.onLine === 'boolean'".into(),
        description: "Online status".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "navigator.cookieEnabled".into(),
        category: ApiCategory::Navigator,
        test_expression: "typeof navigator.cookieEnabled === 'boolean'".into(),
        description: "Cookie support".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "navigator.hardwareConcurrency".into(),
        category: ApiCategory::Navigator,
        test_expression: "typeof navigator.hardwareConcurrency === 'number'".into(),
        description: "CPU core count".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "navigator.maxTouchPoints".into(),
        category: ApiCategory::Navigator,
        test_expression: "typeof navigator.maxTouchPoints === 'number'".into(),
        description: "Touch support".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "navigator.clipboard".into(),
        category: ApiCategory::Clipboard,
        test_expression: "typeof navigator.clipboard === 'object'".into(),
        description: "Clipboard API".into(),
        priority: Priority::High,
        spec_url: Some("https://w3c.github.io/clipboard-apis/".into()),
    });

    apis.push(WebApi {
        name: "navigator.permissions".into(),
        category: ApiCategory::Permissions,
        test_expression: "typeof navigator.permissions === 'object'".into(),
        description: "Permissions API".into(),
        priority: Priority::High,
        spec_url: Some("https://w3c.github.io/permissions/".into()),
    });

    apis.push(WebApi {
        name: "navigator.geolocation".into(),
        category: ApiCategory::Geolocation,
        test_expression: "typeof navigator.geolocation === 'object'".into(),
        description: "Geolocation API".into(),
        priority: Priority::Medium,
        spec_url: Some("https://w3c.github.io/geolocation-api/".into()),
    });

    apis.push(WebApi {
        name: "navigator.mediaDevices".into(),
        category: ApiCategory::MediaDevices,
        test_expression: "typeof navigator.mediaDevices === 'object'".into(),
        description: "MediaDevices API".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "navigator.locks".into(),
        category: ApiCategory::Navigator,
        test_expression: "typeof navigator.locks === 'object'".into(),
        description: "Web Locks API".into(),
        priority: Priority::Medium,
        spec_url: Some("https://w3c.github.io/web-locks/".into()),
    });

    apis.push(WebApi {
        name: "navigator.storage".into(),
        category: ApiCategory::Navigator,
        test_expression: "typeof navigator.storage === 'object'".into(),
        description: "StorageManager API".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "navigator.sendBeacon".into(),
        category: ApiCategory::Navigator,
        test_expression: "typeof navigator.sendBeacon === 'function'".into(),
        description: "Beacon API".into(),
        priority: Priority::Medium,
        spec_url: Some("https://w3c.github.io/beacon/".into()),
    });

    apis.push(WebApi {
        name: "navigator.vibrate".into(),
        category: ApiCategory::Vibration,
        test_expression: "typeof navigator.vibrate === 'function'".into(),
        description: "Vibration API".into(),
        priority: Priority::Low,
        spec_url: None,
    });

    // =========================================================================
    // WEB AUDIO
    // =========================================================================

    apis.push(WebApi {
        name: "AudioContext".into(),
        category: ApiCategory::WebAudio,
        test_expression: "typeof AudioContext === 'function'".into(),
        description: "AudioContext interface".into(),
        priority: Priority::High,
        spec_url: Some("https://webaudio.github.io/web-audio-api/".into()),
    });

    apis.push(WebApi {
        name: "OfflineAudioContext".into(),
        category: ApiCategory::WebAudio,
        test_expression: "typeof OfflineAudioContext === 'function'".into(),
        description: "OfflineAudioContext interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "AudioBuffer".into(),
        category: ApiCategory::WebAudio,
        test_expression: "typeof AudioBuffer === 'function'".into(),
        description: "AudioBuffer interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "AudioNode".into(),
        category: ApiCategory::WebAudio,
        test_expression: "typeof AudioNode === 'function'".into(),
        description: "AudioNode interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "GainNode".into(),
        category: ApiCategory::WebAudio,
        test_expression: "typeof GainNode === 'function'".into(),
        description: "GainNode interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "OscillatorNode".into(),
        category: ApiCategory::WebAudio,
        test_expression: "typeof OscillatorNode === 'function'".into(),
        description: "OscillatorNode interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    // =========================================================================
    // WEBGL
    // =========================================================================

    apis.push(WebApi {
        name: "WebGLRenderingContext".into(),
        category: ApiCategory::WebGl,
        test_expression: "typeof WebGLRenderingContext === 'function'".into(),
        description: "WebGL 1.0 context".into(),
        priority: Priority::High,
        spec_url: Some("https://www.khronos.org/registry/webgl/specs/latest/1.0/".into()),
    });

    apis.push(WebApi {
        name: "WebGL2RenderingContext".into(),
        category: ApiCategory::WebGl2,
        test_expression: "typeof WebGL2RenderingContext === 'function'".into(),
        description: "WebGL 2.0 context".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    // =========================================================================
    // CRYPTO
    // =========================================================================

    apis.push(WebApi {
        name: "crypto".into(),
        category: ApiCategory::Crypto,
        test_expression: "typeof crypto === 'object'".into(),
        description: "Crypto API".into(),
        priority: Priority::Critical,
        spec_url: Some("https://w3c.github.io/webcrypto/".into()),
    });

    apis.push(WebApi {
        name: "crypto.subtle".into(),
        category: ApiCategory::Crypto,
        test_expression: "typeof crypto.subtle === 'object'".into(),
        description: "SubtleCrypto API".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "crypto.getRandomValues".into(),
        category: ApiCategory::Crypto,
        test_expression: "typeof crypto.getRandomValues === 'function'".into(),
        description: "Random values generation".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "crypto.randomUUID".into(),
        category: ApiCategory::Crypto,
        test_expression: "typeof crypto.randomUUID === 'function'".into(),
        description: "UUID generation".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "CryptoKey".into(),
        category: ApiCategory::Crypto,
        test_expression: "typeof CryptoKey === 'function'".into(),
        description: "CryptoKey interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    // =========================================================================
    // PERFORMANCE
    // =========================================================================

    apis.push(WebApi {
        name: "performance".into(),
        category: ApiCategory::PerformanceApi,
        test_expression: "typeof performance === 'object'".into(),
        description: "Performance API".into(),
        priority: Priority::Critical,
        spec_url: Some("https://w3c.github.io/hr-time/".into()),
    });

    apis.push(WebApi {
        name: "performance.now".into(),
        category: ApiCategory::PerformanceApi,
        test_expression: "typeof performance.now === 'function'".into(),
        description: "High-resolution timestamp".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "performance.mark".into(),
        category: ApiCategory::PerformanceApi,
        test_expression: "typeof performance.mark === 'function'".into(),
        description: "Performance marks".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "performance.measure".into(),
        category: ApiCategory::PerformanceApi,
        test_expression: "typeof performance.measure === 'function'".into(),
        description: "Performance measures".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "PerformanceObserver".into(),
        category: ApiCategory::PerformanceApi,
        test_expression: "typeof PerformanceObserver === 'function'".into(),
        description: "PerformanceObserver interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "performance.timing".into(),
        category: ApiCategory::PerformanceApi,
        test_expression: "typeof performance.timing === 'object'".into(),
        description: "Navigation timing (deprecated but widely used)".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "performance.getEntries".into(),
        category: ApiCategory::PerformanceApi,
        test_expression: "typeof performance.getEntries === 'function'".into(),
        description: "Performance entries".into(),
        priority: Priority::High,
        spec_url: None,
    });

    // Observers
    apis.push(WebApi {
        name: "IntersectionObserver".into(),
        category: ApiCategory::IntersectionObserver,
        test_expression: "typeof IntersectionObserver === 'function'".into(),
        description: "IntersectionObserver interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://w3c.github.io/IntersectionObserver/".into()),
    });

    apis.push(WebApi {
        name: "ResizeObserver".into(),
        category: ApiCategory::ResizeObserver,
        test_expression: "typeof ResizeObserver === 'function'".into(),
        description: "ResizeObserver interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://drafts.csswg.org/resize-observer/".into()),
    });

    // =========================================================================
    // FILE API
    // =========================================================================

    apis.push(WebApi {
        name: "File".into(),
        category: ApiCategory::FileApi,
        test_expression: "typeof File === 'function'".into(),
        description: "File interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://w3c.github.io/FileAPI/".into()),
    });

    apis.push(WebApi {
        name: "Blob".into(),
        category: ApiCategory::FileApi,
        test_expression: "typeof Blob === 'function'".into(),
        description: "Blob interface".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "FileReader".into(),
        category: ApiCategory::FileApi,
        test_expression: "typeof FileReader === 'function'".into(),
        description: "FileReader interface".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "FileList".into(),
        category: ApiCategory::FileApi,
        test_expression: "typeof FileList === 'function'".into(),
        description: "FileList interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    // =========================================================================
    // STREAMS
    // =========================================================================

    apis.push(WebApi {
        name: "ReadableStream".into(),
        category: ApiCategory::Streams,
        test_expression: "typeof ReadableStream === 'function'".into(),
        description: "ReadableStream interface".into(),
        priority: Priority::High,
        spec_url: Some("https://streams.spec.whatwg.org/".into()),
    });

    apis.push(WebApi {
        name: "WritableStream".into(),
        category: ApiCategory::Streams,
        test_expression: "typeof WritableStream === 'function'".into(),
        description: "WritableStream interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "TransformStream".into(),
        category: ApiCategory::Streams,
        test_expression: "typeof TransformStream === 'function'".into(),
        description: "TransformStream interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "ByteLengthQueuingStrategy".into(),
        category: ApiCategory::Streams,
        test_expression: "typeof ByteLengthQueuingStrategy === 'function'".into(),
        description: "ByteLengthQueuingStrategy interface".into(),
        priority: Priority::Low,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "CountQueuingStrategy".into(),
        category: ApiCategory::Streams,
        test_expression: "typeof CountQueuingStrategy === 'function'".into(),
        description: "CountQueuingStrategy interface".into(),
        priority: Priority::Low,
        spec_url: None,
    });

    // =========================================================================
    // ENCODING
    // =========================================================================

    apis.push(WebApi {
        name: "TextEncoder".into(),
        category: ApiCategory::Encoding,
        test_expression: "typeof TextEncoder === 'function'".into(),
        description: "TextEncoder interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://encoding.spec.whatwg.org/".into()),
    });

    apis.push(WebApi {
        name: "TextDecoder".into(),
        category: ApiCategory::Encoding,
        test_expression: "typeof TextDecoder === 'function'".into(),
        description: "TextDecoder interface".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "TextEncoderStream".into(),
        category: ApiCategory::Encoding,
        test_expression: "typeof TextEncoderStream === 'function'".into(),
        description: "TextEncoderStream interface".into(),
        priority: Priority::Low,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "TextDecoderStream".into(),
        category: ApiCategory::Encoding,
        test_expression: "typeof TextDecoderStream === 'function'".into(),
        description: "TextDecoderStream interface".into(),
        priority: Priority::Low,
        spec_url: None,
    });

    // =========================================================================
    // URL
    // =========================================================================

    apis.push(WebApi {
        name: "URL".into(),
        category: ApiCategory::Url,
        test_expression: "typeof URL === 'function'".into(),
        description: "URL interface".into(),
        priority: Priority::Critical,
        spec_url: Some("https://url.spec.whatwg.org/".into()),
    });

    apis.push(WebApi {
        name: "URLSearchParams".into(),
        category: ApiCategory::Url,
        test_expression: "typeof URLSearchParams === 'function'".into(),
        description: "URLSearchParams interface".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    // =========================================================================
    // WEB COMPONENTS
    // =========================================================================

    apis.push(WebApi {
        name: "customElements".into(),
        category: ApiCategory::WebComponents,
        test_expression: "typeof customElements === 'object'".into(),
        description: "Custom Elements registry".into(),
        priority: Priority::High,
        spec_url: Some("https://html.spec.whatwg.org/multipage/custom-elements.html".into()),
    });

    apis.push(WebApi {
        name: "customElements.define".into(),
        category: ApiCategory::WebComponents,
        test_expression: "typeof customElements.define === 'function'".into(),
        description: "Custom element registration".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "ShadowRoot".into(),
        category: ApiCategory::WebComponents,
        test_expression: "typeof ShadowRoot === 'function'".into(),
        description: "ShadowRoot interface".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "Element.attachShadow".into(),
        category: ApiCategory::WebComponents,
        test_expression: "typeof Element.prototype.attachShadow === 'function'".into(),
        description: "Shadow DOM attachment".into(),
        priority: Priority::High,
        spec_url: None,
    });

    // =========================================================================
    // HISTORY / LOCATION
    // =========================================================================

    apis.push(WebApi {
        name: "history".into(),
        category: ApiCategory::History,
        test_expression: "typeof history === 'object'".into(),
        description: "History API".into(),
        priority: Priority::Critical,
        spec_url: Some("https://html.spec.whatwg.org/multipage/history.html".into()),
    });

    apis.push(WebApi {
        name: "history.pushState".into(),
        category: ApiCategory::History,
        test_expression: "typeof history.pushState === 'function'".into(),
        description: "Push state".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "history.replaceState".into(),
        category: ApiCategory::History,
        test_expression: "typeof history.replaceState === 'function'".into(),
        description: "Replace state".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "location".into(),
        category: ApiCategory::History,
        test_expression: "typeof location === 'object'".into(),
        description: "Location interface".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    // =========================================================================
    // NOTIFICATIONS
    // =========================================================================

    apis.push(WebApi {
        name: "Notification".into(),
        category: ApiCategory::Notifications,
        test_expression: "typeof Notification === 'function'".into(),
        description: "Notification API".into(),
        priority: Priority::Medium,
        spec_url: Some("https://notifications.spec.whatwg.org/".into()),
    });

    apis.push(WebApi {
        name: "Notification.permission".into(),
        category: ApiCategory::Notifications,
        test_expression: "typeof Notification.permission === 'string'".into(),
        description: "Notification permission".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    // =========================================================================
    // CLIPBOARD
    // =========================================================================

    apis.push(WebApi {
        name: "ClipboardItem".into(),
        category: ApiCategory::Clipboard,
        test_expression: "typeof ClipboardItem === 'function'".into(),
        description: "ClipboardItem interface".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    // =========================================================================
    // TIMERS
    // =========================================================================

    apis.push(WebApi {
        name: "setTimeout".into(),
        category: ApiCategory::Other,
        test_expression: "typeof setTimeout === 'function'".into(),
        description: "setTimeout function".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "setInterval".into(),
        category: ApiCategory::Other,
        test_expression: "typeof setInterval === 'function'".into(),
        description: "setInterval function".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "clearTimeout".into(),
        category: ApiCategory::Other,
        test_expression: "typeof clearTimeout === 'function'".into(),
        description: "clearTimeout function".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "clearInterval".into(),
        category: ApiCategory::Other,
        test_expression: "typeof clearInterval === 'function'".into(),
        description: "clearInterval function".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "requestAnimationFrame".into(),
        category: ApiCategory::Other,
        test_expression: "typeof requestAnimationFrame === 'function'".into(),
        description: "requestAnimationFrame function".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "cancelAnimationFrame".into(),
        category: ApiCategory::Other,
        test_expression: "typeof cancelAnimationFrame === 'function'".into(),
        description: "cancelAnimationFrame function".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "requestIdleCallback".into(),
        category: ApiCategory::Other,
        test_expression: "typeof requestIdleCallback === 'function'".into(),
        description: "requestIdleCallback function".into(),
        priority: Priority::Medium,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "queueMicrotask".into(),
        category: ApiCategory::Other,
        test_expression: "typeof queueMicrotask === 'function'".into(),
        description: "queueMicrotask function".into(),
        priority: Priority::High,
        spec_url: None,
    });

    // =========================================================================
    // STRUCTURED CLONE
    // =========================================================================

    apis.push(WebApi {
        name: "structuredClone".into(),
        category: ApiCategory::Other,
        test_expression: "typeof structuredClone === 'function'".into(),
        description: "structuredClone function".into(),
        priority: Priority::High,
        spec_url: Some("https://html.spec.whatwg.org/multipage/structured-data.html#dom-structuredclone".into()),
    });

    // =========================================================================
    // BASE64
    // =========================================================================

    apis.push(WebApi {
        name: "atob".into(),
        category: ApiCategory::Other,
        test_expression: "typeof atob === 'function'".into(),
        description: "Base64 decode".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "btoa".into(),
        category: ApiCategory::Other,
        test_expression: "typeof btoa === 'function'".into(),
        description: "Base64 encode".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    // =========================================================================
    // WINDOW
    // =========================================================================

    apis.push(WebApi {
        name: "window".into(),
        category: ApiCategory::Other,
        test_expression: "typeof window === 'object'".into(),
        description: "Window object".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "document".into(),
        category: ApiCategory::Other,
        test_expression: "typeof document === 'object'".into(),
        description: "Document object".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "console".into(),
        category: ApiCategory::Other,
        test_expression: "typeof console === 'object'".into(),
        description: "Console API".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "alert".into(),
        category: ApiCategory::Other,
        test_expression: "typeof alert === 'function'".into(),
        description: "Alert dialog".into(),
        priority: Priority::Low,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "confirm".into(),
        category: ApiCategory::Other,
        test_expression: "typeof confirm === 'function'".into(),
        description: "Confirm dialog".into(),
        priority: Priority::Low,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "prompt".into(),
        category: ApiCategory::Other,
        test_expression: "typeof prompt === 'function'".into(),
        description: "Prompt dialog".into(),
        priority: Priority::Low,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "getComputedStyle".into(),
        category: ApiCategory::Cssom,
        test_expression: "typeof getComputedStyle === 'function'".into(),
        description: "Get computed styles".into(),
        priority: Priority::Critical,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "matchMedia".into(),
        category: ApiCategory::Cssom,
        test_expression: "typeof matchMedia === 'function'".into(),
        description: "Media query matching".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis.push(WebApi {
        name: "screen".into(),
        category: ApiCategory::Other,
        test_expression: "typeof screen === 'object'".into(),
        description: "Screen object".into(),
        priority: Priority::High,
        spec_url: None,
    });

    apis
}

/// Generate JavaScript code to test all APIs
pub fn generate_test_script(apis: &[WebApi]) -> String {
    let mut script = String::from("(function() {\n  const results = {};\n");

    for api in apis {
        let safe_name = api.name.replace(".", "_").replace(" ", "_");
        script.push_str(&format!(
            "  try {{ results['{}'] = !!({}) }} catch(e) {{ results['{}'] = false }}\n",
            api.name, api.test_expression, api.name
        ));
    }

    script.push_str("  return results;\n})()");
    script
}

/// Print a formatted report
pub fn print_report(results: &[ApiTestResult]) {
    let mut by_category: HashMap<ApiCategory, Vec<&ApiTestResult>> = HashMap::new();

    for result in results {
        by_category.entry(result.api.category.clone())
            .or_default()
            .push(result);
    }

    let total = results.len();
    let implemented = results.iter().filter(|r| r.implemented).count();
    let critical_total = results.iter().filter(|r| r.api.priority == Priority::Critical).count();
    let critical_impl = results.iter().filter(|r| r.api.priority == Priority::Critical && r.implemented).count();
    let high_total = results.iter().filter(|r| r.api.priority == Priority::High).count();
    let high_impl = results.iter().filter(|r| r.api.priority == Priority::High && r.implemented).count();

    println!("\n╔══════════════════════════════════════════════════════════════════════╗");
    println!("║               THALORA WEB API COVERAGE REPORT                        ║");
    println!("╠══════════════════════════════════════════════════════════════════════╣");
    println!("║ Overall: {}/{} ({:.1}%)                                           ║",
             implemented, total, (implemented as f64 / total as f64) * 100.0);
    println!("║ Critical: {}/{} ({:.1}%)                                          ║",
             critical_impl, critical_total, (critical_impl as f64 / critical_total as f64) * 100.0);
    println!("║ High Priority: {}/{} ({:.1}%)                                     ║",
             high_impl, high_total, (high_impl as f64 / high_total as f64) * 100.0);
    println!("╚══════════════════════════════════════════════════════════════════════╝\n");

    // Print by category
    for (category, cat_results) in by_category.iter() {
        let cat_impl = cat_results.iter().filter(|r| r.implemented).count();
        let cat_total = cat_results.len();

        println!("\n{:?} ({}/{})", category, cat_impl, cat_total);
        println!("{}", "─".repeat(50));

        for result in cat_results {
            let status = if result.implemented { "✓" } else { "✗" };
            let priority = match result.api.priority {
                Priority::Critical => "[CRIT]",
                Priority::High => "[HIGH]",
                Priority::Medium => "[MED ]",
                Priority::Low => "[LOW ]",
            };
            println!("  {} {} {} - {}", status, priority, result.api.name, result.api.description);
        }
    }
}
