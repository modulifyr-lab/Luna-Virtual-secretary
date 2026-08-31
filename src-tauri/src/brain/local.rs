pub struct OllamaClient {
    base_url: String,
}

impl OllamaClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
        }
    }

    pub async fn query(&self, prompt: &str, model: &str, _system_prompt: &str) -> Result<String, String> {
        // TODO: Call Ollama local HTTP API (http://localhost:11434/api/generate or /api/chat) via reqwest
        // TODO: Pass requested model (e.g. Llama 3 8B or Phi-4-mini/Qwen 3B/4B depending on GPU heavy status)
        Ok(format!("Ollama local ({}) response stub for: {}", model, prompt))
    }
}
