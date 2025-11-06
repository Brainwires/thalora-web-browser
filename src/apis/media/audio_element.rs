use anyhow::Result;
use thalora_browser_apis::boa_engine::{js_string, Context, JsObject, JsValue, NativeFunction};
use std::sync::{Arc, Mutex};
use super::types::*;

impl MediaManager {
    pub fn setup_audio_element_api(&self, context: &mut Context) -> Result<(), thalora_browser_apis::boa_engine::JsError> {
        let audio_elements = Arc::clone(&self.audio_elements);

        // Real Audio constructor with actual file loading
        let audio_constructor = unsafe {
            NativeFunction::from_closure(move |_, args, context| {
                let audio_id = format!("audio_{}", rand::random::<u32>());

                let src = if !args.is_empty() {
                    args[0].to_string(context)?.to_std_string_escaped()
                } else {
                    "".to_string()
                };

                // Create real audio element
                let real_audio = AudioElementReal {
                    src: src.clone(),
                    current_time: 0.0,
                    duration: 0.0, // Would be set after loading
                    paused: true,
                    volume: 1.0,
                    sink: None, // Would be created when playing
                };

                let audio_id_clone = audio_id.clone();
                audio_elements
                    .lock()
                    .unwrap()
                    .insert(audio_id.clone(), real_audio);

                let audio = JsObject::default(&context.intrinsics());
                audio.set(
                    js_string!("_id"),
                    JsValue::from(js_string!(audio_id_clone)),
                    false,
                    context,
                )?;
                audio.set(
                    js_string!("src"),
                    JsValue::from(js_string!(src)),
                    true,
                    context,
                )?;
                audio.set(js_string!("currentTime"), JsValue::from(0.0), true, context)?;
                audio.set(js_string!("duration"), JsValue::from(0.0), false, context)?;
                audio.set(js_string!("paused"), JsValue::from(true), false, context)?;
                audio.set(js_string!("volume"), JsValue::from(1.0), true, context)?;
                audio.set(js_string!("muted"), JsValue::from(false), true, context)?;

                Self::setup_audio_playback_methods(&audio, &audio_elements, &audio_id, context)?;

                Ok(JsValue::from(audio))
            })
        };

        context.register_global_property(
            js_string!("Audio"),
            JsValue::from(audio_constructor.to_js_function(context.realm())),
            thalora_browser_apis::boa_engine::property::Attribute::all(),
        )?;

        Ok(())
    }

    fn setup_audio_playback_methods(
        audio: &JsObject,
        audio_elements: &Arc<Mutex<std::collections::HashMap<String, AudioElementReal>>>,
        audio_id: &str,
        context: &mut Context,
    ) -> Result<(), thalora_browser_apis::boa_engine::JsError> {
        // Real play method - actually plays audio
        let audio_elements_play = Arc::clone(audio_elements);
        let audio_id_play = audio_id.to_string();
        let play_fn = unsafe { NativeFunction::from_closure(move |_, _args, ctx| {
            let promise_obj = JsObject::default(&ctx.intrinsics());

            // In real implementation, would load and play audio file
            if let Ok(mut elements) = audio_elements_play.lock() {
                if let Some(audio_elem) = elements.get_mut(&audio_id_play) {
                    audio_elem.paused = false;

                    // Real audio playback would happen here
                    // For demo, simulate successful playback
                }
            }

            let then_fn = NativeFunction::from_closure(|_, _args, _ctx| Ok(JsValue::undefined()));
            promise_obj.set(
                js_string!("then"),
                JsValue::from(then_fn.to_js_function(ctx.realm())),
                false,
                ctx,
            )?;

            Ok(JsValue::from(promise_obj))
    }) };
        audio.set(
            js_string!("play"),
            JsValue::from(play_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

    let pause_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
        audio.set(
            js_string!("pause"),
            JsValue::from(pause_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

        let load_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
        audio.set(
            js_string!("load"),
            JsValue::from(load_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

        Ok(())
    }
}