pub mod cloud;
pub mod connectivity;
pub mod local;

use std::sync::Arc;
use serde::{Deserialize, Serialize};
use cloud::GroqClient;
use local::OllamaClient;
use crate::config::AppConfig;
use crate::context::ForegroundContext;
use crate::memory::MemoryStore;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BrainResponse {
    pub reasoning: String,
    pub response: String,
}

/// Personality system prompt for Luna, defined as a constant in one place for easy tuning.
pub const LUNA_SYSTEM_PROMPT: &str = "\
You are Luna, a fiery, highly efficient, witty, and sharp-tongued virtual secretary on Windows. \
You get things done with supreme confidence, directness, and a touch of fiery attitude. \
Keep your answers brief, snappy, helpful, and full of personality.\n\n\
You MUST ALWAYS respond in the following strict format:\n\
REASONING: <one short sentence on why/how you're answering>\n\
RESPONSE: <the actual answer to speak/display>\
";

/// Parses the raw LLM response into reasoning and response components.
/// If parsing fails, falls back gracefully treating the whole output as response.
pub fn parse_llm_response(raw_output: &str) -> BrainResponse {
    let raw_trimmed = raw_output.trim();
    let uppercase = raw_trimmed.to_uppercase();

    if let (Some(reasoning_pos), Some(response_pos)) = (uppercase.find("REASONING:"), uppercase.find("RESPONSE:")) {
        if reasoning_pos < response_pos {
            let reasoning_start = reasoning_pos + "REASONING:".len();
            let reasoning_raw = &raw_trimmed[reasoning_start..response_pos];
            let reasoning_part = reasoning_raw.trim().trim_matches(|c: char| c == '*' || c == '_' || c == '`');

            let response_start = response_pos + "RESPONSE:".len();
            let response_raw = &raw_trimmed[response_start..];
            let response_part = response_raw.trim().trim_matches(|c: char| c == '*' || c == '_' || c == '`');

            let reasoning_text = if reasoning_part.trim().is_empty() {
                "(no structured reasoning returned)".to_string()
            } else {
                reasoning_part.trim().to_string()
            };

            let response_text = response_part.trim().to_string();

            if !response_text.is_empty() {
                return BrainResponse {
                    reasoning: reasoning_text,
                    response: response_text,
                };
            }
        }
    }

    BrainResponse {
        reasoning: "(no structured reasoning returned)".to_string(),
        response: raw_trimmed.to_string(),
    }
}

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

    pub async fn process_prompt(&self, prompt: &str, is_gpu_heavy_app: bool) -> Result<BrainResponse, String> {
        self.process_prompt_with_system(prompt, is_gpu_heavy_app, LUNA_SYSTEM_PROMPT).await
    }

    pub async fn process_prompt_with_system(&self, prompt: &str, is_gpu_heavy_app: bool, system_prompt: &str) -> Result<BrainResponse, String> {
        let raw_res = if connectivity::check_online_status().await {
            match self.groq.query(prompt, system_prompt).await {
                Ok(response) => Ok(response),
                Err(err) => {
                    eprintln!("[BrainRouter] Groq Cloud failed ({}), falling back to local Ollama.", err);
                    let model = if is_gpu_heavy_app {
                        "phi4-mini"
                    } else {
                        "llama3:8b"
                    };
                    self.ollama.query(prompt, model, system_prompt).await
                }
            }
        } else {
            // Offline or Groq fallback: select model based on GPU context
            let model = if is_gpu_heavy_app {
                "phi4-mini"
            } else {
                "llama3:8b"
            };

            self.ollama.query(prompt, model, system_prompt).await
        };

        raw_res.map(|raw| parse_llm_response(&raw))
    }
}

/// Dynamically builds the system prompt incorporating up to 20 recent facts.
pub fn build_system_prompt(memory_store: &MemoryStore) -> String {
    let mut prompt = LUNA_SYSTEM_PROMPT.to_string();

    if let Ok(facts) = memory_store.get_recent_facts(20) {
        if !facts.is_empty() {
            prompt.push_str("\n\nKnown context about the user:");
            for fact in facts {
                prompt.push_str(&format!("\n- {}", fact.fact_text));
            }
        }
    }

    prompt
}

