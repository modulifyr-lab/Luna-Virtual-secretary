pub mod cloud;
pub mod connectivity;
pub mod local;

use cloud::GroqClient;
use local::OllamaClient;
use crate::config::AppConfig;
use crate::context::ForegroundContext;

/// Personality system prompt for Luna, defined as a constant in one place for easy tuning.
pub const LUNA_SYSTEM_PROMPT: &str = "\
You are Luna, a fiery, highly efficient, witty, and sharp-tongued virtual secretary on Windows. \
You get things done with supreme confidence, directness, and a touch of fiery attitude. \
Keep your answers brief, snappy, helpful, and full of personality.\
";

pub struct BrainRouter {
    groq: GroqClient,
    ollama: OllamaClient,
}

impl BrainRouter {
    pub fn new(groq_key: String, ollama_url: String) -> Self {
        Self {
            groq: GroqClient::new(groq_key),
            ollama: OllamaClient::new(ollama_url),
        }
    }

    pub async fn process_prompt(&self, prompt: &str, is_gpu_heavy_app: bool) -> Result<String, String> {
        if connectivity::check_online_status().await {
            match self.groq.query(prompt, LUNA_SYSTEM_PROMPT).await {
                Ok(response) => return Ok(response),
                Err(err) => {
                    eprintln!("[BrainRouter] Groq Cloud failed ({}), falling back to local Ollama.", err);
                }
            }
        }

        // Offline or Groq fallback: select model based on GPU context
        let model = if is_gpu_heavy_app {
            "phi4-mini"
        } else {
            "llama3:8b"
        };

        self.ollama.query(prompt, model, LUNA_SYSTEM_PROMPT).await
    }
}

/// Routes user input to either Groq (if online) or Ollama (if offline or if Groq fails).
pub async fn get_response(user_text: &str, config: &AppConfig) -> Result<String, String> {
    let router = BrainRouter::new(config.groq_api_key.clone(), config.ollama_base_url.clone());
    let is_gpu_heavy = ForegroundContext::is_gpu_heavy(&config.heavy_gpu_apps);
    router.process_prompt(user_text, is_gpu_heavy).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_brain_router_fallback_to_ollama_on_groq_failure() {
        let router = BrainRouter::new("invalid_key".to_string(), "http://127.0.0.1:59999".to_string());
        let res = router.process_prompt("Hello", false).await;
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Ollama HTTP request failed"));
    }
}
