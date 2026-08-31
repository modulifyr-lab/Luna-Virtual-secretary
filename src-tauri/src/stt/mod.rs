use std::path::Path;

pub struct WhisperStt {
    // TODO: Keep model path for on-demand loading rather than resident memory
    model_path: String,
}

impl WhisperStt {
    pub fn new(model_path: impl Into<String>) -> Self {
        Self {
            model_path: model_path.into(),
        }
    }

    pub async fn transcribe_audio(&self, _audio_samples: &[f32]) -> Result<String, String> {
        // TODO: Load whisper-rs context on demand
        // TODO: Run speech-to-text inference on audio samples
        // TODO: Unload/free context to save RAM when done
        Ok("Transcribed text stub".to_string())
    }
}
