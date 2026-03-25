use super::types::*;
use anyhow::Result;
use rodio::DeviceTrait;
use thalora_browser_apis::boa_engine::Context;

impl MediaManager {
    /// Setup real Media APIs in global scope
    pub fn setup_media_apis(
        &self,
        context: &mut Context,
    ) -> Result<(), thalora_browser_apis::boa_engine::JsError> {
        self.setup_audio_context_api(context)?;
        self.setup_audio_element_api(context)?;
        self.setup_media_recorder_api(context)?;
        self.setup_speech_apis(context)?;
        Ok(())
    }

    /// Get audio device information
    pub fn get_audio_devices(&self) -> Vec<String> {
        self.audio_devices
            .iter()
            .map(|device| format!("{:?}", device.name()))
            .collect()
    }

    /// Create a new audio context
    pub fn create_audio_context(&self) -> String {
        let ctx_id = format!("ctx_{}", rand::random::<u32>());
        let real_ctx = AudioContextReal {
            sample_rate: 44100.0,
            current_time: 0.0,
            state: "running".to_string(),
            destination: "speakers".to_string(),
            oscillators: std::collections::HashMap::new(),
            gain_nodes: std::collections::HashMap::new(),
        };

        self.audio_contexts
            .lock()
            .unwrap()
            .insert(ctx_id.clone(), real_ctx);
        ctx_id
    }

    /// Get audio context by ID
    pub fn get_audio_context(&self, ctx_id: &str) -> Option<AudioContextReal> {
        self.audio_contexts.lock().unwrap().get(ctx_id).cloned()
    }

    /// Create a new audio element
    pub fn create_audio_element(&self, src: &str) -> String {
        let audio_id = format!("audio_{}", rand::random::<u32>());
        let real_audio = AudioElementReal {
            src: src.to_string(),
            current_time: 0.0,
            duration: 0.0,
            paused: true,
            volume: 1.0,
            sink: None,
        };

        self.audio_elements
            .lock()
            .unwrap()
            .insert(audio_id.clone(), real_audio);
        audio_id
    }

    /// Get all audio contexts
    pub fn get_all_audio_contexts(&self) -> Vec<String> {
        self.audio_contexts
            .lock()
            .unwrap()
            .keys()
            .cloned()
            .collect()
    }

    /// Clean up audio resources
    pub fn cleanup_audio_resources(&self) {
        self.audio_contexts.lock().unwrap().clear();
        self.audio_elements.lock().unwrap().clear();
        self.media_recorders.lock().unwrap().clear();
    }
}

impl Default for MediaManager {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| MediaManager {
            audio_contexts: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            audio_elements: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            media_recorders: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::HashMap::new(),
            )),
            speech_synthesis: std::sync::Arc::new(std::sync::Mutex::new(SpeechSynthesisReal {
                speaking: false,
                pending: false,
                paused: false,
                voices: vec![],
            })),
            audio_host: cpal::default_host(),
            audio_devices: vec![],
        })
    }
}
