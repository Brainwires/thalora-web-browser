// Minimal stub for audio_context API to unblock compilation.
use boa_engine::Context;
use super::types::*;

impl MediaManager {
    pub fn setup_audio_context_api(&self, _context: &mut Context) -> std::result::Result<(), boa_engine::JsError> {
        Ok(())
    }
}
