//! HTMLVideoElement Web API implementation
//!
//! Provides video playback API surface for web compatibility.
//! https://html.spec.whatwg.org/multipage/media.html#htmlvideoelement

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};
use std::thread;

/// Video state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VideoState {
    /// Video is empty (no source)
    Empty,
    /// Video metadata is being loaded
    Loading,
    /// Video is ready to play
    Ready,
    /// Video is playing
    Playing,
    /// Video is paused
    Paused,
    /// Video playback ended
    Ended,
    /// Video had an error
    Error,
}

/// Network state constants
pub mod network_state {
    pub const NETWORK_EMPTY: u16 = 0;
    pub const NETWORK_IDLE: u16 = 1;
    pub const NETWORK_LOADING: u16 = 2;
    pub const NETWORK_NO_SOURCE: u16 = 3;
}

/// Ready state constants
pub mod ready_state {
    pub const HAVE_NOTHING: u16 = 0;
    pub const HAVE_METADATA: u16 = 1;
    pub const HAVE_CURRENT_DATA: u16 = 2;
    pub const HAVE_FUTURE_DATA: u16 = 3;
    pub const HAVE_ENOUGH_DATA: u16 = 4;
}

/// Internal video player state
#[derive(Debug)]
pub struct VideoPlayerState {
    /// Source URL
    src: String,
    /// Current state
    state: VideoState,
    /// Volume (0.0 to 1.0)
    volume: f32,
    /// Muted flag
    muted: bool,
    /// Loop flag
    loop_video: bool,
    /// Autoplay flag
    autoplay: bool,
    /// Controls flag
    controls: bool,
    /// Poster image URL
    poster: String,
    /// Current time in seconds
    current_time: f64,
    /// Duration in seconds
    duration: f64,
    /// Playback rate
    playback_rate: f64,
    /// Video width in pixels
    video_width: u32,
    /// Video height in pixels
    video_height: u32,
    /// Is video ready to play
    can_play: bool,
    /// Error message if any
    error: Option<String>,
    /// Network state
    network_state: u16,
    /// Ready state
    ready_state: u16,
    /// Preload setting
    preload: String,
    /// Cross-origin setting
    cross_origin: Option<String>,
    /// Current source MIME type
    current_type: String,
    /// Buffered time ranges (simplified as single range)
    buffered_start: f64,
    buffered_end: f64,
    /// Seeking flag
    seeking: bool,
}

impl VideoPlayerState {
    pub fn new() -> Self {
        Self {
            src: String::new(),
            state: VideoState::Empty,
            volume: 1.0,
            muted: false,
            loop_video: false,
            autoplay: false,
            controls: false,
            poster: String::new(),
            current_time: 0.0,
            duration: 0.0,
            playback_rate: 1.0,
            video_width: 0,
            video_height: 0,
            can_play: false,
            error: None,
            network_state: network_state::NETWORK_EMPTY,
            ready_state: ready_state::HAVE_NOTHING,
            preload: String::from("auto"),
            cross_origin: None,
            current_type: String::new(),
            buffered_start: 0.0,
            buffered_end: 0.0,
            seeking: false,
        }
    }
}

/// Internal data for HTMLVideoElement
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct HTMLVideoElementData {
    #[unsafe_ignore_trace]
    state: Arc<Mutex<VideoPlayerState>>,
}

