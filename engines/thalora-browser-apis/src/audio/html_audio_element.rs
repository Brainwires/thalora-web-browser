//! HTMLAudioElement Web API implementation
//!
//! Provides audio playback using rodio with a dedicated audio thread.
//! https://html.spec.whatwg.org/multipage/media.html#htmlaudioelement

use boa_engine::{
    builtins::{BuiltInObject, IntrinsicObject, BuiltInConstructor, BuiltInBuilder},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    object::{internal_methods::get_prototype_from_constructor, JsObject},
    string::StaticJsStrings,
    value::JsValue,
    Context, JsArgs, JsData, JsNativeError, JsResult, js_string,
    JsString, realm::Realm, property::Attribute, Source,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex, mpsc};
use std::io::Cursor;
use std::thread;
use rodio::{OutputStream, Sink, Decoder, Source as RodioSource};

/// Audio state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioState {
    /// Audio is empty (no source)
    Empty,
    /// Audio metadata is being loaded
    Loading,
    /// Audio is ready to play
    Ready,
    /// Audio is playing
    Playing,
    /// Audio is paused
    Paused,
    /// Audio playback ended
    Ended,
    /// Audio had an error
    Error,
}

/// Commands sent to the audio thread
#[derive(Debug)]
enum AudioCommand {
    /// Play audio with data, loop flag, and volume
    Play(Vec<u8>, bool, f32),
    /// Pause playback
    Pause,
    /// Resume playback
    Resume,
    /// Stop playback
    Stop,
    /// Set volume (0.0 - 1.0)
    SetVolume(f32),
    /// Shutdown the audio thread
    Shutdown,
}

/// Internal audio player state
#[derive(Debug)]
pub struct AudioPlayerState {
    /// Source URL
    src: String,
    /// Current state
    state: AudioState,
    /// Volume (0.0 to 1.0)
    volume: f32,
    /// Muted flag
    muted: bool,
    /// Loop flag
    loop_audio: bool,
    /// Autoplay flag
    autoplay: bool,
    /// Current time in seconds
    current_time: f64,
    /// Duration in seconds
    duration: f64,
    /// Playback rate
    playback_rate: f64,
    /// Audio data (raw bytes)
    audio_data: Option<Vec<u8>>,
    /// Is audio ready to play
    can_play: bool,
    /// Error message if any
    error: Option<String>,
}

impl AudioPlayerState {
    pub fn new() -> Self {
        Self {
            src: String::new(),
            state: AudioState::Empty,
            volume: 1.0,
            muted: false,
            loop_audio: false,
            autoplay: false,
            current_time: 0.0,
            duration: 0.0,
            playback_rate: 1.0,
            audio_data: None,
            can_play: false,
            error: None,
        }
    }
}

/// Audio thread handle - owns the channel sender
struct AudioThread {
    sender: mpsc::Sender<AudioCommand>,
}

impl AudioThread {
    fn new() -> Option<Self> {
        let (tx, rx) = mpsc::channel::<AudioCommand>();

        thread::spawn(move || {
            // Create OutputStream on this thread - it stays on this thread
            let Ok((_stream, handle)) = OutputStream::try_default() else {
                eprintln!("Failed to create audio output stream");
                return;
            };

            let mut current_sink: Option<Sink> = None;

            while let Ok(cmd) = rx.recv() {
                match cmd {
                    AudioCommand::Play(data, loop_audio, volume) => {
                        // Stop any existing playback
                        current_sink.take();

                        if let Ok(sink) = Sink::try_new(&handle) {
                            sink.set_volume(volume);

                            let cursor = Cursor::new(data);
                            if let Ok(decoder) = Decoder::new(cursor) {
                                if loop_audio {
                                    sink.append(decoder.repeat_infinite());
                                } else {
                                    sink.append(decoder);
                                }
                                current_sink = Some(sink);
                            }
                        }
                    }
                    AudioCommand::Pause => {
                        if let Some(ref sink) = current_sink {
                            sink.pause();
                        }
                    }
                    AudioCommand::Resume => {
                        if let Some(ref sink) = current_sink {
                            sink.play();
                        }
                    }
                    AudioCommand::Stop => {
                        current_sink.take();
                    }
                    AudioCommand::SetVolume(vol) => {
                        if let Some(ref sink) = current_sink {
                            sink.set_volume(vol);
                        }
                    }
                    AudioCommand::Shutdown => {
                        current_sink.take();
                        break;
                    }
                }
            }
        });

        Some(Self { sender: tx })
    }