/// Routes user input to either Groq (if online) or Ollama (if offline or if Groq fails).
/// Logs conversations to SQLite memory and triggers background fact extraction.
pub async fn get_response(user_text: &str, config: &AppConfig) -> Result<BrainResponse, String> {
    get_response_with_source(user_text, config, "typed").await
}

pub async fn get_response_with_source(user_text: &str, config: &AppConfig, source: &str) -> Result<BrainResponse, String> {
    let memory_store = Arc::new(MemoryStore::new(&config.db_path));
    if let Err(e) = memory_store.init() {
        eprintln!("[Memory] DB init warning in get_response: {}", e);
    }

    // Step 2 — Log user input
    let user_conv_id = memory_store
        .log_conversation("user", user_text, source)
        .unwrap_or_else(|e| {
            eprintln!("[Memory] Failed to log user input: {}", e);
            -1
        });

    let system_prompt = build_system_prompt(&memory_store);

    // 1. Try matching built-in skills first (weather, dictionary, news, file_search, office, web_search)
    let response_res = if let Some(skill_res) = crate::skills::SkillDispatcher::try_dispatch(user_text).await {
        skill_res.map(|text| BrainResponse {
            reasoning: "(skill response)".to_string(),
            response: text,
        })
    } else {
        // 2. Fall back to BrainRouter (Groq online / Ollama offline)
        let router = BrainRouter::new(config.groq_api_key.clone(), config.ollama_base_url.clone());
        let is_gpu_heavy = ForegroundContext::is_gpu_heavy(&config.heavy_gpu_apps);
        router.process_prompt_with_system(user_text, is_gpu_heavy, &system_prompt).await
    };

    if let Ok(ref brain_resp) = response_res {
        // Step 2 — Log assistant response
        if let Err(e) = memory_store.log_conversation("assistant", &brain_resp.response, "system") {
            eprintln!("[Memory] Failed to log assistant response: {}", e);
        }

        // Step 3 — Background fact extraction check
        let memory_store_bg = Arc::clone(&memory_store);
        let config_bg = config.clone();
        tokio::spawn(async move {
            maybe_extract_facts(memory_store_bg, config_bg, user_conv_id).await;
        });
    }

    response_res
}

