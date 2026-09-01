use std::sync::Arc;
use tauri::{command, AppHandle, Manager, State};
use crate::brain::{self, BrainResponse};
use crate::config::AppConfig;
use crate::tts::KokoroTts;

#[command]
pub async fn process_user_input(
    input: String,
    app_handle: AppHandle,
    config: State<'_, AppConfig>,
    tts: State<'_, Arc<KokoroTts>>,
) -> Result<BrainResponse, String> {
    let brain_res = brain::get_response(&input, &config).await?;

    let tts_clone = Arc::clone(&tts);
    let speech_text = brain_res.response.clone();
    let app_handle_clone = app_handle.clone();

    tauri::async_runtime::spawn(async move {
        let _ = app_handle_clone.emit_all("stt-state-changed", "speaking");
        match tts_clone.synthesize_speech(&speech_text).await {
            Ok(pcm_bytes) => {
                if !pcm_bytes.is_empty() {
                    let _ = tts_clone.play_audio(pcm_bytes);
                }
            }
            Err(e) => {
                eprintln!("[TTS] Speech synthesis error: {}", e);
            }
        }
        let _ = app_handle_clone.emit_all("stt-state-changed", "idle");
    });

    Ok(brain_res)
}

#[command]
pub async fn get_status() -> Result<String, String> {
    if brain::connectivity::check_online_status().await {
        Ok("Online (Groq)".to_string())
    } else {
        Ok("Offline (Ollama)".to_string())
    }
}
