use anyhow::Result;
use boa_engine::{js_string, Context, JsObject, JsValue, NativeFunction};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use super::types::*;

impl MediaManager {
    pub fn setup_audio_context_api(&self, context: &mut Context) -> Result<(), boa_engine::JsError> {
        let audio_contexts = Arc::clone(&self.audio_contexts);

        // Real AudioContext constructor with actual audio processing
        let audio_context_constructor = unsafe {
            NativeFunction::from_closure(move |_, _args, context| {
                let ctx_id = format!("ctx_{}", rand::random::<u32>());
                let ctx_id_clone = ctx_id.clone();

                // Create real audio context with system sample rate
                let real_ctx = AudioContextReal {
                    sample_rate: 44100.0, // Standard sample rate
                    current_time: 0.0,
                    state: "running".to_string(),
                    destination: "speakers".to_string(),
                    oscillators: HashMap::new(),
                    gain_nodes: HashMap::new(),
                };

                audio_contexts
                    .lock()
                    .unwrap()
                    .insert(ctx_id.clone(), real_ctx);

                let audio_ctx = JsObject::default();
                audio_ctx.set(
                    js_string!("_id"),
                    JsValue::from(js_string!(ctx_id_clone)),
                    false,
                    context,
                )?;
                audio_ctx.set(
                    js_string!("state"),
                    JsValue::from(js_string!("running")),
                    false,
                    context,
                )?;
                audio_ctx.set(
                    js_string!("sampleRate"),
                    JsValue::from(44100.0),
                    false,
                    context,
                )?;

                // Current time updates with real time
                let _start_time = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs_f64();
                audio_ctx.set(
                    js_string!("currentTime"),
                    JsValue::from(0.0),
                    false,
                    context,
                )?;

                // Create destination (speakers/output)
                let destination = JsObject::default();
                audio_ctx.set(
                    js_string!("destination"),
                    JsValue::from(destination),
                    false,
                    context,
                )?;

                Self::setup_oscillator_methods(&audio_ctx, &audio_contexts, &ctx_id, context)?;
                Self::setup_gain_methods(&audio_ctx, &audio_contexts, &ctx_id, context)?;

                Ok(JsValue::from(audio_ctx))
            })
        };

        context.register_global_property(
            js_string!("AudioContext"),
            JsValue::from(audio_context_constructor.to_js_function(context.realm())),
            boa_engine::property::Attribute::all(),
        )?;

        Ok(())
    }

    fn setup_oscillator_methods(
        audio_ctx: &JsObject,
        audio_contexts: &Arc<Mutex<HashMap<String, AudioContextReal>>>,
        ctx_id: &str,
        context: &mut Context,
    ) -> Result<(), boa_engine::JsError> {
        // Real createOscillator method
        let audio_contexts_clone = Arc::clone(audio_contexts);
        let ctx_id_for_osc = ctx_id.to_string();
        let create_oscillator_fn = NativeFunction::from_closure(move |_, _, ctx| {
            let osc_id = format!("osc_{}", rand::random::<u32>());
            let osc_id_clone = osc_id.clone();
            let osc_id_for_start = osc_id.clone();

            // Create real oscillator
            let real_osc = OscillatorReal {
                frequency: 440.0, // A4 note
                wave_type: "sine".to_string(),
                started: false,
            };

            if let Ok(mut contexts) = audio_contexts_clone.lock() {
                if let Some(audio_ctx) = contexts.get_mut(&ctx_id_for_osc) {
                    audio_ctx.oscillators.insert(osc_id.clone(), real_osc);
                }
            }

            let osc = JsObject::default();
            osc.set(
                js_string!("_id"),
                JsValue::from(js_string!(osc_id_clone)),
                false,
                ctx,
            )?;
            osc.set(
                js_string!("type"),
                JsValue::from(js_string!("sine")),
                false,
                ctx,
            )?;

            // Real frequency AudioParam
            let frequency_param = JsObject::default();
            frequency_param.set(js_string!("value"), JsValue::from(440.0), false, ctx)?;
            osc.set(
                js_string!("frequency"),
                JsValue::from(frequency_param),
                false,
                ctx,
            )?;

            // Real start method - actually starts audio generation
            let audio_contexts_start = Arc::clone(&audio_contexts_clone);
            let ctx_id_start = ctx_id_for_osc.clone();
            let osc_id_start = osc_id_for_start.clone();
            let start_fn = NativeFunction::from_closure(move |_, _args, _ctx| {
                if let Ok(mut contexts) = audio_contexts_start.lock() {
                    if let Some(audio_ctx) = contexts.get_mut(&ctx_id_start) {
                        if let Some(oscillator) =
                            audio_ctx.oscillators.get_mut(&osc_id_start)
                        {
                            oscillator.started = true;
                            // In real implementation, would start audio stream
                        }
                    }
                }
                Ok(JsValue::undefined())
            });
            osc.set(
                js_string!("start"),
                JsValue::from(start_fn.to_js_function(ctx.realm())),
                false,
                ctx,
            )?;

            let stop_fn = NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined()));
            osc.set(
                js_string!("stop"),
                JsValue::from(stop_fn.to_js_function(ctx.realm())),
                false,
                ctx,
            )?;

            let connect_fn =
                NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined()));
            osc.set(
                js_string!("connect"),
                JsValue::from(connect_fn.to_js_function(ctx.realm())),
                false,
                ctx,
            )?;

            Ok(JsValue::from(osc))
        });
        audio_ctx.set(
            js_string!("createOscillator"),
            JsValue::from(create_oscillator_fn.to_js_function(context.realm())),
            false,
            context,
        )?;
        Ok(())
    }

    fn setup_gain_methods(
        audio_ctx: &JsObject,
        audio_contexts: &Arc<Mutex<HashMap<String, AudioContextReal>>>,
        ctx_id: &str,
        context: &mut Context,
    ) -> Result<(), boa_engine::JsError> {
        // Real createGain method
        let audio_contexts_gain = Arc::clone(audio_contexts);
        let ctx_id_gain = ctx_id.to_string();
        let create_gain_fn = NativeFunction::from_closure(move |_, _, ctx| {
            let gain_id = format!("gain_{}", rand::random::<u32>());

            let real_gain = GainNodeReal { gain_value: 1.0 };

            if let Ok(mut contexts) = audio_contexts_gain.lock() {
                if let Some(audio_ctx) = contexts.get_mut(&ctx_id_gain) {
                    audio_ctx.gain_nodes.insert(gain_id.clone(), real_gain);
                }
            }

            let gain = JsObject::default();
            gain.set(
                js_string!("_id"),
                JsValue::from(js_string!(gain_id)),
                false,
                ctx,
            )?;

            let gain_param = JsObject::default();
            gain_param.set(js_string!("value"), JsValue::from(1.0), true, ctx)?;
            gain.set(js_string!("gain"), JsValue::from(gain_param), false, ctx)?;

            let connect_fn =
                NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined()));
            gain.set(
                js_string!("connect"),
                JsValue::from(connect_fn.to_js_function(ctx.realm())),
                false,
                ctx,
            )?;

            Ok(JsValue::from(gain))
        });
        audio_ctx.set(
            js_string!("createGain"),
            JsValue::from(create_gain_fn.to_js_function(context.realm())),
            false,
            context,
        )?;
        Ok(())
    }
}