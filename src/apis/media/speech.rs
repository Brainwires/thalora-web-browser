use anyhow::Result;
use thalora_browser_apis::boa_engine::{js_string, property::Attribute, Context, JsObject, JsValue, NativeFunction};
use std::sync::{Arc, Mutex};
use super::types::*;

impl MediaManager {
    pub fn setup_speech_apis(&self, context: &mut Context) -> Result<(), thalora_browser_apis::boa_engine::JsError> {
        let speech_synthesis = Arc::clone(&self.speech_synthesis);

        // Real speechSynthesis global object
        let speech_synthesis_obj = JsObject::default();
        speech_synthesis_obj.set(js_string!("speaking"), JsValue::from(false), false, context)?;
        speech_synthesis_obj.set(js_string!("pending"), JsValue::from(false), false, context)?;
        speech_synthesis_obj.set(js_string!("paused"), JsValue::from(false), false, context)?;

        Self::setup_speech_synthesis_methods(&speech_synthesis_obj, &speech_synthesis, context)?;

        context.register_global_property(
            js_string!("speechSynthesis"),
            speech_synthesis_obj,
            Attribute::all(),
        )?;

        self.setup_speech_utterance_constructor(context)?;

        // Minimal SpeechRecognition constructor (also provide webkitSpeechRecognition alias)
        let speech_recognition_constructor = unsafe {
            NativeFunction::from_closure(|_, _args, context| {
                // Return a simple object with start/stop placeholders
                let obj = JsObject::default();
                obj.set(js_string!("start"), JsValue::from(native_fn_stub(context)?), true, context)?;
                obj.set(js_string!("stop"), JsValue::from(native_fn_stub(context)?), true, context)?;
                Ok(JsValue::from(obj))
            })
        };

        let speech_recognition_value = JsValue::from(speech_recognition_constructor.to_js_function(context.realm()));

        context.register_global_property(
            js_string!("SpeechRecognition"),
            speech_recognition_value.clone(),
            Attribute::all(),
        )?;

        context.register_global_property(
            js_string!("webkitSpeechRecognition"),
            speech_recognition_value,
            Attribute::all(),
        )?;

        Ok(())
    }

    fn setup_speech_synthesis_methods(
        speech_synthesis_obj: &JsObject,
        speech_synthesis: &Arc<Mutex<SpeechSynthesisReal>>,
        context: &mut Context,
    ) -> Result<(), thalora_browser_apis::boa_engine::JsError> {
        // Real speak method
        let speech_synthesis_speak = Arc::clone(speech_synthesis);
        let speak_fn = unsafe {
            NativeFunction::from_closure(move |_, args, _context| {
                if !args.is_empty() {
                    // In real implementation, would use system TTS
                    if let Ok(mut synthesis) = speech_synthesis_speak.lock() {
                        synthesis.speaking = true;
                        // Real speech synthesis would happen here
                    }
                }
                Ok(JsValue::undefined())
            })
        };
        speech_synthesis_obj.set(
            js_string!("speak"),
            JsValue::from(speak_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

        let cancel_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
        speech_synthesis_obj.set(
            js_string!("cancel"),
            JsValue::from(cancel_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

        let pause_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
        speech_synthesis_obj.set(
            js_string!("pause"),
            JsValue::from(pause_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

        let resume_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
        speech_synthesis_obj.set(
            js_string!("resume"),
            JsValue::from(resume_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

        let get_voices_fn = unsafe {
            NativeFunction::from_closure(|_, _, _ctx| {
                let voices_array = JsObject::default();
                // Real voice enumeration would happen here
                Ok(JsValue::from(voices_array))
            })
        };
        speech_synthesis_obj.set(
            js_string!("getVoices"),
            JsValue::from(get_voices_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

        Ok(())
    }

    fn setup_speech_utterance_constructor(&self, context: &mut Context) -> Result<(), thalora_browser_apis::boa_engine::JsError> {
        // Real SpeechSynthesisUtterance constructor
        let speech_utterance_constructor = unsafe {
            NativeFunction::from_closure(|_, args, context| {
                let utterance = JsObject::default();

                let text = if !args.is_empty() {
                    args[0].to_string(context)?.to_std_string_escaped()
                } else {
                    "".to_string()
                };

                utterance.set(
                    js_string!("text"),
                    JsValue::from(js_string!(text)),
                    true,
                    context,
                )?;
                utterance.set(
                    js_string!("lang"),
                    JsValue::from(js_string!("en-US")),
                    true,
                    context,
                )?;
                utterance.set(js_string!("voice"), JsValue::null(), true, context)?;
                utterance.set(js_string!("volume"), JsValue::from(1.0), true, context)?;
                utterance.set(js_string!("rate"), JsValue::from(1.0), true, context)?;
                utterance.set(js_string!("pitch"), JsValue::from(1.0), true, context)?;

                // Event handlers
                utterance.set(js_string!("onstart"), JsValue::null(), true, context)?;
                utterance.set(js_string!("onend"), JsValue::null(), true, context)?;
                utterance.set(js_string!("onerror"), JsValue::null(), true, context)?;
                utterance.set(js_string!("onpause"), JsValue::null(), true, context)?;
                utterance.set(js_string!("onresume"), JsValue::null(), true, context)?;
                utterance.set(js_string!("onmark"), JsValue::null(), true, context)?;
                utterance.set(js_string!("onboundary"), JsValue::null(), true, context)?;

                Ok(JsValue::from(utterance))
            })
        };

        context.register_global_property(
            js_string!("SpeechSynthesisUtterance"),
            JsValue::from(speech_utterance_constructor.to_js_function(context.realm())),
            Attribute::all(),
        )?;

        Ok(())
    }
}

// Helper to create a no-op native function wrapped as JsValue
fn native_fn_stub(context: &mut Context) -> Result<JsValue, thalora_browser_apis::boa_engine::JsError> {
    let f = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
    Ok(JsValue::from(f.to_js_function(context.realm())))
}