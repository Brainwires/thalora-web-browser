use anyhow::Result;
use boa_engine::{Context, JsObject, JsValue, NativeFunction, js_string, property::Attribute};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Media API implementation for full browser compatibility (Audio, Video, MediaStream, etc.)
pub struct MediaManager {
    audio_contexts: Arc<Mutex<HashMap<String, AudioContext>>>,
    media_streams: Arc<Mutex<HashMap<String, MediaStream>>>,
}

#[derive(Debug, Clone)]
pub struct AudioContext {
    pub state: String,
    pub sample_rate: f64,
    pub current_time: f64,
}

#[derive(Debug, Clone)]
pub struct MediaStream {
    pub id: String,
    pub active: bool,
    pub tracks: Vec<String>,
}

impl MediaManager {
    pub fn new() -> Self {
        Self {
            audio_contexts: Arc::new(Mutex::new(HashMap::new())),
            media_streams: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Setup comprehensive Media APIs in global scope
    pub fn setup_media_apis(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        self.setup_audio_api(context)?;
        self.setup_video_api(context)?;
        self.setup_media_recorder_api(context)?;
        self.setup_speech_apis(context)?;
        Ok(())
    }

    /// Setup Web Audio API
    fn setup_audio_api(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        // AudioContext constructor
        let audio_context_constructor = unsafe { NativeFunction::from_closure({
            let contexts = Arc::clone(&self.audio_contexts);
            move |_, args, context| {
                let audio_ctx = JsObject::default();

                // AudioContext properties
                audio_ctx.set(js_string!("state"), JsValue::from(js_string!("running")), false, context)?;
                audio_ctx.set(js_string!("sampleRate"), JsValue::from(44100.0), false, context)?;
                audio_ctx.set(js_string!("currentTime"), JsValue::from(0.0), false, context)?;
                audio_ctx.set(js_string!("destination"), JsValue::from(JsObject::default()), false, context)?;

                // AudioContext methods
                let create_oscillator_fn = unsafe { NativeFunction::from_closure(|_, _, ctx| {
                    let osc = JsObject::default();
                    osc.set(js_string!("type"), JsValue::from(js_string!("sine")), true, ctx)?;
                    osc.set(js_string!("frequency"), JsValue::from(JsObject::default()), false, ctx)?;

                    let start_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    osc.set(js_string!("start"), JsValue::from(start_fn.to_js_function(ctx.realm())), false, ctx)?;

                    let stop_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    osc.set(js_string!("stop"), JsValue::from(stop_fn.to_js_function(ctx.realm())), false, ctx)?;

                    let connect_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    osc.set(js_string!("connect"), JsValue::from(connect_fn.to_js_function(ctx.realm())), false, ctx)?;

                    Ok(JsValue::from(osc))
                }) };
                audio_ctx.set(js_string!("createOscillator"), JsValue::from(create_oscillator_fn.to_js_function(context.realm())), false, context)?;

                let create_gain_fn = unsafe { NativeFunction::from_closure(|_, _, ctx| {
                    let gain = JsObject::default();
                    let gain_param = JsObject::default();
                    gain_param.set(js_string!("value"), JsValue::from(1.0), true, ctx)?;
                    gain.set(js_string!("gain"), JsValue::from(gain_param), false, ctx)?;

                    let connect_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    gain.set(js_string!("connect"), JsValue::from(connect_fn.to_js_function(ctx.realm())), false, ctx)?;

                    Ok(JsValue::from(gain))
                }) };
                audio_ctx.set(js_string!("createGain"), JsValue::from(create_gain_fn.to_js_function(context.realm())), false, context)?;

                let decode_audio_data_fn = unsafe { NativeFunction::from_closure(|_, args, ctx| {
                    let promise_obj = JsObject::default();

                    let then_fn = unsafe { NativeFunction::from_closure(move |_, callback_args, callback_ctx| {
                        if !callback_args.is_empty() && callback_args[0].is_callable() {
                            let audio_buffer = JsObject::default();
                            audio_buffer.set(js_string!("length"), JsValue::from(44100), false, callback_ctx)?;
                            audio_buffer.set(js_string!("sampleRate"), JsValue::from(44100.0), false, callback_ctx)?;
                            audio_buffer.set(js_string!("numberOfChannels"), JsValue::from(2), false, callback_ctx)?;

                            if let Some(callback) = callback_args[0].as_callable() {
                                let _ = callback.call(&JsValue::undefined(), &[JsValue::from(audio_buffer)], callback_ctx);
                            }
                        }
                        Ok(JsValue::undefined())
                    }) };
                    promise_obj.set(js_string!("then"), JsValue::from(then_fn.to_js_function(ctx.realm())), false, ctx)?;

                    let catch_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                    promise_obj.set(js_string!("catch"), JsValue::from(catch_fn.to_js_function(ctx.realm())), false, ctx)?;

                    Ok(JsValue::from(promise_obj))
                }) };
                audio_ctx.set(js_string!("decodeAudioData"), JsValue::from(decode_audio_data_fn.to_js_function(context.realm())), false, context)?;

                Ok(JsValue::from(audio_ctx))
            }
        }) };
        context.register_global_property(js_string!("AudioContext"), JsValue::from(audio_context_constructor.to_js_function(context.realm())), Attribute::all())?;

        // Audio constructor
        let audio_constructor = unsafe { NativeFunction::from_closure(|_, args, context| {
            let audio = JsObject::default();

            let src = if !args.is_empty() {
                args[0].to_string(context)?.to_std_string_escaped()
            } else {
                "".to_string()
            };

            audio.set(js_string!("src"), JsValue::from(js_string!(src)), true, context)?;
            audio.set(js_string!("currentTime"), JsValue::from(0.0), true, context)?;
            audio.set(js_string!("duration"), JsValue::from(0.0), false, context)?;
            audio.set(js_string!("paused"), JsValue::from(true), false, context)?;
            audio.set(js_string!("volume"), JsValue::from(1.0), true, context)?;
            audio.set(js_string!("muted"), JsValue::from(false), true, context)?;

            let play_fn = unsafe { NativeFunction::from_closure(|_, _, ctx| {
                let promise_obj = JsObject::default();
                let then_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                promise_obj.set(js_string!("then"), JsValue::from(then_fn.to_js_function(ctx.realm())), false, ctx)?;
                Ok(JsValue::from(promise_obj))
            }) };
            audio.set(js_string!("play"), JsValue::from(play_fn.to_js_function(context.realm())), false, context)?;

            let pause_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
            audio.set(js_string!("pause"), JsValue::from(pause_fn.to_js_function(context.realm())), false, context)?;

            Ok(JsValue::from(audio))
        }) };
        context.register_global_property(js_string!("Audio"), JsValue::from(audio_constructor.to_js_function(context.realm())), Attribute::all())?;

        Ok(())
    }

    /// Setup HTML5 Video API
    fn setup_video_api(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        // HTMLVideoElement properties and methods are typically added to DOM elements
        // This sets up the video codec detection and capabilities

        let navigator_obj = context.global_object().get(js_string!("navigator"), context)?;
        if let Some(nav_obj) = navigator_obj.as_object() {
            // Check if mediaDevices exists, if not create it
            let media_devices = if let Ok(existing) = nav_obj.get(js_string!("mediaDevices"), context) {
                if let Some(obj) = existing.as_object() {
                    obj.clone()
                } else {
                    JsObject::default()
                }
            } else {
                JsObject::default()
            };

            let get_display_media_fn = unsafe { NativeFunction::from_closure(|_, args, ctx| {
                let promise_obj = JsObject::default();

                let then_fn = unsafe { NativeFunction::from_closure(move |_, callback_args, callback_ctx| {
                    if !callback_args.is_empty() && callback_args[0].is_callable() {
                        let stream_obj = JsObject::default();
                        stream_obj.set(js_string!("id"), JsValue::from(js_string!("screen-capture-stream")), false, callback_ctx)?;
                        stream_obj.set(js_string!("active"), JsValue::from(true), false, callback_ctx)?;

                        if let Some(callback) = callback_args[0].as_callable() {
                            let _ = callback.call(&JsValue::undefined(), &[JsValue::from(stream_obj)], callback_ctx);
                        }
                    }
                    Ok(JsValue::undefined())
                }) };
                promise_obj.set(js_string!("then"), JsValue::from(then_fn.to_js_function(ctx.realm())), false, ctx)?;

                let catch_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
                promise_obj.set(js_string!("catch"), JsValue::from(catch_fn.to_js_function(ctx.realm())), false, ctx)?;

                Ok(JsValue::from(promise_obj))
            }) };
            media_devices.set(js_string!("getDisplayMedia"), JsValue::from(get_display_media_fn.to_js_function(context.realm())), false, context)?;

            nav_obj.set(js_string!("mediaDevices"), JsValue::from(media_devices), false, context)?;
        }

        Ok(())
    }

    /// Setup MediaRecorder API
    fn setup_media_recorder_api(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        let media_recorder_constructor = unsafe { NativeFunction::from_closure(|_, args, context| {
            let recorder = JsObject::default();

            recorder.set(js_string!("state"), JsValue::from(js_string!("inactive")), false, context)?;
            recorder.set(js_string!("mimeType"), JsValue::from(js_string!("video/webm")), false, context)?;

            let start_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
            recorder.set(js_string!("start"), JsValue::from(start_fn.to_js_function(context.realm())), false, context)?;

            let stop_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
            recorder.set(js_string!("stop"), JsValue::from(stop_fn.to_js_function(context.realm())), false, context)?;

            let pause_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
            recorder.set(js_string!("pause"), JsValue::from(pause_fn.to_js_function(context.realm())), false, context)?;

            let resume_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
            recorder.set(js_string!("resume"), JsValue::from(resume_fn.to_js_function(context.realm())), false, context)?;

            // Event handlers
            recorder.set(js_string!("ondataavailable"), JsValue::null(), true, context)?;
            recorder.set(js_string!("onstop"), JsValue::null(), true, context)?;
            recorder.set(js_string!("onstart"), JsValue::null(), true, context)?;

            Ok(JsValue::from(recorder))
        }) };
        context.register_global_property(js_string!("MediaRecorder"), JsValue::from(media_recorder_constructor.to_js_function(context.realm())), Attribute::all())?;

        // MediaRecorder.isTypeSupported static method
        let is_type_supported_fn = unsafe { NativeFunction::from_closure(|_, args, _| {
            Ok(JsValue::from(true)) // Support all types for compatibility
        }) };

        if let Ok(media_recorder_obj) = context.global_object().get(js_string!("MediaRecorder"), context) {
            if let Some(mr_obj) = media_recorder_obj.as_object() {
                mr_obj.set(js_string!("isTypeSupported"), JsValue::from(is_type_supported_fn.to_js_function(context.realm())), false, context)?;
            }
        }

        Ok(())
    }

    /// Setup Speech APIs (SpeechSynthesis, SpeechRecognition)
    fn setup_speech_apis(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        // SpeechSynthesis in window
        let speech_synthesis_obj = JsObject::default();

        speech_synthesis_obj.set(js_string!("speaking"), JsValue::from(false), false, context)?;
        speech_synthesis_obj.set(js_string!("pending"), JsValue::from(false), false, context)?;
        speech_synthesis_obj.set(js_string!("paused"), JsValue::from(false), false, context)?;

        let speak_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
        speech_synthesis_obj.set(js_string!("speak"), JsValue::from(speak_fn.to_js_function(context.realm())), false, context)?;

        let cancel_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
        speech_synthesis_obj.set(js_string!("cancel"), JsValue::from(cancel_fn.to_js_function(context.realm())), false, context)?;

        let get_voices_fn = unsafe { NativeFunction::from_closure(|_, _, ctx| {
            let voices_array = JsObject::default();
            voices_array.set(js_string!("length"), JsValue::from(0), false, ctx)?;
            Ok(JsValue::from(voices_array))
        }) };
        speech_synthesis_obj.set(js_string!("getVoices"), JsValue::from(get_voices_fn.to_js_function(context.realm())), false, context)?;

        context.register_global_property(js_string!("speechSynthesis"), speech_synthesis_obj, Attribute::all())?;

        // SpeechSynthesisUtterance constructor
        let speech_utterance_constructor = unsafe { NativeFunction::from_closure(|_, args, context| {
            let utterance = JsObject::default();

            let text = if !args.is_empty() {
                args[0].to_string(context)?.to_std_string_escaped()
            } else {
                "".to_string()
            };

            utterance.set(js_string!("text"), JsValue::from(js_string!(text)), true, context)?;
            utterance.set(js_string!("lang"), JsValue::from(js_string!("en-US")), true, context)?;
            utterance.set(js_string!("volume"), JsValue::from(1.0), true, context)?;
            utterance.set(js_string!("rate"), JsValue::from(1.0), true, context)?;
            utterance.set(js_string!("pitch"), JsValue::from(1.0), true, context)?;

            Ok(JsValue::from(utterance))
        }) };
        context.register_global_property(js_string!("SpeechSynthesisUtterance"), JsValue::from(speech_utterance_constructor.to_js_function(context.realm())), Attribute::all())?;

        // SpeechRecognition constructor (webkit prefixed for compatibility)
        let speech_recognition_constructor = unsafe { NativeFunction::from_closure(|_, _, context| {
            let recognition = JsObject::default();

            recognition.set(js_string!("continuous"), JsValue::from(false), true, context)?;
            recognition.set(js_string!("interimResults"), JsValue::from(false), true, context)?;
            recognition.set(js_string!("lang"), JsValue::from(js_string!("en-US")), true, context)?;
            recognition.set(js_string!("maxAlternatives"), JsValue::from(1), true, context)?;

            let start_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
            recognition.set(js_string!("start"), JsValue::from(start_fn.to_js_function(context.realm())), false, context)?;

            let stop_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
            recognition.set(js_string!("stop"), JsValue::from(stop_fn.to_js_function(context.realm())), false, context)?;

            Ok(JsValue::from(recognition))
        }) };
        let speech_fn = JsValue::from(speech_recognition_constructor.to_js_function(context.realm()));
        context.register_global_property(js_string!("SpeechRecognition"), speech_fn.clone(), Attribute::all())?;
        context.register_global_property(js_string!("webkitSpeechRecognition"), speech_fn, Attribute::all())?;

        Ok(())
    }
}