    fn send(&self, cmd: AudioCommand) -> Result<(), String> {
        self.sender.send(cmd).map_err(|e| format!("Audio thread error: {}", e))
    }
}

/// Global audio thread (lazily initialized)
static AUDIO_THREAD: once_cell::sync::Lazy<Option<AudioThread>> =
    once_cell::sync::Lazy::new(|| AudioThread::new());

/// Internal data for HTMLAudioElement
#[derive(Clone, Trace, Finalize, JsData)]
pub struct HTMLAudioElementData {
    /// Player state
    #[unsafe_ignore_trace]
    state: Arc<Mutex<AudioPlayerState>>,
}

impl std::fmt::Debug for HTMLAudioElementData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HTMLAudioElementData")
            .field("state", &self.state)
            .finish()
    }
}

impl HTMLAudioElementData {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(AudioPlayerState::new())),
        }
    }

    pub fn get_src(&self) -> String {
        self.state.lock().unwrap().src.clone()
    }

    pub fn set_src(&self, src: String) {
        let mut state = self.state.lock().unwrap();
        state.src = src.clone();
        state.state = AudioState::Loading;
        state.audio_data = None;
        state.can_play = false;
        state.error = None;
        drop(state);

        // Start loading in background
        self.load_audio(src);
    }

    fn load_audio(&self, src: String) {
        let state = self.state.clone();

        // Load audio in a blocking manner for simplicity
        thread::spawn(move || {
            let result = if src.starts_with("data:") {
                // Data URL
                Self::load_from_data_url(&src)
            } else if src.starts_with("http://") || src.starts_with("https://") {
                // HTTP URL
                Self::load_from_url(&src)
            } else {
                // Local file
                Self::load_from_file(&src)
            };

            let mut state = state.lock().unwrap();
            match result {
                Ok((data, duration)) => {
                    state.audio_data = Some(data);
                    state.duration = duration;
                    state.state = AudioState::Ready;
                    state.can_play = true;
                }
                Err(e) => {
                    state.state = AudioState::Error;
                    state.error = Some(e);
                }
            }
        });
    }

    fn load_from_url(url: &str) -> Result<(Vec<u8>, f64), String> {
        // Use tokio block_on for synchronous HTTP request
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("Failed to create tokio runtime: {}", e))?;

        let data = rt.block_on(async {
            let client = rquest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

            let response = client
                .get(url)
                .send()
                .await
                .map_err(|e| format!("Failed to fetch audio: {}", e))?;

            if !response.status().is_success() {
                return Err(format!("HTTP error: {}", response.status()));
            }

            response
                .bytes()
                .await
                .map_err(|e| format!("Failed to read audio data: {}", e))
                .map(|b| b.to_vec())
        })?;

        // Try to get duration from the audio data
        let duration = Self::get_audio_duration(&data).unwrap_or(0.0);

        Ok((data, duration))
    }

    fn load_from_file(path: &str) -> Result<(Vec<u8>, f64), String> {
        let data = std::fs::read(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let duration = Self::get_audio_duration(&data).unwrap_or(0.0);

        Ok((data, duration))
    }

    fn load_from_data_url(data_url: &str) -> Result<(Vec<u8>, f64), String> {
        // Parse data URL: data:[<mediatype>][;base64],<data>
        let content = data_url.strip_prefix("data:").ok_or("Invalid data URL")?;

        let (_, encoded) = content.split_once(',').ok_or("Invalid data URL format")?;

        let data = if content.contains(";base64") {
            base64::Engine::decode(&base64::engine::general_purpose::STANDARD, encoded)
                .map_err(|e| format!("Failed to decode base64: {}", e))?
        } else {
            urlencoding::decode(encoded)
                .map_err(|e| format!("Failed to decode URL encoding: {}", e))?
                .into_owned()
                .into_bytes()
        };

        let duration = Self::get_audio_duration(&data).unwrap_or(0.0);

        Ok((data, duration))
    }

    fn get_audio_duration(data: &[u8]) -> Option<f64> {
        // Try to decode and get duration using the Source trait
        let cursor = Cursor::new(data.to_vec());
        if let Ok(decoder) = Decoder::new(cursor) {
            // Use Source::total_duration() from the RodioSource trait
            if let Some(duration) = RodioSource::total_duration(&decoder) {
                return Some(duration.as_secs_f64());
            }
        }
        None
    }

    pub fn play(&self) -> Result<(), String> {
        let state = self.state.lock().unwrap();

        if state.state == AudioState::Error {
            return Err(state.error.clone().unwrap_or_else(|| "Unknown error".to_string()));
        }

        if state.state == AudioState::Loading {
            return Err("Audio is still loading".to_string());
        }

        // If paused, just resume
        if state.state == AudioState::Paused {
            drop(state);
            if let Some(ref audio_thread) = *AUDIO_THREAD {
                audio_thread.send(AudioCommand::Resume)?;
            }
            let mut state = self.state.lock().unwrap();
            state.state = AudioState::Playing;
            return Ok(());
        }

        let audio_data = state.audio_data.clone().ok_or("No audio data loaded")?;
        let volume = if state.muted { 0.0 } else { state.volume };
        let loop_audio = state.loop_audio;
        drop(state);

        // Send play command to audio thread
        let audio_thread = AUDIO_THREAD.as_ref().ok_or("No audio output available")?;
        audio_thread.send(AudioCommand::Play(audio_data, loop_audio, volume))?;

        // Update state
        let mut state = self.state.lock().unwrap();
        state.state = AudioState::Playing;

        Ok(())
    }

    pub fn pause(&self) {
        if let Some(ref audio_thread) = *AUDIO_THREAD {
            let _ = audio_thread.send(AudioCommand::Pause);
        }
        let mut state = self.state.lock().unwrap();
        if state.state == AudioState::Playing {
            state.state = AudioState::Paused;
        }
    }

    pub fn stop(&self) {
        if let Some(ref audio_thread) = *AUDIO_THREAD {
            let _ = audio_thread.send(AudioCommand::Stop);
        }
        let mut state = self.state.lock().unwrap();
        state.state = AudioState::Ready;
        state.current_time = 0.0;
    }

    pub fn get_volume(&self) -> f32 {
        self.state.lock().unwrap().volume
    }

    pub fn set_volume(&self, volume: f32) {
        let volume = volume.clamp(0.0, 1.0);
        let mut state = self.state.lock().unwrap();
        state.volume = volume;

        if !state.muted {
            if let Some(ref audio_thread) = *AUDIO_THREAD {
                let _ = audio_thread.send(AudioCommand::SetVolume(volume));
            }
        }
    }

    pub fn get_muted(&self) -> bool {
        self.state.lock().unwrap().muted
    }

    pub fn set_muted(&self, muted: bool) {
        let mut state = self.state.lock().unwrap();
        state.muted = muted;

        let volume = if muted { 0.0 } else { state.volume };
        if let Some(ref audio_thread) = *AUDIO_THREAD {
            let _ = audio_thread.send(AudioCommand::SetVolume(volume));
        }
    }

    pub fn get_loop(&self) -> bool {
        self.state.lock().unwrap().loop_audio
    }

    pub fn set_loop(&self, loop_audio: bool) {
        self.state.lock().unwrap().loop_audio = loop_audio;
    }

    pub fn get_autoplay(&self) -> bool {
        self.state.lock().unwrap().autoplay
    }

    pub fn set_autoplay(&self, autoplay: bool) {
        self.state.lock().unwrap().autoplay = autoplay;
    }

    pub fn get_current_time(&self) -> f64 {
        self.state.lock().unwrap().current_time
    }

    pub fn set_current_time(&self, _time: f64) {
        // Seeking is complex with rodio - would require recreating the sink
        // For now, this is a no-op
    }

    pub fn get_duration(&self) -> f64 {
        self.state.lock().unwrap().duration
    }

    pub fn get_paused(&self) -> bool {
        let state = self.state.lock().unwrap();
        matches!(state.state, AudioState::Empty | AudioState::Paused | AudioState::Ended | AudioState::Ready)
    }

    pub fn get_ended(&self) -> bool {
        let state = self.state.lock().unwrap();
        state.state == AudioState::Ended
    }

    pub fn get_ready_state(&self) -> u16 {
        let state = self.state.lock().unwrap();
        match state.state {
            AudioState::Empty => 0,      // HAVE_NOTHING
            AudioState::Loading => 1,    // HAVE_METADATA
            AudioState::Ready | AudioState::Paused => 4, // HAVE_ENOUGH_DATA
            AudioState::Playing => 4,    // HAVE_ENOUGH_DATA
            AudioState::Ended => 4,      // HAVE_ENOUGH_DATA
            AudioState::Error => 0,      // HAVE_NOTHING
        }
    }

    pub fn can_play(&self) -> bool {
        self.state.lock().unwrap().can_play
    }
}

