pub struct GroqClient {
    api_key: String,
}

impl GroqClient {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }

    pub async fn query(&self, prompt: &str, _system_prompt: &str) -> Result<String, String> {
        // TODO: Call Groq API endpoint (OpenAI-compatible) via reqwest
        // TODO: Pass system prompt ("fiery secretary personality") + user prompt
        Ok(format!("Groq Cloud response stub for: {}", prompt))
    }
}