impl HTMLVideoElementData {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(VideoPlayerState::new())),
        }
    }

    pub fn get_src(&self) -> String {
        self.state.lock().unwrap().src.clone()
    }

    pub fn set_src(&self, src: String) {
        let mut state = self.state.lock().unwrap();
        state.src = src.clone();
        state.state = VideoState::Loading;
        state.network_state = network_state::NETWORK_LOADING;
        state.can_play = false;
        state.error = None;
        drop(state);

        // Start loading metadata in background
        self.load_video_metadata(src);
    }

    fn load_video_metadata(&self, src: String) {
        let state = self.state.clone();

        thread::spawn(move || {
            let result = if src.starts_with("data:") {
                Self::parse_data_url_metadata(&src)
            } else if src.starts_with("http://") || src.starts_with("https://") {
                Self::fetch_video_metadata(&src)
            } else {
                Self::read_local_metadata(&src)
            };

            let mut state = state.lock().unwrap();
            match result {
                Ok(metadata) => {
                    state.video_width = metadata.width;
                    state.video_height = metadata.height;
                    state.duration = metadata.duration;
                    state.current_type = metadata.mime_type;
                    state.state = VideoState::Ready;
                    state.network_state = network_state::NETWORK_IDLE;
                    state.ready_state = ready_state::HAVE_ENOUGH_DATA;
                    state.can_play = true;
                    state.buffered_end = metadata.duration;
                }
                Err(e) => {
                    state.state = VideoState::Error;
                    state.network_state = network_state::NETWORK_NO_SOURCE;
                    state.error = Some(e);
                }
            }
        });
    }

    fn parse_data_url_metadata(_data_url: &str) -> Result<VideoMetadata, String> {
        // For data URLs, we can't easily determine video dimensions
        // Return placeholder values
        Ok(VideoMetadata {
            width: 640,
            height: 480,
            duration: 0.0,
            mime_type: String::from("video/mp4"),
        })
    }

    fn fetch_video_metadata(url: &str) -> Result<VideoMetadata, String> {
        // Fetch video headers to determine type
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client
            .head(url)
            .send()
            .map_err(|e| format!("Failed to fetch video: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("HTTP error: {}", response.status()));
        }

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(String::from)
            .unwrap_or_else(|| Self::guess_mime_type(url));

        // For remote videos, we can't easily determine dimensions without downloading
        // Return placeholder dimensions; real implementation would need partial download
        Ok(VideoMetadata {
            width: 1920,
            height: 1080,
            duration: 0.0,
            mime_type: content_type,
        })
    }

    fn read_local_metadata(path: &str) -> Result<VideoMetadata, String> {
        // Check if file exists
        if !std::path::Path::new(path).exists() {
            return Err(format!("File not found: {}", path));
        }

        let mime_type = Self::guess_mime_type(path);

        // Without actual video decoding, return placeholder dimensions
        Ok(VideoMetadata {
            width: 1280,
            height: 720,
            duration: 0.0,
            mime_type,
        })
    }

    fn guess_mime_type(path: &str) -> String {
        let lower = path.to_lowercase();
        if lower.ends_with(".mp4") || lower.ends_with(".m4v") {
            "video/mp4".to_string()
        } else if lower.ends_with(".webm") {
            "video/webm".to_string()
        } else if lower.ends_with(".ogg") || lower.ends_with(".ogv") {
            "video/ogg".to_string()
        } else if lower.ends_with(".mkv") {
            "video/x-matroska".to_string()
        } else if lower.ends_with(".avi") {
            "video/x-msvideo".to_string()
        } else if lower.ends_with(".mov") {
            "video/quicktime".to_string()
        } else {
            "video/mp4".to_string()
        }
    }

    pub fn play(&self) -> Result<(), String> {
        let mut state = self.state.lock().unwrap();

        if state.state == VideoState::Error {
            return Err(state.error.clone().unwrap_or_else(|| "Unknown error".to_string()));
        }

        if state.state == VideoState::Loading {
            return Err("Video is still loading".to_string());
        }

        // Note: In a headless browser, actual video playback isn't needed
        // We just update the state for API compatibility
        state.state = VideoState::Playing;
        Ok(())
    }

    pub fn pause(&self) {
        let mut state = self.state.lock().unwrap();
        if state.state == VideoState::Playing {
            state.state = VideoState::Paused;
        }
    }

    pub fn load(&self) {
        let mut state = self.state.lock().unwrap();
        let src = state.src.clone();
        state.state = VideoState::Loading;
        state.network_state = network_state::NETWORK_LOADING;
        state.ready_state = ready_state::HAVE_NOTHING;
        state.current_time = 0.0;
        drop(state);

        if !src.is_empty() {
            self.load_video_metadata(src);
        }
    }

    // Getters and setters for all properties
    pub fn get_volume(&self) -> f32 { self.state.lock().unwrap().volume }
    pub fn set_volume(&self, v: f32) { self.state.lock().unwrap().volume = v.clamp(0.0, 1.0); }

    pub fn get_muted(&self) -> bool { self.state.lock().unwrap().muted }
    pub fn set_muted(&self, v: bool) { self.state.lock().unwrap().muted = v; }

    pub fn get_loop(&self) -> bool { self.state.lock().unwrap().loop_video }
    pub fn set_loop(&self, v: bool) { self.state.lock().unwrap().loop_video = v; }

    pub fn get_autoplay(&self) -> bool { self.state.lock().unwrap().autoplay }
    pub fn set_autoplay(&self, v: bool) { self.state.lock().unwrap().autoplay = v; }

    pub fn get_controls(&self) -> bool { self.state.lock().unwrap().controls }
    pub fn set_controls(&self, v: bool) { self.state.lock().unwrap().controls = v; }

    pub fn get_poster(&self) -> String { self.state.lock().unwrap().poster.clone() }
    pub fn set_poster(&self, v: String) { self.state.lock().unwrap().poster = v; }

    pub fn get_current_time(&self) -> f64 { self.state.lock().unwrap().current_time }
    pub fn set_current_time(&self, v: f64) {
        let mut state = self.state.lock().unwrap();
        state.current_time = v.max(0.0).min(state.duration);
    }

    pub fn get_duration(&self) -> f64 { self.state.lock().unwrap().duration }
    pub fn get_video_width(&self) -> u32 { self.state.lock().unwrap().video_width }
    pub fn get_video_height(&self) -> u32 { self.state.lock().unwrap().video_height }

    pub fn get_playback_rate(&self) -> f64 { self.state.lock().unwrap().playback_rate }
    pub fn set_playback_rate(&self, v: f64) { self.state.lock().unwrap().playback_rate = v.max(0.0); }

    pub fn get_paused(&self) -> bool {
        let state = self.state.lock().unwrap();
        !matches!(state.state, VideoState::Playing)
    }

    pub fn get_ended(&self) -> bool {
        self.state.lock().unwrap().state == VideoState::Ended
    }

    pub fn get_network_state(&self) -> u16 { self.state.lock().unwrap().network_state }
    pub fn get_ready_state(&self) -> u16 { self.state.lock().unwrap().ready_state }

    pub fn get_preload(&self) -> String { self.state.lock().unwrap().preload.clone() }
    pub fn set_preload(&self, v: String) { self.state.lock().unwrap().preload = v; }

    pub fn get_cross_origin(&self) -> Option<String> { self.state.lock().unwrap().cross_origin.clone() }
    pub fn set_cross_origin(&self, v: Option<String>) { self.state.lock().unwrap().cross_origin = v; }

    pub fn get_seeking(&self) -> bool { self.state.lock().unwrap().seeking }

    pub fn get_buffered_start(&self) -> f64 { self.state.lock().unwrap().buffered_start }
    pub fn get_buffered_end(&self) -> f64 { self.state.lock().unwrap().buffered_end }
}

