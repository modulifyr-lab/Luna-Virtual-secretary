pub struct WebSearchSkill;

impl WebSearchSkill {
    pub async fn search(query: &str) -> Result<String, String> {
        // TODO: Execute python-bridge/web_search.py via Command
        Ok(format!("Web search stub for query: {}", query))
    }
}
