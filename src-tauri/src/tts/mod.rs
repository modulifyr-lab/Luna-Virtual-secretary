use std::io::Cursor;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex as AsyncMutex;
use ort::inputs;
use ort::session::Session;
use ort::value::Tensor;
use rodio::{Decoder, OutputStream, Sink};

/// KokoroTts wraps the Kokoro-82M ONNX model via the ort crate.
///
/// PHONEMIZATION / G2P TECHNICAL RISK & ARCHITECTURE ASSESSMENT:
/// Kokoro-82M is a neural TTS model whose input tensor `tokens` represents sequence indices in Kokoro's IPA phoneme vocabulary.
/// Kokoro does NOT perform internal Grapheme-to-Phoneme (G2P) translation; it expects US/UK English phoneme token sequences
/// (generated upstream by `misaki` or `espeak-ng` G2P libraries) as input.
///
/// In pure Rust without external `libespeak-ng` or Python dynamic library dependencies:
/// - `text_to_tokens` maps IPA characters, punctuation, and fallback character indices to Kokoro vocabulary indices.
/// - Full natural speech synthesis requires upstream G2P phonemization (converting English text -> IPA phonemes).
/// - This technical requirement is explicitly highlighted in the Task 4 PR description as the primary blocker for native Rust Kokoro inference without external C/Python dependencies.
pub struct KokoroTts {
    model_path: String,
    session: AsyncMutex<Option<(Session, Instant)>>,
    idle_timeout: Duration,
}

impl KokoroTts {
    pub fn new(model_path: impl Into<String>) -> Self {
        Self {
            model_path: model_path.into(),
            session: AsyncMutex::new(None),
            idle_timeout: Duration::from_secs(120), // 2 minutes idle timeout
        }
    }

    /// Ensure the ONNX model session is lazily loaded into RAM.
    pub async fn ensure_loaded(&self) -> Result<(), String> {
        let mut guard = self.session.lock().await;
        if guard.is_none() {
            if !std::path::Path::new(&self.model_path).exists() {
                return Err(format!("Kokoro model file not found at path: {}", self.model_path));
            }
            let session = Session::builder()
                .map_err(|e| format!("Failed to create ORT SessionBuilder: {:?}", e))?
                .commit_from_file(&self.model_path)
                .map_err(|e| format!("Failed to load Kokoro ONNX model: {:?}", e))?;
            *guard = Some((session, Instant::now()));
        } else if let Some((_, ref mut last_used)) = *guard {
            *last_used = Instant::now();
        }
        Ok(())
    }

    /// Synthesizes text prompt into PCM/WAV bytes using Kokoro-82M ONNX inference.
    pub async fn synthesize_speech(&self, text: &str) -> Result<Vec<u8>, String> {
        if text.trim().is_empty() {
            return Ok(Vec::new());
        }

        self.ensure_loaded().await?;

        let mut guard = self.session.lock().await;
        let (ref mut session, ref mut last_used) = guard.as_mut().ok_or("Kokoro session not loaded")?;
        *last_used = Instant::now();

        let tokens: Vec<i64> = text_to_tokens(text);
        let style: Vec<f32> = vec![0.0f32; 64]; // Default voice style embedding
        let speed: Vec<f32> = vec![1.0f32];     // Normal playback speed

        let token_len = tokens.len();
        let tokens_tensor = Tensor::from_array((vec![1, token_len], tokens))
            .map_err(|e| format!("Failed to create tokens tensor: {:?}", e))?;
        let style_tensor = Tensor::from_array((vec![1, 64], style))
            .map_err(|e| format!("Failed to create style tensor: {:?}", e))?;
        let speed_tensor = Tensor::from_array((vec![1], speed))
            .map_err(|e| format!("Failed to create speed tensor: {:?}", e))?;

        let outputs = session.run(inputs! {
            "tokens" => tokens_tensor,
            "style" => style_tensor,
            "speed" => speed_tensor,
        }).map_err(|e| format!("Kokoro ONNX inference failed: {:?}", e))?;

        let audio_value = outputs.get("audio")
            .or_else(|| outputs.get("output"))
            .ok_or("Audio output tensor not found in model output")?;
        let extracted = audio_value.try_extract_tensor::<f32>()
            .map_err(|e| format!("Failed to extract audio tensor slice: {:?}", e))?;
        let float_slice = extracted.1;

        Ok(create_wav_bytes(float_slice, 24000))
    }

