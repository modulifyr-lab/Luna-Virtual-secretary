use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct GroqClient {
    api_key: String,
    client: Client,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
}

impl GroqClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default();
        Self {
            api_key: api_key.into(),
            client,
        }
    }

    pub async fn query(&self, prompt: &str, system_prompt: &str) -> Result<String, String> {
        if self.api_key.trim().is_empty() {
            return Err("Groq API key is empty".to_string());
        }

        let request_payload = ChatCompletionRequest {
            model: "llama-3.3-70b-versatile".to_string(),
            messages: vec![
                ChatMessage {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                ChatMessage {
                    role: "user".to_string(),
                    content: prompt.to_string(),
                },
            ],
        };

        let response = self
            .client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key.trim()))
            .header("Content-Type", "application/json")
            .json(&request_payload)
            .send()
            .await
            .map_err(|e| format!("Groq HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let err_body = response.text().await.unwrap_or_default();
            return Err(format!("Groq API returned error status {}: {}", status, err_body));
        }

        let parsed: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Groq response: {}", e))?;

        parsed
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| "Groq response contained no choices".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_groq_client_empty_key_error() {
        let client = GroqClient::new("");
        let result = client.query("Hello", "System prompt").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("empty"));
    }
}