/// Runs periodic background fact extraction if conversation count crosses threshold
async fn maybe_extract_facts(memory_store: Arc<MemoryStore>, config: AppConfig, source_conv_id: i64) {
    let count = match memory_store.get_conversation_count() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[Memory] Failed to get conversation count for fact extraction check: {}", e);
            return;
        }
    };

    if config.fact_extraction_interval > 0 && count % config.fact_extraction_interval == 0 {
        let recent_convs = match memory_store.get_recent_conversations(config.fact_extraction_interval) {
            Ok(convs) => convs,
            Err(e) => {
                eprintln!("[Memory] Failed to fetch recent conversations: {}", e);
                return;
            }
        };

        if recent_convs.is_empty() {
            return;
        }

        let mut history_str = String::new();
        for conv in recent_convs {
            history_str.push_str(&format!("{}: {}\n", conv.role, conv.text));
        }

        let extraction_prompt = format!(
            "Analyze the following recent conversation history and extract durable, long-term facts about the user or user's preferences/recurring context.\n\
            Do NOT summarize the whole conversation. Just return new durable facts, one per line starting with a dash ('-'). If no new durable facts are found, respond with 'NONE'.\n\n\
            Conversation History:\n{}",
            history_str
        );

        let router = BrainRouter::new(config.groq_api_key.clone(), config.ollama_base_url.clone());
        let is_gpu_heavy = ForegroundContext::is_gpu_heavy(&config.heavy_gpu_apps);
        let sys_prompt = "You are a helpful memory extraction assistant. Extract durable facts accurately and concisely.";

        match router.process_prompt_with_system(&extraction_prompt, is_gpu_heavy, sys_prompt).await {
            Ok(brain_res) => {
                let extracted_text = brain_res.response;
                if extracted_text.trim().to_uppercase() != "NONE" {
                    for line in extracted_text.lines() {
                        let trimmed = line.trim();
                        let fact_text = if trimmed.starts_with('-') || trimmed.starts_with('*') {
                            trimmed[1..].trim()
                        } else {
                            trimmed
                        };

                        if !fact_text.is_empty() && fact_text.to_uppercase() != "NONE" {
                            let src_id = if source_conv_id > 0 { Some(source_conv_id) } else { None };
                            if let Err(e) = memory_store.store_fact(fact_text, src_id) {
                                eprintln!("[Memory] Failed to store extracted fact: {}", e);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[Memory] Fact extraction LLM call failed: {}", e);
            }
        }
    }
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

    #[test]
    fn test_build_system_prompt_with_facts() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("luna_sysprompt_test_{}.db", std::process::id()));
        let db_path_str = db_path.to_str().unwrap();

        let store = MemoryStore::new(db_path_str);
        store.init().unwrap();

        let initial_prompt = build_system_prompt(&store);
        assert_eq!(initial_prompt, LUNA_SYSTEM_PROMPT);

        store.store_fact("User prefers dark mode", None).unwrap();
        store.store_fact("User lives in Tokyo", None).unwrap();

        let prompt_with_facts = build_system_prompt(&store);
        assert!(prompt_with_facts.contains("Known context about the user:"));
        assert!(prompt_with_facts.contains("- User prefers dark mode"));
        assert!(prompt_with_facts.contains("- User lives in Tokyo"));

        let _ = std::fs::remove_file(db_path);
    }

    #[tokio::test]
    async fn test_get_response_logs_conversations_and_skills() {
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join(format!("luna_getresp_test_{}.db", std::process::id()));
        let db_path_str = db_path.to_str().unwrap();

        let mut config = AppConfig::default();
        config.db_path = db_path_str.to_string();

        let res = get_response_with_source("weather in London", &config, "voice").await;
        assert!(res.is_ok());
        let brain_resp = res.unwrap();
        assert_eq!(brain_resp.reasoning, "(skill response)");

        let store = MemoryStore::new(db_path_str);
        let convs = store.get_recent_conversations(10).unwrap();
        assert_eq!(convs.len(), 2);
        assert_eq!(convs[0].role, "user");
        assert_eq!(convs[0].source, "voice");
        assert_eq!(convs[0].text, "weather in London");
        assert_eq!(convs[1].role, "assistant");
        assert_eq!(convs[1].source, "system");

        let _ = std::fs::remove_file(db_path);
    }

    #[test]
    fn test_parse_llm_response_valid_format() {
        let input = "REASONING: The user asked for a quick greeting.\nRESPONSE: Hello there! How can I assist you today?";
        let parsed = parse_llm_response(input);
        assert_eq!(parsed.reasoning, "The user asked for a quick greeting.");
        assert_eq!(parsed.response, "Hello there! How can I assist you today?");
    }

    #[test]
    fn test_parse_llm_response_case_insensitive_headers() {
        let input = "reasoning: Checking status of weather API.\nresponse: It's sunny outside.";
        let parsed = parse_llm_response(input);
        assert_eq!(parsed.reasoning, "Checking status of weather API.");
        assert_eq!(parsed.response, "It's sunny outside.");
    }

    #[test]
    fn test_parse_llm_response_malformed_fallback() {
        let input = "Just a raw answer without structured format.";
        let parsed = parse_llm_response(input);
        assert_eq!(parsed.reasoning, "(no structured reasoning returned)");
        assert_eq!(parsed.response, "Just a raw answer without structured format.");
    }

    #[test]
    fn test_parse_llm_response_empty_reasoning_fallback() {
        let input = "REASONING:   \nRESPONSE: Direct answer.";
        let parsed = parse_llm_response(input);
        assert_eq!(parsed.reasoning, "(no structured reasoning returned)");
        assert_eq!(parsed.response, "Direct answer.");
    }
}
