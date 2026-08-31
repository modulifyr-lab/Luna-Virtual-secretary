use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub groq_api_key: String,
    pub hotkey_binding: String,
    pub whisper_model_path: String,
    pub kokoro_model_path: String,
    pub ollama_base_url: String,
    pub heavy_gpu_apps: Vec<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            groq_api_key: String::new(),
            hotkey_binding: "Ctrl+Shift+Space".to_string(),
            whisper_model_path: "models/ggml-base.en.bin".to_string(),
            kokoro_model_path: "models/kokoro-v0_88.onnx".to_string(),
            ollama_base_url: "http://localhost:11434".to_string(),
            heavy_gpu_apps: vec!["minecraft.exe".to_string(), "cyberpunk2077.exe".to_string()],
        }
    }
}

pub fn load_config() -> AppConfig {
    // TODO: Load configuration from file or environment variables
    AppConfig::default()
}
