use tauri::command;

#[command]
pub async fn process_user_input(input: String) -> Result<String, String> {
    // TODO: Dispatch user text/speech input to brain module
    // TODO: Route through skills or LLM and return assistant response
    Ok(format!("Luna received: {}", input))
}

#[command]
pub async fn get_status() -> Result<String, String> {
    // TODO: Query online/offline connectivity and foreground app status
    Ok("Online (Groq)".to_string())
}
