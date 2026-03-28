//! AudioContext Web Audio API implementation
//!
//! Provides the Web Audio API for advanced audio processing.
//! https://webaudio.github.io/web-audio-api/

use boa_engine::{
    Context, JsArgs, JsData, JsNativeError, JsResult, JsString,
    builtins::{BuiltInBuilder, BuiltInConstructor, BuiltInObject, IntrinsicObject},
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{JsObject, internal_methods::get_prototype_from_constructor},
    property::Attribute,
    realm::Realm,
    string::StaticJsStrings,
    value::JsValue,
};
use boa_gc::{Finalize, Trace};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Audio context state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioContextState {
    /// The audio context has been created but not started
    Suspended,
    /// The audio context is running
    Running,
    /// The audio context has been closed
    Closed,
}

impl std::fmt::Display for AudioContextState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioContextState::Suspended => write!(f, "suspended"),
            AudioContextState::Running => write!(f, "running"),
            AudioContextState::Closed => write!(f, "closed"),
        }
    }
}

/// Internal state for AudioContext
#[derive(Debug)]
pub struct AudioContextInner {
    /// Current state
    state: AudioContextState,
    /// Sample rate in Hz
    sample_rate: f32,
    /// Base latency in seconds
    base_latency: f64,
    /// Output latency in seconds
    output_latency: f64,
    /// Start time for currentTime calculation
    start_time: Option<Instant>,
    /// Current time when suspended
    suspended_time: f64,
}

impl AudioContextInner {
    pub fn new() -> Self {
        Self {
            state: AudioContextState::Suspended,
            sample_rate: 44100.0,
            base_latency: 0.01,
            output_latency: 0.02,
            start_time: None,
            suspended_time: 0.0,
        }
    }

    pub fn get_current_time(&self) -> f64 {
        match self.state {
            AudioContextState::Running => {
                if let Some(start) = self.start_time {
                    start.elapsed().as_secs_f64()
                } else {
                    0.0
                }
            }
            _ => self.suspended_time,
        }
    }
}

/// Internal data for AudioContext
#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub struct AudioContextData {
    #[unsafe_ignore_trace]
    inner: Arc<Mutex<AudioContextInner>>,
}

impl AudioContextData {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AudioContextInner::new())),
        }
    }

    pub fn get_state(&self) -> AudioContextState {
        self.inner.lock().unwrap().state
    }

    pub fn get_sample_rate(&self) -> f32 {
        self.inner.lock().unwrap().sample_rate
    }

    pub fn get_current_time(&self) -> f64 {
        self.inner.lock().unwrap().get_current_time()
    }

    pub fn get_base_latency(&self) -> f64 {
        self.inner.lock().unwrap().base_latency
    }

    pub fn get_output_latency(&self) -> f64 {
        self.inner.lock().unwrap().output_latency
    }

    pub fn resume(&self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.state == AudioContextState::Suspended {
            inner.state = AudioContextState::Running;
            inner.start_time = Some(Instant::now());
        }
    }

    pub fn suspend(&self) {
        let mut inner = self.inner.lock().unwrap();
        if inner.state == AudioContextState::Running {
            inner.suspended_time = inner.get_current_time();
            inner.state = AudioContextState::Suspended;
            inner.start_time = None;
        }
    }

    pub fn close(&self) {
        let mut inner = self.inner.lock().unwrap();
        inner.suspended_time = inner.get_current_time();
        inner.state = AudioContextState::Closed;
        inner.start_time = None;
    }
}

/// JavaScript `AudioContext` builtin implementation.
#[derive(Debug, Copy, Clone)]
pub struct AudioContext;