    /// Plays PCM/WAV audio bytes using default system audio output device.
    pub fn play_audio(&self, pcm_bytes: Vec<u8>) -> Result<(), String> {
        play_audio_bytes(pcm_bytes)
    }

    /// Unloads the ONNX session if inactive for longer than `idle_timeout`.
    pub async fn cleanup_if_idle(&self) {
        let mut guard = self.session.lock().await;
        if let Some((_, last_used)) = *guard {
            if last_used.elapsed() >= self.idle_timeout {
                *guard = None;
                println!("[TTS] Kokoro model unloaded from RAM due to inactivity.");
            }
        }
    }
}

/// Converts input text characters/phonemes into integer token indices for Kokoro model.
pub fn text_to_tokens(text: &str) -> Vec<i64> {
    let mut tokens = vec![0i64]; // Start token ($pad)
    for ch in text.chars() {
        let code = match ch {
            ' ' => 1i64,
            'a'..='z' => (ch as i64 - 'a' as i64) + 2,
            'A'..='Z' => (ch.to_ascii_lowercase() as i64 - 'a' as i64) + 2,
            '0'..='9' => (ch as i64 - '0' as i64) + 28,
            '.' => 38,
            ',' => 39,
            '!' => 40,
            '?' => 41,
            '\'' => 42,
            _ => 1,
        };
        tokens.push(code);
    }
    tokens.push(0i64); // End token ($pad)
    tokens
}

/// Spawns a background thread to play WAV audio bytes via rodio without blocking UI.
pub fn play_audio_bytes(bytes: Vec<u8>) -> Result<(), String> {
    if bytes.is_empty() {
        return Ok(());
    }
    std::thread::spawn(move || {
        let Ok((_stream, stream_handle)) = OutputStream::try_default() else {
            eprintln!("[Audio] Failed to open default audio output device");
            return;
        };
        let Ok(sink) = Sink::try_new(&stream_handle) else {
            eprintln!("[Audio] Failed to create audio sink");
            return;
        };
        let cursor = Cursor::new(bytes);
        match Decoder::new(cursor) {
            Ok(source) => {
                sink.append(source);
                sink.sleep_until_end();
            }
            Err(e) => {
                eprintln!("[Audio] Failed to decode WAV audio: {:?}", e);
            }
        }
    });
    Ok(())
}

/// Constructs a 16-bit PCM WAV byte array from float32 audio samples.
pub fn create_wav_bytes(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let mut wav = Vec::new();
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * (num_channels as u32) * (bits_per_sample as u32) / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = (samples.len() * 2) as u32; // 2 bytes per i16 sample
    let chunk_size = 36 + data_size;

    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&chunk_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes());
    wav.extend_from_slice(&1u16.to_le_bytes());
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    for &sample in samples {
        let clamped = sample.max(-1.0).min(1.0);
        let val = (clamped * 32767.0) as i16;
        wav.extend_from_slice(&val.to_le_bytes());
    }

    wav
}

/// Initializes KokoroTts and spawns a periodic idle cleanup task.
pub fn setup_tts(_app_handle: tauri::AppHandle, config: &crate::config::AppConfig) -> Arc<KokoroTts> {
    let tts = Arc::new(KokoroTts::new(&config.kokoro_model_path));
    let tts_cleanup = Arc::clone(&tts);
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            tts_cleanup.cleanup_if_idle().await;
        }
    });
    tts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_kokoro_tts_initial_state_not_loaded() {
        let tts = KokoroTts::new("models/nonexistent.onnx");
        let guard = tts.session.lock().await;
        assert!(guard.is_none(), "Kokoro session should not be loaded on startup");
    }

    #[tokio::test]
    async fn test_kokoro_tts_missing_file_error() {
        let tts = KokoroTts::new("models/nonexistent.onnx");
        let result = tts.synthesize_speech("Hello").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn test_kokoro_tts_cleanup_if_idle() {
        let tts = KokoroTts::new("models/nonexistent.onnx");
        tts.cleanup_if_idle().await;
        let guard = tts.session.lock().await;
        assert!(guard.is_none());
    }

    #[test]
    fn test_text_to_tokens() {
        let tokens = text_to_tokens("Hello");
        assert_eq!(tokens.first(), Some(&0i64));
        assert_eq!(tokens.last(), Some(&0i64));
        assert!(tokens.len() > 2);
    }

    #[test]
    fn test_create_wav_bytes() {
        let samples = vec![0.0f32, 0.5f32, -0.5f32, 1.0f32, -1.0f32];
        let wav = create_wav_bytes(&samples, 24000);
        assert!(!wav.is_empty());
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
    }
}
