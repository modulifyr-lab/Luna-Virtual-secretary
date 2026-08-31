use tauri::command;
use crate::brain;
use crate::config::AppConfig;

#[command]
pub async fn process_user_input(
    input: String,
    config: tauri::State<'_, AppConfig>,
) -> Result<String, String> {
    brain::get_response(&input, &config).await
}

#[command]
pub async fn get_status() -> Result<String, String> {
    if brain::connectivity::check_online_status().await {
        Ok("Online (Groq)".to_string())
    } else {
        Ok("Offline (Ollama)".to_string())
    }
}