/// Video metadata
struct VideoMetadata {
    width: u32,
    height: u32,
    duration: f64,
    mime_type: String,
}

/// JavaScript `HTMLVideoElement` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct HTMLVideoElement;

impl IntrinsicObject for HTMLVideoElement {
    fn init(realm: &Realm) {
        // Create getters and setters
        let src_getter = BuiltInBuilder::callable(realm, get_src).name(js_string!("get src")).build();
        let src_setter = BuiltInBuilder::callable(realm, set_src).name(js_string!("set src")).build();

        let volume_getter = BuiltInBuilder::callable(realm, get_volume).name(js_string!("get volume")).build();
        let volume_setter = BuiltInBuilder::callable(realm, set_volume).name(js_string!("set volume")).build();

        let muted_getter = BuiltInBuilder::callable(realm, get_muted).name(js_string!("get muted")).build();
        let muted_setter = BuiltInBuilder::callable(realm, set_muted).name(js_string!("set muted")).build();

        let loop_getter = BuiltInBuilder::callable(realm, get_loop).name(js_string!("get loop")).build();
        let loop_setter = BuiltInBuilder::callable(realm, set_loop).name(js_string!("set loop")).build();

        let autoplay_getter = BuiltInBuilder::callable(realm, get_autoplay).name(js_string!("get autoplay")).build();
        let autoplay_setter = BuiltInBuilder::callable(realm, set_autoplay).name(js_string!("set autoplay")).build();

        let controls_getter = BuiltInBuilder::callable(realm, get_controls).name(js_string!("get controls")).build();
        let controls_setter = BuiltInBuilder::callable(realm, set_controls).name(js_string!("set controls")).build();

        let poster_getter = BuiltInBuilder::callable(realm, get_poster).name(js_string!("get poster")).build();
        let poster_setter = BuiltInBuilder::callable(realm, set_poster).name(js_string!("set poster")).build();

        let current_time_getter = BuiltInBuilder::callable(realm, get_current_time).name(js_string!("get currentTime")).build();
        let current_time_setter = BuiltInBuilder::callable(realm, set_current_time).name(js_string!("set currentTime")).build();

        let duration_getter = BuiltInBuilder::callable(realm, get_duration).name(js_string!("get duration")).build();
        let video_width_getter = BuiltInBuilder::callable(realm, get_video_width).name(js_string!("get videoWidth")).build();
        let video_height_getter = BuiltInBuilder::callable(realm, get_video_height).name(js_string!("get videoHeight")).build();

        let playback_rate_getter = BuiltInBuilder::callable(realm, get_playback_rate).name(js_string!("get playbackRate")).build();
        let playback_rate_setter = BuiltInBuilder::callable(realm, set_playback_rate).name(js_string!("set playbackRate")).build();

        let paused_getter = BuiltInBuilder::callable(realm, get_paused).name(js_string!("get paused")).build();
        let ended_getter = BuiltInBuilder::callable(realm, get_ended).name(js_string!("get ended")).build();

        let network_state_getter = BuiltInBuilder::callable(realm, get_network_state).name(js_string!("get networkState")).build();
        let ready_state_getter = BuiltInBuilder::callable(realm, get_ready_state).name(js_string!("get readyState")).build();

        let preload_getter = BuiltInBuilder::callable(realm, get_preload).name(js_string!("get preload")).build();
        let preload_setter = BuiltInBuilder::callable(realm, set_preload).name(js_string!("set preload")).build();

        let seeking_getter = BuiltInBuilder::callable(realm, get_seeking).name(js_string!("get seeking")).build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            // Constants
            .property(js_string!("HAVE_NOTHING"), ready_state::HAVE_NOTHING, Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT)
            .property(js_string!("HAVE_METADATA"), ready_state::HAVE_METADATA, Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT)
            .property(js_string!("HAVE_CURRENT_DATA"), ready_state::HAVE_CURRENT_DATA, Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT)
            .property(js_string!("HAVE_FUTURE_DATA"), ready_state::HAVE_FUTURE_DATA, Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT)
            .property(js_string!("HAVE_ENOUGH_DATA"), ready_state::HAVE_ENOUGH_DATA, Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT)
            .property(js_string!("NETWORK_EMPTY"), network_state::NETWORK_EMPTY, Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT)
            .property(js_string!("NETWORK_IDLE"), network_state::NETWORK_IDLE, Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT)
            .property(js_string!("NETWORK_LOADING"), network_state::NETWORK_LOADING, Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT)
            .property(js_string!("NETWORK_NO_SOURCE"), network_state::NETWORK_NO_SOURCE, Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT)
            // Accessors
            .accessor(js_string!("src"), Some(src_getter), Some(src_setter), Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("volume"), Some(volume_getter), Some(volume_setter), Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("muted"), Some(muted_getter), Some(muted_setter), Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("loop"), Some(loop_getter), Some(loop_setter), Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("autoplay"), Some(autoplay_getter), Some(autoplay_setter), Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("controls"), Some(controls_getter), Some(controls_setter), Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("poster"), Some(poster_getter), Some(poster_setter), Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("currentTime"), Some(current_time_getter), Some(current_time_setter), Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("duration"), Some(duration_getter), None, Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("videoWidth"), Some(video_width_getter), None, Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("videoHeight"), Some(video_height_getter), None, Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("playbackRate"), Some(playback_rate_getter), Some(playback_rate_setter), Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("paused"), Some(paused_getter), None, Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("ended"), Some(ended_getter), None, Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("networkState"), Some(network_state_getter), None, Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("readyState"), Some(ready_state_getter), None, Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("preload"), Some(preload_getter), Some(preload_setter), Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            .accessor(js_string!("seeking"), Some(seeking_getter), None, Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE)
            // Methods
            .method(play, js_string!("play"), 0)
            .method(pause, js_string!("pause"), 0)
            .method(load, js_string!("load"), 0)
            .method(can_play_type, js_string!("canPlayType"), 1)
            .method(get_video_playback_quality, js_string!("getVideoPlaybackQuality"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLVideoElement {
    const NAME: JsString = StaticJsStrings::HTML_VIDEO_ELEMENT;
}

impl BuiltInConstructor for HTMLVideoElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 30;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 30;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_video_element;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_video_element,
            context,
        )?;

        let data = HTMLVideoElementData::new();

        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            data,
        );

        Ok(obj.into())
    }
}

// === Property Getters and Setters ===

fn get_src(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsString::from(data.get_src()).into())
}