/// JavaScript `HTMLAudioElement` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct HTMLAudioElement;

impl IntrinsicObject for HTMLAudioElement {
    fn init(realm: &Realm) {
        // Create getters and setters
        let src_getter = BuiltInBuilder::callable(realm, get_src)
            .name(js_string!("get src"))
            .build();
        let src_setter = BuiltInBuilder::callable(realm, set_src)
            .name(js_string!("set src"))
            .build();

        let volume_getter = BuiltInBuilder::callable(realm, get_volume)
            .name(js_string!("get volume"))
            .build();
        let volume_setter = BuiltInBuilder::callable(realm, set_volume)
            .name(js_string!("set volume"))
            .build();

        let muted_getter = BuiltInBuilder::callable(realm, get_muted)
            .name(js_string!("get muted"))
            .build();
        let muted_setter = BuiltInBuilder::callable(realm, set_muted)
            .name(js_string!("set muted"))
            .build();

        let loop_getter = BuiltInBuilder::callable(realm, get_loop)
            .name(js_string!("get loop"))
            .build();
        let loop_setter = BuiltInBuilder::callable(realm, set_loop)
            .name(js_string!("set loop"))
            .build();

        let autoplay_getter = BuiltInBuilder::callable(realm, get_autoplay)
            .name(js_string!("get autoplay"))
            .build();
        let autoplay_setter = BuiltInBuilder::callable(realm, set_autoplay)
            .name(js_string!("set autoplay"))
            .build();

        let current_time_getter = BuiltInBuilder::callable(realm, get_current_time)
            .name(js_string!("get currentTime"))
            .build();
        let current_time_setter = BuiltInBuilder::callable(realm, set_current_time)
            .name(js_string!("set currentTime"))
            .build();

        let duration_getter = BuiltInBuilder::callable(realm, get_duration)
            .name(js_string!("get duration"))
            .build();

        let paused_getter = BuiltInBuilder::callable(realm, get_paused)
            .name(js_string!("get paused"))
            .build();

        let ended_getter = BuiltInBuilder::callable(realm, get_ended)
            .name(js_string!("get ended"))
            .build();

        let ready_state_getter = BuiltInBuilder::callable(realm, get_ready_state)
            .name(js_string!("get readyState"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .inherits(Some(realm.intrinsics().constructors().html_element().prototype()))
            .accessor(
                js_string!("src"),
                Some(src_getter),
                Some(src_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("volume"),
                Some(volume_getter),
                Some(volume_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("muted"),
                Some(muted_getter),
                Some(muted_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("loop"),
                Some(loop_getter),
                Some(loop_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("autoplay"),
                Some(autoplay_getter),
                Some(autoplay_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("currentTime"),
                Some(current_time_getter),
                Some(current_time_setter),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("duration"),
                Some(duration_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("paused"),
                Some(paused_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("ended"),
                Some(ended_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("readyState"),
                Some(ready_state_getter),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(play, js_string!("play"), 0)
            .method(pause, js_string!("pause"), 0)
            .method(load, js_string!("load"), 0)
            .method(can_play_type, js_string!("canPlayType"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for HTMLAudioElement {
    const NAME: JsString = StaticJsStrings::HTML_AUDIO_ELEMENT;
}

impl BuiltInConstructor for HTMLAudioElement {
    const CONSTRUCTOR_ARGUMENTS: usize = 1;
    const PROTOTYPE_STORAGE_SLOTS: usize = 30;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 30;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::html_audio_element;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::html_audio_element,
            context,
        )?;

        let audio_data = HTMLAudioElementData::new();

        // Handle optional source URL argument
        if let Some(src) = args.get(0) {
            if !src.is_undefined() {
                let src_str = src.to_string(context)?.to_std_string_escaped();
                audio_data.set_src(src_str);
            }
        }

        let obj = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            audio_data,
        );

        Ok(obj.into())
    }
}

// ============== Property Accessors ==============

fn get_src(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            return Ok(JsValue::from(js_string!(data.get_src())));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn set_src(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            let src = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
            data.set_src(src);
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn get_volume(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            return Ok(JsValue::from(data.get_volume() as f64));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn set_volume(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            let volume = args.get_or_undefined(0).to_number(context)? as f32;
            data.set_volume(volume);
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn get_muted(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            return Ok(JsValue::from(data.get_muted()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn set_muted(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            let muted = args.get_or_undefined(0).to_boolean();
            data.set_muted(muted);
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn get_loop(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            return Ok(JsValue::from(data.get_loop()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn set_loop(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            let loop_audio = args.get_or_undefined(0).to_boolean();
            data.set_loop(loop_audio);
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn get_autoplay(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            return Ok(JsValue::from(data.get_autoplay()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn set_autoplay(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            let autoplay = args.get_or_undefined(0).to_boolean();
            data.set_autoplay(autoplay);
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn get_current_time(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            return Ok(JsValue::from(data.get_current_time()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn set_current_time(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            let time = args.get_or_undefined(0).to_number(context)?;
            data.set_current_time(time);
            return Ok(JsValue::undefined());
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn get_duration(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            let duration = data.get_duration();
            if duration == 0.0 {
                return Ok(JsValue::from(f64::NAN));
            }
            return Ok(JsValue::from(duration));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn get_paused(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            return Ok(JsValue::from(data.get_paused()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn get_ended(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            return Ok(JsValue::from(data.get_ended()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

fn get_ready_state(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    if let Some(obj) = this.as_object() {
        if let Some(data) = obj.downcast_ref::<HTMLAudioElementData>() {
            return Ok(JsValue::from(data.get_ready_state()));
        }
    }
    Err(JsNativeError::typ()
        .with_message("'this' is not an HTMLAudioElement")
        .into())
}

// ============== Methods ==============

fn play(this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let audio_data = this_obj.downcast_ref::<HTMLAudioElementData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an HTMLAudioElement")
    })?;

    match audio_data.play() {
        Ok(()) => {
            // Return a resolved promise
            context.eval(Source::from_bytes("Promise.resolve()"))
        }
        Err(e) => {
            // Return a rejected promise
            context.eval(Source::from_bytes(&format!(
                "Promise.reject(new Error('{}'))",
                e.replace('\'', "\\'")
            )))
        }
    }
}

fn pause(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let audio_data = this_obj.downcast_ref::<HTMLAudioElementData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an HTMLAudioElement")
    })?;

    audio_data.pause();
    Ok(JsValue::undefined())
}

fn load(this: &JsValue, _args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let this_obj = this.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an object")
    })?;

    let audio_data = this_obj.downcast_ref::<HTMLAudioElementData>().ok_or_else(|| {
        JsNativeError::typ().with_message("'this' is not an HTMLAudioElement")
    })?;

    // Reload the current source
    let src = audio_data.get_src();
    if !src.is_empty() {
        audio_data.set_src(src);
    }

    Ok(JsValue::undefined())
}

fn can_play_type(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let mime_type = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();

    // Check supported types based on rodio/symphonia capabilities
    let support = match mime_type.as_str() {
        "audio/mp3" | "audio/mpeg" => "probably",
        "audio/wav" | "audio/wave" | "audio/x-wav" => "probably",
        "audio/ogg" | "audio/ogg; codecs=\"vorbis\"" => "probably",
        "audio/flac" => "probably",
        "audio/aac" | "audio/mp4" => "maybe",
        "audio/webm" | "audio/webm; codecs=\"opus\"" => "maybe",
        _ => "",
    };

    Ok(JsValue::from(js_string!(support)))
}