impl IntrinsicObject for AudioContext {
    fn init(realm: &Realm) {
        // Create getters
        let state_getter = BuiltInBuilder::callable(realm, get_state)
            .name(js_string!("get state"))
            .build();
        let sample_rate_getter = BuiltInBuilder::callable(realm, get_sample_rate)
            .name(js_string!("get sampleRate"))
            .build();
        let current_time_getter = BuiltInBuilder::callable(realm, get_current_time)
            .name(js_string!("get currentTime"))
            .build();
        let base_latency_getter = BuiltInBuilder::callable(realm, get_base_latency)
            .name(js_string!("get baseLatency"))
            .build();
        let output_latency_getter = BuiltInBuilder::callable(realm, get_output_latency)
            .name(js_string!("get outputLatency"))
            .build();
        let destination_getter = BuiltInBuilder::callable(realm, get_destination)
            .name(js_string!("get destination"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .property(
                js_string!("HAVE_NOTHING"),
                0,
                Attribute::READONLY | Attribute::NON_ENUMERABLE | Attribute::PERMANENT,
            )
            .accessor(
                js_string!("state"),
                Some(state_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .accessor(
                js_string!("sampleRate"),
                Some(sample_rate_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .accessor(
                js_string!("currentTime"),
                Some(current_time_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .accessor(
                js_string!("baseLatency"),
                Some(base_latency_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .accessor(
                js_string!("outputLatency"),
                Some(output_latency_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .accessor(
                js_string!("destination"),
                Some(destination_getter),
                None,
                Attribute::CONFIGURABLE | Attribute::NON_ENUMERABLE,
            )
            .method(resume, js_string!("resume"), 0)
            .method(suspend, js_string!("suspend"), 0)
            .method(close, js_string!("close"), 0)
            .method(create_gain, js_string!("createGain"), 0)
            .method(create_oscillator, js_string!("createOscillator"), 0)
            .method(create_buffer_source, js_string!("createBufferSource"), 0)
            .method(create_analyser, js_string!("createAnalyser"), 0)
            .method(create_buffer, js_string!("createBuffer"), 3)
            .method(decode_audio_data, js_string!("decodeAudioData"), 1)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for AudioContext {
    const NAME: JsString = StaticJsStrings::AUDIO_CONTEXT;
}

impl BuiltInConstructor for AudioContext {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 22; // Accessors and methods on prototype
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 20;
    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::audio_context;

    fn constructor(
        new_target: &JsValue,
        _args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::audio_context,
            context,
        )?;

        let data = AudioContextData::new();

        let obj =
            JsObject::from_proto_and_data_with_shared_shape(context.root_shape(), prototype, data);

        Ok(obj.into())
    }
}

// === Property Getters ===

fn get_state(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this = _this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    let data = this
        .downcast_ref::<AudioContextData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    Ok(JsString::from(data.get_state().to_string()).into())
}

fn get_sample_rate(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this = _this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    let data = this
        .downcast_ref::<AudioContextData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    Ok(JsValue::from(data.get_sample_rate() as f64))
}

fn get_current_time(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this = _this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    let data = this
        .downcast_ref::<AudioContextData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    Ok(JsValue::from(data.get_current_time()))
}

fn get_base_latency(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this = _this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    let data = this
        .downcast_ref::<AudioContextData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    Ok(JsValue::from(data.get_base_latency()))
}

fn get_output_latency(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    let this = _this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    let data = this
        .downcast_ref::<AudioContextData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    Ok(JsValue::from(data.get_output_latency()))
}

fn get_destination(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Return a basic AudioDestinationNode-like object
    let obj = JsObject::with_null_proto();
    obj.set(
        js_string!("numberOfInputs"),
        JsValue::from(1),
        false,
        context,
    )?;
    obj.set(
        js_string!("numberOfOutputs"),
        JsValue::from(0),
        false,
        context,
    )?;
    obj.set(js_string!("channelCount"), JsValue::from(2), false, context)?;
    obj.set(
        js_string!("maxChannelCount"),
        JsValue::from(2),
        false,
        context,
    )?;
    Ok(obj.into())
}

// === Methods ===

fn resume(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this = _this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    let data = this
        .downcast_ref::<AudioContextData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    data.resume();

    // Return a resolved promise
    let promise_constructor = context.intrinsics().constructors().promise().constructor();
    let promise = boa_engine::builtins::promise::Promise::promise_resolve(
        &promise_constructor,
        JsValue::undefined(),
        context,
    )?;
    Ok(promise.into())
}

fn suspend(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this = _this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    let data = this
        .downcast_ref::<AudioContextData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    data.suspend();

    let promise_constructor = context.intrinsics().constructors().promise().constructor();
    let promise = boa_engine::builtins::promise::Promise::promise_resolve(
        &promise_constructor,
        JsValue::undefined(),
        context,
    )?;
    Ok(promise.into())
}

fn close(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let this = _this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    let data = this
        .downcast_ref::<AudioContextData>()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioContext"))?;

    data.close();

    let promise_constructor = context.intrinsics().constructors().promise().constructor();
    let promise = boa_engine::builtins::promise::Promise::promise_resolve(
        &promise_constructor,
        JsValue::undefined(),
        context,
    )?;
    Ok(promise.into())
}

fn create_gain(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create a GainNode-like object
    let obj = JsObject::with_null_proto();
    obj.set(
        js_string!("numberOfInputs"),
        JsValue::from(1),
        false,
        context,
    )?;
    obj.set(
        js_string!("numberOfOutputs"),
        JsValue::from(1),
        false,
        context,
    )?;
    obj.set(js_string!("channelCount"), JsValue::from(2), false, context)?;

    // Create gain AudioParam
    let gain_param = create_audio_param(context, 1.0, 0.0, f32::MAX as f64)?;
    obj.set(js_string!("gain"), gain_param, false, context)?;

    // Add connect method
    let connect_fn = BuiltInBuilder::callable(context.realm(), node_connect)
        .name(js_string!("connect"))
        .build();
    obj.set(js_string!("connect"), connect_fn, false, context)?;

    // Add disconnect method
    let disconnect_fn = BuiltInBuilder::callable(context.realm(), node_disconnect)
        .name(js_string!("disconnect"))
        .build();
    obj.set(js_string!("disconnect"), disconnect_fn, false, context)?;

    Ok(obj.into())
}

fn create_oscillator(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    // Create an OscillatorNode-like object
    let obj = JsObject::with_null_proto();
    obj.set(
        js_string!("numberOfInputs"),
        JsValue::from(0),
        false,
        context,
    )?;
    obj.set(
        js_string!("numberOfOutputs"),
        JsValue::from(1),
        false,
        context,
    )?;
    obj.set(js_string!("channelCount"), JsValue::from(2), false, context)?;
    obj.set(js_string!("type"), JsString::from("sine"), false, context)?;

    // Create frequency and detune AudioParams
    let frequency_param = create_audio_param(context, 440.0, -22050.0, 22050.0)?;
    obj.set(js_string!("frequency"), frequency_param, false, context)?;

    let detune_param = create_audio_param(context, 0.0, -1200.0, 1200.0)?;
    obj.set(js_string!("detune"), detune_param, false, context)?;

    // Add methods
    let connect_fn = BuiltInBuilder::callable(context.realm(), node_connect)
        .name(js_string!("connect"))
        .build();
    obj.set(js_string!("connect"), connect_fn, false, context)?;

    let disconnect_fn = BuiltInBuilder::callable(context.realm(), node_disconnect)
        .name(js_string!("disconnect"))
        .build();
    obj.set(js_string!("disconnect"), disconnect_fn, false, context)?;

    let start_fn = BuiltInBuilder::callable(context.realm(), oscillator_start)
        .name(js_string!("start"))
        .build();
    obj.set(js_string!("start"), start_fn, false, context)?;

    let stop_fn = BuiltInBuilder::callable(context.realm(), oscillator_stop)
        .name(js_string!("stop"))
        .build();
    obj.set(js_string!("stop"), stop_fn, false, context)?;

    Ok(obj.into())
}

fn create_buffer_source(
    _this: &JsValue,
    _args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    // Create an AudioBufferSourceNode-like object
    let obj = JsObject::with_null_proto();
    obj.set(
        js_string!("numberOfInputs"),
        JsValue::from(0),
        false,
        context,
    )?;
    obj.set(
        js_string!("numberOfOutputs"),
        JsValue::from(1),
        false,
        context,
    )?;
    obj.set(js_string!("channelCount"), JsValue::from(2), false, context)?;
    obj.set(js_string!("buffer"), JsValue::null(), false, context)?;
    obj.set(js_string!("loop"), JsValue::from(false), false, context)?;
    obj.set(js_string!("loopStart"), JsValue::from(0.0), false, context)?;
    obj.set(js_string!("loopEnd"), JsValue::from(0.0), false, context)?;

    // Create playbackRate AudioParam
    let playback_rate_param = create_audio_param(context, 1.0, 0.0, 1024.0)?;
    obj.set(
        js_string!("playbackRate"),
        playback_rate_param,
        false,
        context,
    )?;

    // Create detune AudioParam
    let detune_param = create_audio_param(context, 0.0, -1200.0, 1200.0)?;
    obj.set(js_string!("detune"), detune_param, false, context)?;

    // Add methods
    let connect_fn = BuiltInBuilder::callable(context.realm(), node_connect)
        .name(js_string!("connect"))
        .build();
    obj.set(js_string!("connect"), connect_fn, false, context)?;

    let disconnect_fn = BuiltInBuilder::callable(context.realm(), node_disconnect)
        .name(js_string!("disconnect"))
        .build();
    obj.set(js_string!("disconnect"), disconnect_fn, false, context)?;

    let start_fn = BuiltInBuilder::callable(context.realm(), buffer_source_start)
        .name(js_string!("start"))
        .build();
    obj.set(js_string!("start"), start_fn, false, context)?;

    let stop_fn = BuiltInBuilder::callable(context.realm(), buffer_source_stop)
        .name(js_string!("stop"))
        .build();
    obj.set(js_string!("stop"), stop_fn, false, context)?;

    Ok(obj.into())
}

fn create_analyser(_this: &JsValue, _args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    // Create an AnalyserNode-like object
    let obj = JsObject::with_null_proto();
    obj.set(
        js_string!("numberOfInputs"),
        JsValue::from(1),
        false,
        context,
    )?;
    obj.set(
        js_string!("numberOfOutputs"),
        JsValue::from(1),
        false,
        context,
    )?;
    obj.set(js_string!("channelCount"), JsValue::from(2), false, context)?;
    obj.set(js_string!("fftSize"), JsValue::from(2048), false, context)?;
    obj.set(
        js_string!("frequencyBinCount"),
        JsValue::from(1024),
        false,
        context,
    )?;
    obj.set(
        js_string!("minDecibels"),
        JsValue::from(-100.0),
        false,
        context,
    )?;
    obj.set(
        js_string!("maxDecibels"),
        JsValue::from(-30.0),
        false,
        context,
    )?;
    obj.set(
        js_string!("smoothingTimeConstant"),
        JsValue::from(0.8),
        false,
        context,
    )?;

    // Add methods
    let connect_fn = BuiltInBuilder::callable(context.realm(), node_connect)
        .name(js_string!("connect"))
        .build();
    obj.set(js_string!("connect"), connect_fn, false, context)?;

    let disconnect_fn = BuiltInBuilder::callable(context.realm(), node_disconnect)
        .name(js_string!("disconnect"))
        .build();
    obj.set(js_string!("disconnect"), disconnect_fn, false, context)?;

    let get_byte_frequency_data_fn =
        BuiltInBuilder::callable(context.realm(), analyser_get_byte_frequency_data)
            .name(js_string!("getByteFrequencyData"))
            .build();
    obj.set(
        js_string!("getByteFrequencyData"),
        get_byte_frequency_data_fn,
        false,
        context,
    )?;

    let get_float_frequency_data_fn =
        BuiltInBuilder::callable(context.realm(), analyser_get_float_frequency_data)
            .name(js_string!("getFloatFrequencyData"))
            .build();
    obj.set(
        js_string!("getFloatFrequencyData"),
        get_float_frequency_data_fn,
        false,
        context,
    )?;

    let get_byte_time_domain_data_fn =
        BuiltInBuilder::callable(context.realm(), analyser_get_byte_time_domain_data)
            .name(js_string!("getByteTimeDomainData"))
            .build();
    obj.set(
        js_string!("getByteTimeDomainData"),
        get_byte_time_domain_data_fn,
        false,
        context,
    )?;

    let get_float_time_domain_data_fn =
        BuiltInBuilder::callable(context.realm(), analyser_get_float_time_domain_data)
            .name(js_string!("getFloatTimeDomainData"))
            .build();
    obj.set(
        js_string!("getFloatTimeDomainData"),
        get_float_time_domain_data_fn,
        false,
        context,
    )?;

    Ok(obj.into())
}

fn create_buffer(_this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
    let num_channels = args.get_or_undefined(0).to_u32(context)?.max(1);
    let length = args.get_or_undefined(1).to_u32(context)?;
    let sample_rate = args.get_or_undefined(2).to_number(context)? as f32;

    // Create an AudioBuffer-like object
    let obj = JsObject::with_null_proto();
    obj.set(
        js_string!("numberOfChannels"),
        JsValue::from(num_channels),
        false,
        context,
    )?;
    obj.set(js_string!("length"), JsValue::from(length), false, context)?;
    obj.set(
        js_string!("sampleRate"),
        JsValue::from(sample_rate as f64),
        false,
        context,
    )?;
    obj.set(
        js_string!("duration"),
        JsValue::from(length as f64 / sample_rate as f64),
        false,
        context,
    )?;

    // Add getChannelData method
    let get_channel_data_fn = BuiltInBuilder::callable(context.realm(), buffer_get_channel_data)
        .name(js_string!("getChannelData"))
        .build();
    obj.set(
        js_string!("getChannelData"),
        get_channel_data_fn,
        false,
        context,
    )?;

    // Add copyFromChannel and copyToChannel methods
    let copy_from_fn = BuiltInBuilder::callable(context.realm(), buffer_copy_from_channel)
        .name(js_string!("copyFromChannel"))
        .build();
    obj.set(js_string!("copyFromChannel"), copy_from_fn, false, context)?;

    let copy_to_fn = BuiltInBuilder::callable(context.realm(), buffer_copy_to_channel)
        .name(js_string!("copyToChannel"))
        .build();
    obj.set(js_string!("copyToChannel"), copy_to_fn, false, context)?;

    Ok(obj.into())
}

fn decode_audio_data(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let _array_buffer = args.get_or_undefined(0);

    // For now, return a promise that resolves to an empty audio buffer
    // Real implementation would decode the audio data using symphonia
    let buffer = create_buffer(
        _this,
        &[
            JsValue::from(2),       // channels
            JsValue::from(44100),   // length (1 second)
            JsValue::from(44100.0), // sample rate
        ],
        context,
    )?;

    let promise_constructor = context.intrinsics().constructors().promise().constructor();
    let promise = boa_engine::builtins::promise::Promise::promise_resolve(
        &promise_constructor,
        buffer,
        context,
    )?;
    Ok(promise.into())
}

// === Helper Functions ===

fn create_audio_param(
    context: &mut Context,
    default: f64,
    min: f64,
    max: f64,
) -> JsResult<JsValue> {
    let obj = JsObject::with_null_proto();
    obj.set(js_string!("value"), JsValue::from(default), false, context)?;
    obj.set(
        js_string!("defaultValue"),
        JsValue::from(default),
        false,
        context,
    )?;
    obj.set(js_string!("minValue"), JsValue::from(min), false, context)?;
    obj.set(js_string!("maxValue"), JsValue::from(max), false, context)?;
    obj.set(
        js_string!("automationRate"),
        JsString::from("a-rate"),
        false,
        context,
    )?;

    // Add automation methods
    let set_value_at_time_fn = BuiltInBuilder::callable(context.realm(), param_set_value_at_time)
        .name(js_string!("setValueAtTime"))
        .build();
    obj.set(
        js_string!("setValueAtTime"),
        set_value_at_time_fn,
        false,
        context,
    )?;

    let linear_ramp_fn =
        BuiltInBuilder::callable(context.realm(), param_linear_ramp_to_value_at_time)
            .name(js_string!("linearRampToValueAtTime"))
            .build();
    obj.set(
        js_string!("linearRampToValueAtTime"),
        linear_ramp_fn,
        false,
        context,
    )?;

    let exponential_ramp_fn =
        BuiltInBuilder::callable(context.realm(), param_exponential_ramp_to_value_at_time)
            .name(js_string!("exponentialRampToValueAtTime"))
            .build();
    obj.set(
        js_string!("exponentialRampToValueAtTime"),
        exponential_ramp_fn,
        false,
        context,
    )?;

    let set_target_at_time_fn = BuiltInBuilder::callable(context.realm(), param_set_target_at_time)
        .name(js_string!("setTargetAtTime"))
        .build();
    obj.set(
        js_string!("setTargetAtTime"),
        set_target_at_time_fn,
        false,
        context,
    )?;

    let set_value_curve_fn =
        BuiltInBuilder::callable(context.realm(), param_set_value_curve_at_time)
            .name(js_string!("setValueCurveAtTime"))
            .build();
    obj.set(
        js_string!("setValueCurveAtTime"),
        set_value_curve_fn,
        false,
        context,
    )?;

    let cancel_scheduled_fn =
        BuiltInBuilder::callable(context.realm(), param_cancel_scheduled_values)
            .name(js_string!("cancelScheduledValues"))
            .build();
    obj.set(
        js_string!("cancelScheduledValues"),
        cancel_scheduled_fn,
        false,
        context,
    )?;

    let cancel_and_hold_fn =
        BuiltInBuilder::callable(context.realm(), param_cancel_and_hold_at_time)
            .name(js_string!("cancelAndHoldAtTime"))
            .build();
    obj.set(
        js_string!("cancelAndHoldAtTime"),
        cancel_and_hold_fn,
        false,
        context,
    )?;

    Ok(obj.into())
}

// === Node Methods ===

fn node_connect(this: &JsValue, args: &[JsValue], _context: &mut Context) -> JsResult<JsValue> {
    let _destination = args.get_or_undefined(0);
    // In a real implementation, this would establish the audio connection
    // For now, just return the destination for chaining
    Ok(args.get_or_undefined(0).clone())
}

fn node_disconnect(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    // In a real implementation, this would disconnect the node
    Ok(JsValue::undefined())
}

fn oscillator_start(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    // In a real implementation, this would start the oscillator
    Ok(JsValue::undefined())
}

fn oscillator_stop(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    // In a real implementation, this would stop the oscillator
    Ok(JsValue::undefined())
}

fn buffer_source_start(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    // In a real implementation, this would start playback
    Ok(JsValue::undefined())
}

fn buffer_source_stop(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    // In a real implementation, this would stop playback
    Ok(JsValue::undefined())
}

// === Analyser Methods ===

fn analyser_get_byte_frequency_data(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    // In a real implementation, this would fill the array with frequency data
    Ok(JsValue::undefined())
}

fn analyser_get_float_frequency_data(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    // In a real implementation, this would fill the array with frequency data
    Ok(JsValue::undefined())
}

fn analyser_get_byte_time_domain_data(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    // In a real implementation, this would fill the array with time domain data
    Ok(JsValue::undefined())
}

fn analyser_get_float_time_domain_data(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    // In a real implementation, this would fill the array with time domain data
    Ok(JsValue::undefined())
}

// === AudioBuffer Methods ===

fn buffer_get_channel_data(
    _this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let this_obj = _this
        .as_object()
        .ok_or_else(|| JsNativeError::typ().with_message("'this' is not an AudioBuffer"))?;

    let length = this_obj
        .get(js_string!("length"), context)?
        .to_u32(context)? as usize;

    let _channel = args.get_or_undefined(0).to_u32(context)? as usize;

    // Create Float32Array by calling its constructor
    let float32_constructor = context
        .intrinsics()
        .constructors()
        .typed_float32_array()
        .constructor();
    let typed_array = float32_constructor.construct(
        &[JsValue::from(length)],
        Some(&float32_constructor),
        context,
    )?;

    Ok(typed_array.into())
}

fn buffer_copy_from_channel(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    // In a real implementation, this would copy data from the channel
    Ok(JsValue::undefined())
}

fn buffer_copy_to_channel(
    _this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    // In a real implementation, this would copy data to the channel
    Ok(JsValue::undefined())
}

// === AudioParam Methods ===

fn param_set_value_at_time(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_number(context)?;
    if let Some(this_obj) = this.as_object() {
        this_obj.set(js_string!("value"), JsValue::from(value), false, context)?;
    }
    Ok(this.clone())
}

fn param_linear_ramp_to_value_at_time(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_number(context)?;
    // For simplicity, just set the value immediately
    if let Some(this_obj) = this.as_object() {
        this_obj.set(js_string!("value"), JsValue::from(value), false, context)?;
    }
    Ok(this.clone())
}

fn param_exponential_ramp_to_value_at_time(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let value = args.get_or_undefined(0).to_number(context)?;
    // For simplicity, just set the value immediately
    if let Some(this_obj) = this.as_object() {
        this_obj.set(js_string!("value"), JsValue::from(value), false, context)?;
    }
    Ok(this.clone())
}

fn param_set_target_at_time(
    this: &JsValue,
    args: &[JsValue],
    context: &mut Context,
) -> JsResult<JsValue> {
    let target = args.get_or_undefined(0).to_number(context)?;
    // For simplicity, just set the value immediately
    if let Some(this_obj) = this.as_object() {
        this_obj.set(js_string!("value"), JsValue::from(target), false, context)?;
    }
    Ok(this.clone())
}

fn param_set_value_curve_at_time(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    // For simplicity, this is a no-op
    Ok(this.clone())
}

fn param_cancel_scheduled_values(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    Ok(this.clone())
}

fn param_cancel_and_hold_at_time(
    this: &JsValue,
    _args: &[JsValue],
    _context: &mut Context,
) -> JsResult<JsValue> {
    Ok(this.clone())
}
