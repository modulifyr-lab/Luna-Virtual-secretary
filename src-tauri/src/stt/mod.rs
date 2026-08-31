pub mod audio;

use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex as AsyncMutex;
use tauri::{AppHandle, Manager};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use crate::config::{self, AppConfig};
use audio::AudioRecorder;

pub struct WhisperStt {
    model_path: String,
    context: AsyncMutex<Option<(WhisperContext, Instant)>>,
    idle_timeout: Duration,
}

impl WhisperStt {
    pub fn new(model_path: impl Into<String>) -> Self {
        Self {
            model_path: model_path.into(),
            context: AsyncMutex::new(None),
            idle_timeout: Duration::from_secs(120), // Unload after 2 minutes of inactivity
        }
    }

    pub async fn ensure_loaded(&self) -> Result<(), String> {
        let mut guard = self.context.lock().await;
        if guard.is_none() {
            if !std::path::Path::new(&self.model_path).exists() {
                return Err(format!("Whisper model file not found at path: {}", self.model_path));
            }
            let ctx = WhisperContext::new_with_params(&self.model_path, WhisperContextParameters::default())
                .map_err(|e| format!("Failed to load Whisper context: {:?}", e))?;
            *guard = Some((ctx, Instant::now()));
        } else if let Some((_, ref mut last_used)) = *guard {
            *last_used = Instant::now();
        }
        Ok(())
    }

    pub async fn transcribe_audio(&self, audio_samples: &[f32]) -> Result<String, String> {
        if audio_samples.is_empty() {
            return Ok(String::new());
        }

        self.ensure_loaded().await?;

        let mut guard = self.context.lock().await;
        let (ref ctx, ref mut last_used) = guard.as_mut().ok_or("Whisper context not loaded")?;
        *last_used = Instant::now();

        let mut state = ctx.create_state().map_err(|e| format!("Failed to create Whisper state: {:?}", e))?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some("en"));
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        state.full(params, audio_samples).map_err(|e| format!("Whisper transcription failed: {:?}", e))?;

        let num_segments = state.full_n_segments().map_err(|e| format!("Failed to get segment count: {:?}", e))?;
        let mut text = String::new();
        for i in 0..num_segments {
            if let Ok(segment) = state.full_get_segment_text(i) {
                text.push_str(&segment);
            }
        }

        Ok(text.trim().to_string())
    }

    pub async fn cleanup_if_idle(&self) {
        let mut guard = self.context.lock().await;
        if let Some((_, last_used)) = *guard {
            if last_used.elapsed() >= self.idle_timeout {
                *guard = None;
                println!("[STT] Whisper model unloaded from RAM due to inactivity.");
            }
        }
    }
}

pub fn setup_hotkey_and_stt(app_handle: AppHandle, config: &AppConfig) -> Result<Arc<WhisperStt>, String> {
    let whisper_stt = Arc::new(WhisperStt::new(&config.whisper_model_path));
    let hotkey = config::parse_hotkey(&config.hotkey_binding)?;

    let manager = GlobalHotKeyManager::new().map_err(|e| format!("Failed to init GlobalHotKeyManager: {:?}", e))?;
    manager.register(hotkey).map_err(|e| format!("Failed to register hotkey {}: {:?}", config.hotkey_binding, e))?;

    let rx = GlobalHotKeyEvent::receiver();
    let hotkey_id = hotkey.id();
    let whisper_clone = Arc::clone(&whisper_stt);
    let app_handle_clone = app_handle.clone();

    std::thread::spawn(move || {
        let _manager = manager;
        let mut recorder = AudioRecorder::new();
        let mut is_recording = false;

        while let Ok(event) = rx.recv() {
            if event.id() == hotkey_id {
                if !is_recording {
                    is_recording = true;
                    let _ = app_handle_clone.emit_all("stt-state-changed", "listening");
                    if let Err(e) = recorder.start() {
                        eprintln!("[STT] Failed to start audio recorder: {}", e);
                        is_recording = false;
                        let _ = app_handle_clone.emit_all("stt-state-changed", "idle");
                    }
                } else {
                    is_recording = false;
                    let _ = app_handle_clone.emit_all("stt-state-changed", "transcribing");
                    let samples = recorder.stop();

                    let whisper = Arc::clone(&whisper_clone);
                    let app_handle = app_handle_clone.clone();

                    tauri::async_runtime::spawn(async move {
                        match whisper.transcribe_audio(&samples).await {
                            Ok(text) => {
                                let _ = app_handle.emit_all("stt-transcribed-text", &text);
                            }
                            Err(e) => {
                                eprintln!("[STT] Transcription error: {}", e);
                                let _ = app_handle.emit_all("stt-transcribed-text", format!("[Error: {}]", e));
                            }
                        }
                        let _ = app_handle.emit_all("stt-state-changed", "idle");
                    });
                }
            }
        }
    });

    let whisper_cleanup = Arc::clone(&whisper_stt);
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            whisper_cleanup.cleanup_if_idle().await;
        }
    });

    Ok(whisper_stt)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_whisper_stt_initial_state_not_loaded() {
        let stt = WhisperStt::new("models/nonexistent.bin");
        let guard = stt.context.lock().await;
        assert!(guard.is_none(), "WhisperContext should not be loaded on startup");
    }

    #[tokio::test]
    async fn test_whisper_stt_missing_file_error() {
        let stt = WhisperStt::new("models/nonexistent.bin");
        let result = stt.transcribe_audio(&[0.0; 100]).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
