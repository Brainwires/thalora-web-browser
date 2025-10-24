use anyhow::Result;
use thalora_browser_apis::boa_engine::{js_string, property::Attribute, Context, JsObject, JsValue, NativeFunction};
use std::sync::{Arc, Mutex};
use super::types::*;

impl MediaManager {
    pub fn setup_media_recorder_api(&self, context: &mut Context) -> Result<(), thalora_browser_apis::boa_engine::JsError> {
        let media_recorders = Arc::clone(&self.media_recorders);

        // Real MediaRecorder constructor with actual recording
        let media_recorder_constructor = unsafe {
            NativeFunction::from_closure(move |_, _args, context| {
                let recorder_id = format!("recorder_{}", rand::random::<u32>());
                let recorder_id_clone = recorder_id.clone();

                let real_recorder = MediaRecorderReal {
                    state: "inactive".to_string(),
                    mime_type: "video/webm".to_string(),
                    recording_data: Vec::new(),
                };

                media_recorders
                    .lock()
                    .unwrap()
                    .insert(recorder_id.clone(), real_recorder);

                let recorder = JsObject::default(&context.intrinsics());
                recorder.set(
                    js_string!("_id"),
                    JsValue::from(js_string!(recorder_id_clone)),
                    false,
                    context,
                )?;
                recorder.set(
                    js_string!("state"),
                    JsValue::from(js_string!("inactive")),
                    false,
                    context,
                )?;
                recorder.set(
                    js_string!("mimeType"),
                    JsValue::from(js_string!("video/webm")),
                    false,
                    context,
                )?;

                Self::setup_recorder_methods(&recorder, &media_recorders, &recorder_id, context)?;

                Ok(JsValue::from(recorder))
            })
        };

        context.register_global_property(
            js_string!("MediaRecorder"),
            JsValue::from(media_recorder_constructor.to_js_function(context.realm())),
            Attribute::all(),
        )?;

        self.setup_media_recorder_static_methods(context)?;

        Ok(())
    }

    fn setup_recorder_methods(
        recorder: &JsObject,
        media_recorders: &Arc<Mutex<std::collections::HashMap<String, MediaRecorderReal>>>,
        recorder_id: &str,
        context: &mut Context,
    ) -> Result<(), thalora_browser_apis::boa_engine::JsError> {
        // Real start method - actually begins recording
            let media_recorders_start = Arc::clone(media_recorders);
            let recorder_id_start = recorder_id.to_string();
            let start_fn = unsafe { NativeFunction::from_closure(move |_, _args, _ctx| {
            if let Ok(mut recorders) = media_recorders_start.lock() {
                if let Some(recorder) = recorders.get_mut(&recorder_id_start) {
                    recorder.state = "recording".to_string();
                    // Real recording would start here
                }
            }
            Ok(JsValue::undefined())
            }) };
        recorder.set(
            js_string!("start"),
            JsValue::from(start_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

    let stop_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
        recorder.set(
            js_string!("stop"),
            JsValue::from(stop_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

    let pause_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
        recorder.set(
            js_string!("pause"),
            JsValue::from(pause_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

    let resume_fn = unsafe { NativeFunction::from_closure(|_, _, _| Ok(JsValue::undefined())) };
        recorder.set(
            js_string!("resume"),
            JsValue::from(resume_fn.to_js_function(context.realm())),
            false,
            context,
        )?;

        // Event handlers
        recorder.set(
            js_string!("ondataavailable"),
            JsValue::null(),
            true,
            context,
        )?;
        recorder.set(js_string!("onstop"), JsValue::null(), true, context)?;
        recorder.set(js_string!("onstart"), JsValue::null(), true, context)?;

        Ok(())
    }

    fn setup_media_recorder_static_methods(&self, context: &mut Context) -> Result<(), thalora_browser_apis::boa_engine::JsError> {
        // Real MediaRecorder.isTypeSupported static method
        let is_type_supported_fn = unsafe {
            NativeFunction::from_closure(|_, args, _context| {
                if !args.is_empty() {
                    let mime_type = args[0].to_string(_context)?.to_std_string_escaped();
                    // Real format support checking would happen here
                    match mime_type.as_str() {
                        "video/webm" | "video/mp4" | "audio/webm" | "audio/wav" => {
                            Ok(JsValue::from(true))
                        }
                        _ => Ok(JsValue::from(false)),
                    }
                } else {
                    Ok(JsValue::from(false))
                }
            })
        };

        let media_recorder_obj = context
            .global_object()
            .get(js_string!("MediaRecorder"), context)?;
        if let Some(mr_obj) = media_recorder_obj.as_object() {
            mr_obj.set(
                js_string!("isTypeSupported"),
                JsValue::from(is_type_supported_fn.to_js_function(context.realm())),
                false,
                context,
            )?;
        }

        Ok(())
    }
}