fn set_src(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let src = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    data.set_src(src);
    Ok(JsValue::undefined())
}

fn get_volume(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_volume() as f64))
}

fn set_volume(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let volume = args.get_or_undefined(0).to_number(context)? as f32;
    data.set_volume(volume);
    Ok(JsValue::undefined())
}

fn get_muted(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_muted()))
}

fn set_muted(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let muted = args.get_or_undefined(0).to_boolean();
    data.set_muted(muted);
    Ok(JsValue::undefined())
}

fn get_loop(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_loop()))
}

fn set_loop(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let loop_val = args.get_or_undefined(0).to_boolean();
    data.set_loop(loop_val);
    Ok(JsValue::undefined())
}

fn get_autoplay(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_autoplay()))
}

fn set_autoplay(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let autoplay = args.get_or_undefined(0).to_boolean();
    data.set_autoplay(autoplay);
    Ok(JsValue::undefined())
}

fn get_controls(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_controls()))
}

fn set_controls(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let controls = args.get_or_undefined(0).to_boolean();
    data.set_controls(controls);
    Ok(JsValue::undefined())
}

fn get_poster(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsString::from(data.get_poster()).into())
}

fn set_poster(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let poster = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    data.set_poster(poster);
    Ok(JsValue::undefined())
}

fn get_current_time(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_current_time()))
}

