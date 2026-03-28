//! Thalora Browser APIs
//!
//! Web standard APIs extracted from Boa engine fork for use in Thalora browser.

// Many functions in this crate are used as callbacks for Boa engine's builtin system
// (passed to BuiltInBuilder::callable()) but the compiler doesn't detect them as "used".
// Additionally, some stub implementations for WASM exist for API compatibility.
#![allow(dead_code)]

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

// Canvas API
pub mod canvas;

// Audio API
pub mod audio;

// Video API
pub mod video;

// WebGL API
pub mod webgl;

// Web Components API
pub mod web_components;

// Intl API (provided by Boa engine, tests only)
pub mod intl;

// Audit module for Web API coverage testing
pub mod audit;

/// Initialize all browser APIs in a Boa context
pub fn initialize_browser_apis(context: &mut boa_engine::Context) -> JsResult<()> {
    use boa_engine::builtins::{BuiltInObject, IntrinsicObject};
    use boa_engine::property::PropertyDescriptor;

    let realm = context.realm().clone();

    // Initialize Console API (must be early for debugging)
    console::console::Console::init(context);

    // Initialize Timer APIs (foundational for async operations)
    timers::timers::Timers::init(context);

    // Initialize Event APIs (foundational for DOM)
    events::event::Event::init(&realm);
    events::event_target::EventTarget::init(&realm);
    events::ui_events::UIEvent::init(&realm);
    events::ui_events::KeyboardEvent::init(&realm);
    events::ui_events::MouseEvent::init(&realm);
    events::ui_events::FocusEvent::init(&realm);
    events::ui_events::InputEvent::init(&realm);
    events::custom_event::CustomEvent::init(&realm);
    events::error_event::ErrorEvent::init(&realm);
    events::progress_event::ProgressEvent::init(&realm);
    events::hash_change_event::HashChangeEvent::init(&realm);
    events::pop_state_event::PopStateEvent::init(&realm);
    events::close_event::CloseEvent::init(&realm);
    events::message_event::MessageEvent::init(&realm);
    events::abort_signal::AbortSignal::init(&realm);

    // Initialize DOM APIs
    dom::node::Node::init(&realm);
    dom::attr::Attr::init(&realm);
    dom::nodelist::NodeList::init(&realm);
    dom::domtokenlist::DOMTokenList::init(&realm);
    dom::htmlcollection::HTMLCollection::init(&realm);
    dom::namednodemap::NamedNodeMap::init(&realm);
    dom::treewalker::TreeWalker::init(&realm);
    dom::nodeiterator::NodeIterator::init(&realm);
    dom::document::Document::init(&realm);
    dom::element::Element::init(&realm);
    dom::character_data::CharacterData::init(&realm);
    dom::text::Text::init(&realm);
    dom::document_fragment::DocumentFragment::init(&realm);
    dom::range::Range::init(&realm);
    dom::selection::Selection::init(&realm);
    dom::html_image_element::HTMLImageElement::init(&realm);
    dom::image_bitmap::ImageBitmap::init(&realm);
    dom::html_element::HTMLElement::init(&realm);
    dom::html_script_element::HTMLScriptElement::init(&realm);
    dom::dom_parser::DOMParser::init(&realm);

    // Initialize Canvas APIs
    canvas::path::Path2D::init(&realm);
    canvas::html_canvas_element::HTMLCanvasElement::init(&realm);
    canvas::rendering_context_2d::CanvasRenderingContext2D::init(&realm);
    canvas::offscreen_canvas::OffscreenCanvas::init(&realm);

    // Initialize Audio APIs
    audio::html_audio_element::HTMLAudioElement::init(&realm);
    audio::audio_context::AudioContext::init(&realm);

    // Initialize Video APIs
    video::html_video_element::HTMLVideoElement::init(&realm);

    // Initialize WebGL APIs
    webgl::context::WebGLRenderingContext::init(&realm);
    webgl::context2::WebGL2RenderingContext::init(&realm);

    // Initialize Browser APIs
    browser::navigator::Navigator::init(&realm);
    browser::window::Window::init(&realm);
    browser::history::History::init(&realm);
    browser::location::Location::init(&realm);
    browser::performance::Performance::init(&realm);

    // Initialize Storage APIs
    storage::storage::Storage::init(&realm);
    storage::storage_event::StorageEvent::init(&realm);
    storage::storage_manager::StorageManager::init(&realm);

    // Initialize Cache API
    storage::cache::Cache::init(&realm);
    storage::cache::CacheStorage::init(&realm);

    // Initialize IndexedDB APIs
    storage::indexed_db::key_range::IDBKeyRange::init(&realm);
    storage::indexed_db::request::IDBRequest::init(&realm);
    storage::indexed_db::factory::IDBFactory::init(&realm);
    storage::indexed_db::database::IDBDatabase::init(&realm);
    storage::indexed_db::transaction::IDBTransaction::init(&realm);
    storage::indexed_db::object_store::IDBObjectStore::init(&realm);
    storage::indexed_db::cursor::IDBCursor::init(&realm);
    storage::indexed_db::index::IDBIndex::init(&realm);

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

    // Initialize Observer APIs
    observers::intersection_observer::IntersectionObserver::init(&realm);
    observers::intersection_observer::IntersectionObserverEntry::init(&realm);
    observers::mutation_observer::MutationObserver::init(&realm);
    observers::mutation_observer::MutationRecord::init(&realm);
    observers::resize_observer::ResizeObserver::init(&realm);
    observers::performance_observer::PerformanceObserver::init(context);

    // Initialize Messaging APIs
    messaging::broadcast_channel::BroadcastChannel::init(&realm);
    messaging::message_channel::MessageChannel::init(&realm);
    messaging::message_port::MessagePort::init(&realm);

    // Initialize Miscellaneous APIs
    misc::abort_controller::AbortController::init(&realm);
    misc::encoding::TextEncoder::init(&realm);
    misc::encoding::TextDecoder::init(&realm);
    misc::url::Url::init(&realm);
    misc::url::UrlSearchParams::init(&realm);
    misc::css::Css::init(&realm);
    misc::form::HTMLFormElement::init(&realm);
    misc::form::HTMLInputElement::init(&realm);
    misc::form::HTMLSelectElement::init(&realm);
    misc::form::HTMLTextAreaElement::init(&realm);
    misc::form::HTMLOptionElement::init(&realm);
    misc::form::ValidityState::init(&realm);
    misc::form_data::FormData::init(&realm);

    // Initialize CSSOM APIs
    browser::cssom::CSSStyleDeclaration::init(&realm);

    // Initialize Streams APIs
    streams::readable_stream::ReadableStream::init(&realm);
    streams::writable_stream::WritableStream::init(&realm);
    streams::transform_stream::TransformStream::init(&realm);
    streams::queuing_strategy::CountQueuingStrategy::init(&realm);
    streams::queuing_strategy::ByteLengthQueuingStrategy::init(&realm);

    // Initialize Web Components APIs
    web_components::custom_element_registry::CustomElementRegistry::init(context);
    web_components::html_template_element::HTMLTemplateElement::init(context);

    // Initialize WebRTC APIs
    webrtc::rtc_peer_connection::RTCPeerConnectionBuiltin::init(&realm);

    // Initialize Worker APIs
    #[cfg(feature = "native")]
    worker::worker::WorkerConstructor::init(&realm);
    #[cfg(feature = "native")]
    worker::service_worker::ServiceWorker::init(&realm);
    #[cfg(feature = "native")]
    worker::service_worker_container::ServiceWorkerContainer::init(&realm);

    // Initialize Fetch APIs
    fetch::fetch::Fetch::init(&realm);
    fetch::fetch::Request::init(&realm);
    fetch::fetch::Response::init(&realm);
    fetch::fetch::Headers::init(&realm);
    fetch::xmlhttprequest::XmlHttpRequest::init(&realm);
    #[cfg(feature = "native")]
    fetch::websocket::WebSocket::init(&realm);

    // Register browser APIs as global properties
    let global_object = context.global_object();

    // Browser APIs - Register Navigator constructor (uppercase)
    global_object.define_property_or_throw(
        browser::navigator::Navigator::NAME,
        PropertyDescriptor::builder()
            .value(browser::navigator::Navigator::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Navigator instance is created later (after Window) with storage/locks set up.
    // The global `navigator` property is set at that point to ensure
    // `window.navigator === navigator` holds true.

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

    global_object.define_property_or_throw(
        events::custom_event::CustomEvent::NAME,
        PropertyDescriptor::builder()
            .value(events::custom_event::CustomEvent::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        events::error_event::ErrorEvent::NAME,
        PropertyDescriptor::builder()
            .value(events::error_event::ErrorEvent::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        events::message_event::MessageEvent::NAME,
        PropertyDescriptor::builder()
            .value(events::message_event::MessageEvent::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        events::abort_signal::AbortSignal::NAME,
        PropertyDescriptor::builder()
            .value(events::abort_signal::AbortSignal::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // UI Event APIs
    global_object.define_property_or_throw(
        events::ui_events::UIEvent::NAME,
        PropertyDescriptor::builder()
            .value(events::ui_events::UIEvent::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        events::ui_events::MouseEvent::NAME,
        PropertyDescriptor::builder()
            .value(events::ui_events::MouseEvent::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        events::ui_events::KeyboardEvent::NAME,
        PropertyDescriptor::builder()
            .value(events::ui_events::KeyboardEvent::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        events::ui_events::FocusEvent::NAME,
        PropertyDescriptor::builder()
            .value(events::ui_events::FocusEvent::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        events::ui_events::InputEvent::NAME,
        PropertyDescriptor::builder()
            .value(events::ui_events::InputEvent::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Progress/Navigation Events
    global_object.define_property_or_throw(
        events::progress_event::ProgressEvent::NAME,
        PropertyDescriptor::builder()
            .value(events::progress_event::ProgressEvent::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        events::hash_change_event::HashChangeEvent::NAME,
        PropertyDescriptor::builder()
            .value(events::hash_change_event::HashChangeEvent::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        events::pop_state_event::PopStateEvent::NAME,
        PropertyDescriptor::builder()
            .value(events::pop_state_event::PopStateEvent::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        events::close_event::CloseEvent::NAME,
        PropertyDescriptor::builder()
            .value(events::close_event::CloseEvent::get(context.intrinsics()))
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

    global_object.define_property_or_throw(
        fetch::xmlhttprequest::XmlHttpRequest::NAME,
        PropertyDescriptor::builder()
            .value(fetch::xmlhttprequest::XmlHttpRequest::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // WebSocket constructor
    #[cfg(feature = "native")]
    global_object.define_property_or_throw(
        fetch::websocket::WebSocket::NAME,
        PropertyDescriptor::builder()
            .value(fetch::websocket::WebSocket::get(context.intrinsics()))
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

    // Observer APIs
    global_object.define_property_or_throw(
        observers::intersection_observer::IntersectionObserver::NAME,
        PropertyDescriptor::builder()
            .value(observers::intersection_observer::IntersectionObserver::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        observers::mutation_observer::MutationObserver::NAME,
        PropertyDescriptor::builder()
            .value(observers::mutation_observer::MutationObserver::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        observers::resize_observer::ResizeObserver::NAME,
        PropertyDescriptor::builder()
            .value(observers::resize_observer::ResizeObserver::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // IntersectionObserverEntry constructor
    global_object.define_property_or_throw(
        observers::intersection_observer::IntersectionObserverEntry::NAME,
        PropertyDescriptor::builder()
            .value(
                observers::intersection_observer::IntersectionObserverEntry::get(
                    context.intrinsics(),
                ),
            )
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // MutationRecord constructor
    global_object.define_property_or_throw(
        observers::mutation_observer::MutationRecord::NAME,
        PropertyDescriptor::builder()
            .value(observers::mutation_observer::MutationRecord::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Messaging APIs
    global_object.define_property_or_throw(
        messaging::broadcast_channel::BroadcastChannel::NAME,
        PropertyDescriptor::builder()
            .value(messaging::broadcast_channel::BroadcastChannel::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        messaging::message_channel::MessageChannel::NAME,
        PropertyDescriptor::builder()
            .value(messaging::message_channel::MessageChannel::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        messaging::message_port::MessagePort::NAME,
        PropertyDescriptor::builder()
            .value(messaging::message_port::MessagePort::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Miscellaneous APIs
    global_object.define_property_or_throw(
        misc::abort_controller::AbortController::NAME,
        PropertyDescriptor::builder()
            .value(misc::abort_controller::AbortController::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        misc::encoding::TextEncoder::NAME,
        PropertyDescriptor::builder()
            .value(misc::encoding::TextEncoder::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        misc::encoding::TextDecoder::NAME,
        PropertyDescriptor::builder()
            .value(misc::encoding::TextDecoder::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Global atob/btoa functions
    let atob_fn = misc::encoding::create_atob_function(context)?;
    global_object.set(js_string!("atob"), atob_fn, false, context)?;

    let btoa_fn = misc::encoding::create_btoa_function(context)?;
    global_object.set(js_string!("btoa"), btoa_fn, false, context)?;

    global_object.define_property_or_throw(
        misc::url::Url::NAME,
        PropertyDescriptor::builder()
            .value(misc::url::Url::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        misc::url::UrlSearchParams::NAME,
        PropertyDescriptor::builder()
            .value(misc::url::UrlSearchParams::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        misc::css::Css::NAME,
        PropertyDescriptor::builder()
            .value(misc::css::Css::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        misc::form::HTMLFormElement::NAME,
        PropertyDescriptor::builder()
            .value(misc::form::HTMLFormElement::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        misc::form::HTMLInputElement::NAME,
        PropertyDescriptor::builder()
            .value(misc::form::HTMLInputElement::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Streams APIs
    global_object.define_property_or_throw(
        streams::readable_stream::ReadableStream::NAME,
        PropertyDescriptor::builder()
            .value(streams::readable_stream::ReadableStream::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        streams::writable_stream::WritableStream::NAME,
        PropertyDescriptor::builder()
            .value(streams::writable_stream::WritableStream::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        streams::transform_stream::TransformStream::NAME,
        PropertyDescriptor::builder()
            .value(streams::transform_stream::TransformStream::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        streams::queuing_strategy::CountQueuingStrategy::NAME,
        PropertyDescriptor::builder()
            .value(streams::queuing_strategy::CountQueuingStrategy::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        streams::queuing_strategy::ByteLengthQueuingStrategy::NAME,
        PropertyDescriptor::builder()
            .value(streams::queuing_strategy::ByteLengthQueuingStrategy::get(
                context.intrinsics(),
            ))
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

    global_object.define_property_or_throw(
        dom::character_data::CharacterData::NAME,
        PropertyDescriptor::builder()
            .value(dom::character_data::CharacterData::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        dom::text::Text::NAME,
        PropertyDescriptor::builder()
            .value(dom::text::Text::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        dom::document_fragment::DocumentFragment::NAME,
        PropertyDescriptor::builder()
            .value(dom::document_fragment::DocumentFragment::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        dom::range::Range::NAME,
        PropertyDescriptor::builder()
            .value(dom::range::Range::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // NamedNodeMap constructor
    global_object.define_property_or_throw(
        dom::namednodemap::NamedNodeMap::NAME,
        PropertyDescriptor::builder()
            .value(dom::namednodemap::NamedNodeMap::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // TreeWalker constructor
    global_object.define_property_or_throw(
        dom::treewalker::TreeWalker::NAME,
        PropertyDescriptor::builder()
            .value(dom::treewalker::TreeWalker::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // NodeIterator constructor
    global_object.define_property_or_throw(
        dom::nodeiterator::NodeIterator::NAME,
        PropertyDescriptor::builder()
            .value(dom::nodeiterator::NodeIterator::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Selection constructor
    global_object.define_property_or_throw(
        dom::selection::Selection::NAME,
        PropertyDescriptor::builder()
            .value(dom::selection::Selection::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // HTMLCollection constructor
    global_object.define_property_or_throw(
        dom::htmlcollection::HTMLCollection::NAME,
        PropertyDescriptor::builder()
            .value(dom::htmlcollection::HTMLCollection::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // HTMLImageElement - real image element with image loading and decoding
    global_object.define_property_or_throw(
        dom::html_image_element::HTMLImageElement::NAME,
        PropertyDescriptor::builder()
            .value(dom::html_image_element::HTMLImageElement::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Also expose as "Image" constructor for compatibility with `new Image()`
    global_object.define_property_or_throw(
        js_string!("Image"),
        PropertyDescriptor::builder()
            .value(dom::html_image_element::HTMLImageElement::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // ImageBitmap constructor
    global_object.define_property_or_throw(
        dom::image_bitmap::ImageBitmap::NAME,
        PropertyDescriptor::builder()
            .value(dom::image_bitmap::ImageBitmap::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // createImageBitmap global function
    let create_image_bitmap_fn = boa_engine::object::FunctionObjectBuilder::new(
        context.realm(),
        boa_engine::NativeFunction::from_fn_ptr(dom::image_bitmap::create_image_bitmap),
    )
    .name("createImageBitmap")
    .length(1)
    .build();
    global_object.set(
        js_string!("createImageBitmap"),
        create_image_bitmap_fn,
        false,
        context,
    )?;

    // HTMLElement constructor
    global_object.define_property_or_throw(
        dom::html_element::HTMLElement::NAME,
        PropertyDescriptor::builder()
            .value(dom::html_element::HTMLElement::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // HTMLScriptElement constructor
    global_object.define_property_or_throw(
        dom::html_script_element::HTMLScriptElement::NAME,
        PropertyDescriptor::builder()
            .value(dom::html_script_element::HTMLScriptElement::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // DOMParser constructor
    global_object.define_property_or_throw(
        dom::dom_parser::DOMParser::NAME,
        PropertyDescriptor::builder()
            .value(dom::dom_parser::DOMParser::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // FormData constructor
    global_object.define_property_or_throw(
        misc::form_data::FormData::NAME,
        PropertyDescriptor::builder()
            .value(misc::form_data::FormData::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // CSSStyleDeclaration constructor
    global_object.define_property_or_throw(
        browser::cssom::CSSStyleDeclaration::NAME,
        PropertyDescriptor::builder()
            .value(browser::cssom::CSSStyleDeclaration::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // getComputedStyle global function
    let get_computed_style_fn = browser::cssom::create_get_computed_style_function(context)?;
    global_object.set(
        js_string!("getComputedStyle"),
        get_computed_style_fn,
        false,
        context,
    )?;

    // Canvas APIs
    global_object.define_property_or_throw(
        canvas::path::Path2D::NAME,
        PropertyDescriptor::builder()
            .value(canvas::path::Path2D::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        canvas::html_canvas_element::HTMLCanvasElement::NAME,
        PropertyDescriptor::builder()
            .value(canvas::html_canvas_element::HTMLCanvasElement::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        canvas::rendering_context_2d::CanvasRenderingContext2D::NAME,
        PropertyDescriptor::builder()
            .value(canvas::rendering_context_2d::CanvasRenderingContext2D::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        canvas::offscreen_canvas::OffscreenCanvas::NAME,
        PropertyDescriptor::builder()
            .value(canvas::offscreen_canvas::OffscreenCanvas::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Audio APIs
    global_object.define_property_or_throw(
        audio::html_audio_element::HTMLAudioElement::NAME,
        PropertyDescriptor::builder()
            .value(audio::html_audio_element::HTMLAudioElement::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Also expose as "Audio" constructor for compatibility with `new Audio()`
    global_object.define_property_or_throw(
        js_string!("Audio"),
        PropertyDescriptor::builder()
            .value(audio::html_audio_element::HTMLAudioElement::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // AudioContext - Web Audio API
    global_object.define_property_or_throw(
        audio::audio_context::AudioContext::NAME,
        PropertyDescriptor::builder()
            .value(audio::audio_context::AudioContext::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Video APIs
    global_object.define_property_or_throw(
        video::html_video_element::HTMLVideoElement::NAME,
        PropertyDescriptor::builder()
            .value(video::html_video_element::HTMLVideoElement::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Worker APIs (native only)
    #[cfg(feature = "native")]
    global_object.define_property_or_throw(
        worker::worker::WorkerConstructor::NAME,
        PropertyDescriptor::builder()
            .value(worker::worker::WorkerConstructor::get(context.intrinsics()))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    #[cfg(feature = "native")]
    global_object.define_property_or_throw(
        worker::service_worker::ServiceWorker::NAME,
        PropertyDescriptor::builder()
            .value(worker::service_worker::ServiceWorker::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Browser APIs - Navigator is already registered above at lines 220-229
    // Removed duplicate Navigator registration to fix assertion error

    // Notification constructor (Web Notifications API)
    let notification_constructor = browser::notification::create_notification_constructor(context)?;
    global_object.define_property_or_throw(
        js_string!("Notification"),
        PropertyDescriptor::builder()
            .value(notification_constructor)
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // ClipboardItem constructor (Async Clipboard API)
    let clipboard_item_constructor =
        browser::clipboard::create_clipboard_item_constructor(context)?;
    global_object.define_property_or_throw(
        js_string!("ClipboardItem"),
        PropertyDescriptor::builder()
            .value(clipboard_item_constructor)
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // WebGL constructors (with static constants per Web spec)
    let webgl_constructor = webgl::WebGLRenderingContext::create_global_constructor(context)?;
    global_object.define_property_or_throw(
        js_string!("WebGLRenderingContext"),
        PropertyDescriptor::builder()
            .value(webgl_constructor)
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    let webgl2_constructor = webgl::WebGL2RenderingContext::create_global_constructor(context)?;
    global_object.define_property_or_throw(
        js_string!("WebGL2RenderingContext"),
        PropertyDescriptor::builder()
            .value(webgl2_constructor)
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
            .value(storage::storage_event::StorageEvent::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    global_object.define_property_or_throw(
        storage::storage_manager::StorageManager::NAME,
        PropertyDescriptor::builder()
            .value(storage::storage_manager::StorageManager::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Register IDBFactory constructor
    global_object.define_property_or_throw(
        storage::indexed_db::factory::IDBFactory::NAME,
        PropertyDescriptor::builder()
            .value(storage::indexed_db::factory::IDBFactory::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Register IDBKeyRange constructor
    global_object.define_property_or_throw(
        storage::indexed_db::key_range::IDBKeyRange::NAME,
        PropertyDescriptor::builder()
            .value(storage::indexed_db::key_range::IDBKeyRange::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Register IDBDatabase constructor
    global_object.define_property_or_throw(
        storage::indexed_db::database::IDBDatabase::NAME,
        PropertyDescriptor::builder()
            .value(storage::indexed_db::database::IDBDatabase::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Register IDBTransaction constructor
    global_object.define_property_or_throw(
        storage::indexed_db::transaction::IDBTransaction::NAME,
        PropertyDescriptor::builder()
            .value(storage::indexed_db::transaction::IDBTransaction::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Register IDBObjectStore constructor
    global_object.define_property_or_throw(
        storage::indexed_db::object_store::IDBObjectStore::NAME,
        PropertyDescriptor::builder()
            .value(
                <storage::indexed_db::object_store::IDBObjectStore as IntrinsicObject>::get(
                    context.intrinsics(),
                ),
            )
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Register IDBCursor constructor
    global_object.define_property_or_throw(
        storage::indexed_db::cursor::IDBCursor::NAME,
        PropertyDescriptor::builder()
            .value(storage::indexed_db::cursor::IDBCursor::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Register IDBIndex constructor
    global_object.define_property_or_throw(
        storage::indexed_db::index::IDBIndex::NAME,
        PropertyDescriptor::builder()
            .value(
                <storage::indexed_db::index::IDBIndex as IntrinsicObject>::get(
                    context.intrinsics(),
                ),
            )
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Register IDBRequest constructor
    global_object.define_property_or_throw(
        storage::indexed_db::request::IDBRequest::NAME,
        PropertyDescriptor::builder()
            .value(storage::indexed_db::request::IDBRequest::get(
                context.intrinsics(),
            ))
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
    )
    .expect("failed to create document instance");

    global_object
        .set(
            boa_engine::js_string!("document"),
            document_instance,
            false,
            context,
        )
        .expect("failed to set global document");

    // Create a global window instance
    let window_constructor = browser::window::Window::get(context.intrinsics());
    let window_args = [];
    let window_instance = browser::window::Window::constructor(
        &window_constructor.clone().into(),
        &window_args,
        context,
    )
    .expect("failed to create window instance");

    global_object
        .set(
            boa_engine::js_string!("window"),
            window_instance.clone(),
            false,
            context,
        )
        .expect("failed to set global window");

    // Create navigator instance manually with proper prototype and data
    let navigator_proto = global_object
        .get(browser::navigator::Navigator::NAME, context)?
        .as_object()
        .and_then(|ctor| ctor.get(boa_engine::js_string!("prototype"), context).ok())
        .and_then(|proto| proto.as_object().map(|obj| obj.clone()))
        .ok_or_else(|| {
            boa_engine::JsNativeError::typ().with_message("Navigator prototype not found")
        })?;

    let mut navigator_data = browser::navigator::Navigator::new();

    // Create StorageManager instance for navigator.storage
    let storage_manager_proto = global_object
        .get(storage::storage_manager::StorageManager::NAME, context)?
        .as_object()
        .and_then(|ctor| ctor.get(boa_engine::js_string!("prototype"), context).ok())
        .and_then(|proto| proto.as_object().map(|obj| obj.clone()))
        .ok_or_else(|| {
            boa_engine::JsNativeError::typ().with_message("StorageManager prototype not found")
        })?;

    let storage_manager_data = storage::storage_manager::StorageManager::new();
    let storage_manager_obj = boa_engine::JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        storage_manager_proto,
        storage_manager_data,
    );

    // Create LockManager instance for navigator.locks
    let lock_manager_proto = global_object
        .get(locks::lock_manager::LockManager::NAME, context)?
        .as_object()
        .and_then(|ctor| ctor.get(boa_engine::js_string!("prototype"), context).ok())
        .and_then(|proto| proto.as_object().map(|obj| obj.clone()))
        .ok_or_else(|| {
            boa_engine::JsNativeError::typ().with_message("LockManager prototype not found")
        })?;

    let lock_manager_data = locks::lock_manager::LockManager::new();
    let lock_manager_obj = boa_engine::JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        lock_manager_proto,
        lock_manager_data,
    );

    // Set lock_manager on navigator_data BEFORE creating the object
    eprintln!("DEBUG: Setting lock_manager on Navigator data structure");
    let lock_manager_generic = lock_manager_obj.upcast();
    navigator_data.set_lock_manager(lock_manager_generic.clone());

    // Now create the navigator object with the data that includes lock_manager
    let navigator_obj = boa_engine::JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        navigator_proto,
        navigator_data,
    );

    // Set navigator.storage (this works with .set() because storage doesn't have a special getter)
    let navigator_generic = navigator_obj.upcast();
    navigator_generic
        .set(
            boa_engine::js_string!("storage"),
            storage_manager_obj,
            false,
            context,
        )
        .expect("failed to set navigator.storage");

    let check_storage = navigator_generic
        .get(boa_engine::js_string!("storage"), context)
        .unwrap();
    eprintln!(
        "DEBUG: After setting navigator.storage, get('storage') = {:?}",
        check_storage
    );

    // Set navigator.locks as a value property (not accessor) for proper descriptor
    navigator_generic
        .define_property_or_throw(
            boa_engine::js_string!("locks"),
            boa_engine::property::PropertyDescriptor::builder()
                .value(lock_manager_generic)
                .writable(false)
                .enumerable(true)
                .configurable(true)
                .build(),
            context,
        )
        .expect("failed to set navigator.locks");

    let navigator_instance: boa_engine::JsValue = navigator_generic.into();

    // Now set navigator on the window object (after setting storage and locks)
    if let Some(window_obj) = window_instance.as_object() {
        if let Some(window_data) = window_obj.downcast_ref::<browser::window::WindowData>() {
            if let Some(nav_obj) = navigator_instance.as_object() {
                window_data.set_navigator(nav_obj.clone());
            }
        }
    }

    // Set as global navigator (same object as window.navigator)
    global_object
        .define_property_or_throw(
            boa_engine::js_string!("navigator"),
            boa_engine::property::PropertyDescriptor::builder()
                .value(navigator_instance.clone())
                .writable(false)
                .enumerable(true)
                .configurable(true),
            context,
        )
        .expect("failed to set global navigator");

    // Per Geolocation API spec, navigator.geolocation must always exist.
    // Set up via JS evaluation - provides standard interface with mock coordinates.
    {
        use boa_engine::Source;
        context
            .eval(Source::from_bytes(
                r#"
            (function() {
                var _watchId = 0;
                var _mockPosition = {
                    coords: {
                        latitude: 37.7749,
                        longitude: -122.4194,
                        accuracy: 100.0,
                        altitude: null,
                        altitudeAccuracy: null,
                        heading: null,
                        speed: null
                    },
                    timestamp: Date.now()
                };
                navigator.geolocation = {
                    getCurrentPosition: function(successCallback, errorCallback, options) {
                        if (typeof successCallback !== 'function') {
                            throw new TypeError('getCurrentPosition requires a success callback');
                        }
                        _mockPosition.timestamp = Date.now();
                        successCallback(_mockPosition);
                    },
                    watchPosition: function(successCallback, errorCallback, options) {
                        if (typeof successCallback !== 'function') {
                            throw new TypeError('watchPosition requires a success callback');
                        }
                        _watchId++;
                        _mockPosition.timestamp = Date.now();
                        successCallback(_mockPosition);
                        return _watchId;
                    },
                    clearWatch: function(id) { /* no-op */ }
                };
            })();
        "#,
            ))
            .map_err(|e| {
                boa_engine::JsNativeError::typ()
                    .with_message(format!("Failed to initialize geolocation: {}", e))
            })?;
    }

    // Create and set global location object
    let location_constructor = context.intrinsics().constructors().location().constructor();
    let location_instance = browser::location::Location::constructor(
        &location_constructor.clone().into(),
        &[],
        context,
    )?;
    global_object
        .set(
            boa_engine::js_string!("location"),
            location_instance,
            false,
            context,
        )
        .expect("failed to set global location");

    // Create and set global history object
    let history_constructor = context.intrinsics().constructors().history().constructor();
    let history_instance =
        browser::history::History::constructor(&history_constructor.clone().into(), &[], context)?;
    global_object
        .set(
            boa_engine::js_string!("history"),
            history_instance,
            false,
            context,
        )
        .expect("failed to set global history");

    // Create and set global performance object
    let performance_instance = browser::performance::create_performance_object(context)?;
    global_object
        .set(
            boa_engine::js_string!("performance"),
            performance_instance,
            false,
            context,
        )
        .expect("failed to set global performance");

    // Add EventTarget functionality to globalThis for WorkerGlobalScope compatibility
    // Store the global event target as a hidden non-enumerable property on globalThis
    let global_event_target = events::event_target::EventTarget::create(context)?;
    let global_event_target_value: boa_engine::JsValue = global_event_target.clone().into();
    global_object.insert_property(
        boa_engine::js_string!("__globalEventTarget__"),
        boa_engine::property::PropertyDescriptor::builder()
            .value(global_event_target_value)
            .writable(false)
            .enumerable(false)
            .configurable(false)
            .build(),
    );

    // Create wrapper functions that retrieve the hidden event target and delegate to it
    let add_listener_fn = boa_engine::builtins::BuiltInBuilder::callable(
        context.realm(),
        |_this: &boa_engine::JsValue,
         args: &[boa_engine::JsValue],
         context: &mut boa_engine::Context| {
            let global = context.global_object();
            let event_target =
                global.get(boa_engine::js_string!("__globalEventTarget__"), context)?;
            events::event_target::EventTarget::add_event_listener(&event_target, args, context)
        },
    )
    .name(boa_engine::js_string!("addEventListener"))
    .length(2)
    .build();
    global_object
        .set(
            boa_engine::js_string!("addEventListener"),
            add_listener_fn,
            false,
            context,
        )
        .expect("failed to set global addEventListener");

    let remove_listener_fn = boa_engine::builtins::BuiltInBuilder::callable(
        context.realm(),
        |_this: &boa_engine::JsValue,
         args: &[boa_engine::JsValue],
         context: &mut boa_engine::Context| {
            let global = context.global_object();
            let event_target =
                global.get(boa_engine::js_string!("__globalEventTarget__"), context)?;
            events::event_target::EventTarget::remove_event_listener(&event_target, args, context)
        },
    )
    .name(boa_engine::js_string!("removeEventListener"))
    .length(2)
    .build();
    global_object
        .set(
            boa_engine::js_string!("removeEventListener"),
            remove_listener_fn,
            false,
            context,
        )
        .expect("failed to set global removeEventListener");

    let dispatch_fn = boa_engine::builtins::BuiltInBuilder::callable(
        context.realm(),
        |_this: &boa_engine::JsValue,
         args: &[boa_engine::JsValue],
         context: &mut boa_engine::Context| {
            let global = context.global_object();
            let event_target =
                global.get(boa_engine::js_string!("__globalEventTarget__"), context)?;
            events::event_target::EventTarget::dispatch_event(&event_target, args, context)
        },
    )
    .name(boa_engine::js_string!("dispatchEvent"))
    .length(1)
    .build();
    global_object
        .set(
            boa_engine::js_string!("dispatchEvent"),
            dispatch_fn,
            false,
            context,
        )
        .expect("failed to set global dispatchEvent");

    // postMessage global function (for window.postMessage behavior)
    let post_message_fn = boa_engine::builtins::BuiltInBuilder::callable(
        context.realm(),
        |_this: &boa_engine::JsValue,
         args: &[boa_engine::JsValue],
         context: &mut boa_engine::Context| {
            // Create a MessageEvent with the data and dispatch it
            let message = args
                .get(0)
                .cloned()
                .unwrap_or(boa_engine::JsValue::undefined());
            let _origin = args.get(1).cloned().unwrap_or(js_string!("*").into());

            // For now, just dispatch a message event to self (in a full implementation,
            // this would be used for cross-origin messaging)
            let global = context.global_object();
            let event_target =
                global.get(boa_engine::js_string!("__globalEventTarget__"), context)?;

            // Create a MessageEvent-like object
            let event_obj = boa_engine::JsObject::with_null_proto();
            event_obj.set(js_string!("type"), js_string!("message"), false, context)?;
            event_obj.set(js_string!("data"), message, false, context)?;

            // Dispatch the event
            events::event_target::EventTarget::dispatch_event(
                &event_target,
                &[event_obj.into()],
                context,
            )
        },
    )
    .name(boa_engine::js_string!("postMessage"))
    .length(1)
    .build();
    global_object
        .set(
            boa_engine::js_string!("postMessage"),
            post_message_fn,
            false,
            context,
        )
        .expect("failed to set global postMessage");

    // Create localStorage and sessionStorage instances
    // Storage constructor cannot be called directly, so we create instances manually
    let storage_proto = global_object
        .get(storage::storage::Storage::NAME, context)?
        .as_object()
        .and_then(|ctor| ctor.get(boa_engine::js_string!("prototype"), context).ok())
        .and_then(|proto| proto.as_object().map(|obj| obj.clone()))
        .ok_or_else(|| {
            boa_engine::JsNativeError::typ().with_message("Storage prototype not found")
        })?;

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

    // Create indexedDB instance (same pattern as Storage)
    let idb_factory_proto = global_object
        .get(storage::indexed_db::factory::IDBFactory::NAME, context)?
        .as_object()
        .and_then(|ctor| ctor.get(boa_engine::js_string!("prototype"), context).ok())
        .and_then(|proto| proto.as_object().map(|obj| obj.clone()))
        .ok_or_else(|| {
            boa_engine::JsNativeError::typ().with_message("IDBFactory prototype not found")
        })?;

    let indexed_db_factory_data = storage::indexed_db::factory::IDBFactory::new().map_err(|e| {
        boa_engine::JsNativeError::error()
            .with_message(format!("Failed to create IndexedDB factory: {}", e))
    })?;

    let indexed_db_obj = boa_engine::JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        idb_factory_proto,
        indexed_db_factory_data,
    );
    let indexed_db: boa_engine::JsValue = indexed_db_obj.into();

    // Set on window object (must be done BEFORE window is finalized)
    if let Some(window_obj) = window_instance.as_object() {
        window_obj
            .set(
                boa_engine::js_string!("localStorage"),
                local_storage.clone(),
                false,
                context,
            )
            .expect("failed to set window.localStorage");
        window_obj
            .set(
                boa_engine::js_string!("sessionStorage"),
                session_storage.clone(),
                false,
                context,
            )
            .expect("failed to set window.sessionStorage");
        window_obj
            .set(
                boa_engine::js_string!("indexedDB"),
                indexed_db.clone(),
                false,
                context,
            )
            .expect("failed to set window.indexedDB");
    }

    // Set as globals
    global_object
        .set(
            boa_engine::js_string!("localStorage"),
            local_storage,
            false,
            context,
        )
        .expect("failed to set global localStorage");
    global_object
        .set(
            boa_engine::js_string!("sessionStorage"),
            session_storage,
            false,
            context,
        )
        .expect("failed to set global sessionStorage");
    global_object
        .set(
            boa_engine::js_string!("indexedDB"),
            indexed_db,
            false,
            context,
        )
        .expect("failed to set global indexedDB");

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

    // Register XMLHttpRequest as global
    global_object.define_property_or_throw(
        fetch::xmlhttprequest::XmlHttpRequest::NAME,
        PropertyDescriptor::builder()
            .value(fetch::xmlhttprequest::XmlHttpRequest::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Register RTCPeerConnection as global
    global_object.define_property_or_throw(
        webrtc::rtc_peer_connection::RTCPeerConnectionBuiltin::NAME,
        PropertyDescriptor::builder()
            .value(webrtc::rtc_peer_connection::RTCPeerConnectionBuiltin::get(
                context.intrinsics(),
            ))
            .writable(true)
            .enumerable(false)
            .configurable(true),
        context,
    )?;

    // Register WebAssembly, MediaRecorder, speechSynthesis, SpeechRecognition stubs
    // Per Web standards, these should exist as globals even if the underlying platform
    // features are not available (headless browser without hardware support).
    use boa_engine::Source;
    context.eval(Source::from_bytes(r#"
        (function() {
            // WebAssembly stub - per WebAssembly JS API spec
            if (typeof WebAssembly === "undefined") {
                var WebAssembly = {
                    compile: function(bytes) { return Promise.reject(new Error("WebAssembly not supported in this environment")); },
                    instantiate: function(bytes, imports) { return Promise.reject(new Error("WebAssembly not supported in this environment")); },
                    validate: function(bytes) { return false; },
                    Module: function() { throw new Error("WebAssembly.Module not supported"); },
                    Instance: function() { throw new Error("WebAssembly.Instance not supported"); },
                    Memory: function() { throw new Error("WebAssembly.Memory not supported"); },
                    Table: function() { throw new Error("WebAssembly.Table not supported"); },
                    CompileError: function(msg) { this.message = msg; this.name = "CompileError"; },
                    LinkError: function(msg) { this.message = msg; this.name = "LinkError"; },
                    RuntimeError: function(msg) { this.message = msg; this.name = "RuntimeError"; }
                };
                globalThis.WebAssembly = WebAssembly;
            }

            // MediaRecorder stub - per MediaStream Recording API spec
            if (typeof MediaRecorder === "undefined") {
                globalThis.MediaRecorder = function MediaRecorder(stream, options) {
                    this.stream = stream;
                    this.state = "inactive";
                    this.mimeType = (options && options.mimeType) || "";
                    this.ondataavailable = null;
                    this.onerror = null;
                    this.onstart = null;
                    this.onstop = null;
                    this.onpause = null;
                    this.onresume = null;
                };
                MediaRecorder.prototype.start = function() { this.state = "recording"; };
                MediaRecorder.prototype.stop = function() { this.state = "inactive"; };
                MediaRecorder.prototype.pause = function() { this.state = "paused"; };
                MediaRecorder.prototype.resume = function() { this.state = "recording"; };
                MediaRecorder.isTypeSupported = function(type) { return false; };
            }

            // SpeechSynthesis stub - per Web Speech API spec
            if (typeof speechSynthesis === "undefined") {
                globalThis.speechSynthesis = {
                    speaking: false,
                    pending: false,
                    paused: false,
                    onvoiceschanged: null,
                    speak: function(utterance) {},
                    cancel: function() {},
                    pause: function() {},
                    resume: function() {},
                    getVoices: function() { return []; }
                };
            }

            // SpeechRecognition stub - per Web Speech API spec
            if (typeof SpeechRecognition === "undefined") {
                globalThis.SpeechRecognition = function SpeechRecognition() {
                    this.continuous = false;
                    this.interimResults = false;
                    this.lang = "";
                    this.maxAlternatives = 1;
                    this.onresult = null;
                    this.onerror = null;
                    this.onstart = null;
                    this.onend = null;
                };
                SpeechRecognition.prototype.start = function() {};
                SpeechRecognition.prototype.stop = function() {};
                SpeechRecognition.prototype.abort = function() {};
            }

            // WebMIDI stub - per Web MIDI API spec (Chrome 124+)
            if (typeof navigator !== "undefined" && typeof navigator.requestMIDIAccess !== "function") {
                navigator.requestMIDIAccess = function(options) {
                    return Promise.resolve({
                        inputs: new Map(),
                        outputs: new Map(),
                        onstatechange: null,
                        sysexEnabled: !!(options && options.sysex)
                    });
                };
            }

            // DOM HTML unsafe methods - per HTML Sanitizer API (Chrome 124+)
            if (typeof Element !== "undefined" && typeof Element.prototype.setHTMLUnsafe !== "function") {
                Element.prototype.setHTMLUnsafe = function(html) {
                    this.innerHTML = html;
                };
            }
            if (typeof Document !== "undefined" && typeof Document.parseHTMLUnsafe !== "function") {
                Document.parseHTMLUnsafe = function(html) {
                    var parser = new DOMParser();
                    return parser.parseFromString(html, "text/html");
                };
            }

            // ReadableStream async iteration - per Streams API (Chrome 124+)
            if (typeof ReadableStream !== "undefined" && typeof Symbol !== "undefined" && typeof Symbol.asyncIterator !== "undefined") {
                if (typeof ReadableStream.prototype[Symbol.asyncIterator] !== "function") {
                    ReadableStream.prototype[Symbol.asyncIterator] = function() {
                        var reader = this.getReader();
                        return {
                            next: function() {
                                return reader.read().then(function(result) {
                                    if (result.done) {
                                        reader.releaseLock();
                                    }
                                    return result;
                                });
                            },
                            return: function() {
                                reader.releaseLock();
                                return Promise.resolve({ done: true, value: undefined });
                            }
                        };
                    };
                }
            }
        })();
    "#)).map_err(|e| {
        boa_engine::JsNativeError::typ()
            .with_message(format!("Failed to initialize browser API stubs: {}", e))
    })?;

    // Add 'self' reference to global scope (Worker/browser compatibility)
    global_object.set(js_string!("self"), global_object.clone(), false, context)?;

    Ok(())
}

// Re-export commonly used Boa types for tests
pub use boa_engine::{Context, JsNativeError, JsResult, JsString, JsValue, Source, js_string};

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
        pub fn assert_native_error(
            code: &'static str,
            kind: JsNativeErrorKind,
            message: &'static str,
        ) -> Self {
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
        crate::initialize_browser_apis(&mut context)
            .expect("Failed to initialize browser APIs in test context");

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
    use crate::{Context, JsString, JsValue, Source};

    #[test]
    fn debug_locks() {
        let mut context = Context::default();
        crate::initialize_browser_apis(&mut context).unwrap();

        println!("=== Testing LockManager ===");

        // 1. Does LockManager constructor exist?
        let result = context
            .eval(Source::from_bytes("typeof LockManager"))
            .unwrap();
        println!("1. typeof LockManager = {:?}", result);

        // 2. Does LockManager.prototype.request exist?
        let result = context
            .eval(Source::from_bytes("typeof LockManager.prototype.request"))
            .unwrap();
        println!("2. typeof LockManager.prototype.request = {:?}", result);

        // 3. Does global navigator exist?
        let result = context
            .eval(Source::from_bytes("typeof navigator"))
            .unwrap();
        println!("3. typeof navigator = {:?}", result);

        // 4. Does navigator.locks exist?
        let result = context
            .eval(Source::from_bytes("typeof navigator.locks"))
            .unwrap();
        println!("4. typeof navigator.locks = {:?}", result);

        // 5. Are navigator and window.navigator the same object?
        let result = context
            .eval(Source::from_bytes("navigator === window.navigator"))
            .unwrap();
        println!("5. navigator === window.navigator: {:?}", result);

        // 6. Does window.navigator have locks?
        let result = context
            .eval(Source::from_bytes("typeof window.navigator.locks"))
            .unwrap();
        println!("6. typeof window.navigator.locks = {:?}", result);

        // 7. Check what properties navigator has
        let result = context
            .eval(Source::from_bytes("Object.keys(navigator)"))
            .unwrap();
        println!("7. Object.keys(navigator) = {:?}", result);

        // 8. Does navigator.storage work?
        let result = context
            .eval(Source::from_bytes("typeof navigator.storage"))
            .unwrap();
        println!("8. typeof navigator.storage = {:?}", result);

        // 9. Does window.navigator.storage work?
        let result = context
            .eval(Source::from_bytes("typeof window.navigator.storage"))
            .unwrap();
        println!("9. typeof window.navigator.storage = {:?}", result);
    }
}
