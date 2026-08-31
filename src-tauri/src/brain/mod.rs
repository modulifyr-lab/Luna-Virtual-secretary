pub mod cloud;
pub mod connectivity;
pub mod local;

use cloud::GroqClient;
use local::OllamaClient;

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
        // TODO: System prompt defining fiery secretary personality
        let system_prompt = "You are Luna, a fiery Windows virtual secretary.";

        if connectivity::check_online_status().await {
            // Route to Groq cloud LLM
            self.groq.query(prompt, system_prompt).await
        } else {
            // Offline: pick model size based on GPU context
            let model = if is_gpu_heavy_app {
                "phi4-mini" // or qwen2.5:3b
            } else {
                "llama3:8b"
            };
            self.ollama.query(prompt, model, system_prompt).await
        }
    }
}
