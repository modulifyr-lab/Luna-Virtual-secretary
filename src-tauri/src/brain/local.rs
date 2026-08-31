use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct OllamaClient {
    base_url: String,
    client: Client,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct OllamaChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaMessageResponse {
    content: String,
}

#[derive(Deserialize)]
struct OllamaChatResponse {
    message: OllamaMessageResponse,
}

impl OllamaClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_default();
        Self {
            base_url: base_url.into(),
            client,
        }
    }

    pub async fn query(&self, prompt: &str, model: &str, system_prompt: &str) -> Result<String, String> {
        let endpoint = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

        let request_payload = OllamaChatRequest {
            model: model.to_string(),
            messages: vec![
                OllamaMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                OllamaMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
            stream: false,
        };

        let response = self
            .client
            .post(&endpoint)
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| format!("Ollama HTTP request failed (url: {}): {}", endpoint, e))?;

        if !response.status().is_success() {
            let status = response.status();
            let err_body = response.text().await.unwrap_or_default();
            return Err(format!("Ollama API returned error status {}: {}", status, err_body));
        }

        let parsed: OllamaChatResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        Ok(parsed.message.content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ollama_client_invalid_url_fails() {
        let client = OllamaClient::new("http://127.0.0.1:59999"); // unreachable port
        let result = client.query("Hello", "llama3:8b", "System prompt").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Ollama HTTP request failed"));
    }
}