fn set_current_time(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let time = args.get_or_undefined(0).to_number(context)?;
    data.set_current_time(time);
    Ok(JsValue::undefined())
}

fn get_duration(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_duration()))
}

fn get_video_width(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_video_width()))
}

fn get_video_height(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_video_height()))
}

fn get_playback_rate(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_playback_rate()))
}

fn set_playback_rate(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let rate = args.get_or_undefined(0).to_number(context)?;
    data.set_playback_rate(rate);
    Ok(JsValue::undefined())
}

fn get_paused(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_paused()))
}

fn get_ended(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_ended()))
}

fn get_network_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_network_state()))
}

fn get_ready_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_ready_state()))
}

fn get_preload(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsString::from(data.get_preload()).into())
}

fn set_preload(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let preload = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    data.set_preload(preload);
    Ok(JsValue::undefined())
}

fn get_seeking(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    Ok(JsValue::from(data.get_seeking()))
}

// === Methods ===

fn play(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;

    match data.play() {
        Ok(()) => {
            // Return a resolved promise
            let promise_constructor = context.intrinsics().constructors().promise().constructor();
            let promise = boa_engine::builtins::promise::Promise::promise_resolve(
                &promise_constructor,
                JsValue::undefined(),
                context,
            )?;
            Ok(promise.into())
        }
        Err(e) => Err(JsNativeError::error().with_message(e).into()),
    }
}

fn pause(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    data.pause();
    Ok(JsValue::undefined())
}

fn load(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let obj = this.as_object().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    let data = obj.downcast_ref::<HTMLVideoElementData>().ok_or_else(|| JsNativeError::typ().with_message("'this' is not an HTMLVideoElement"))?;
    data.load();
    Ok(JsValue::undefined())
}

fn can_play_type(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let mime_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
    let lower = mime_type.to_lowercase();

    // Return support level: "probably", "maybe", or ""
    let support = if lower.contains("video/mp4") || lower.contains("video/webm") || lower.contains("video/ogg") {
        "probably"
    } else if lower.contains("video/") {
        "maybe"
    } else {
        ""
    };

    Ok(JsString::from(support).into())
}

fn get_video_playback_quality(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Return a VideoPlaybackQuality object
    let obj = JsObject::with_null_proto();
    obj.set(js_string!("creationTime"), JsValue::from(0.0), false, context)?;
    obj.set(js_string!("totalVideoFrames"), JsValue::from(0), false, context)?;
    obj.set(js_string!("droppedVideoFrames"), JsValue::from(0), false, context)?;
    obj.set(js_string!("corruptedVideoFrames"), JsValue::from(0), false, context)?;
    Ok(obj.into())
}
