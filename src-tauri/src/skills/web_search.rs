use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct SearchResultItem {
    title: String,
    href: String,
    body: String,
}

#[derive(Debug, Deserialize)]
struct SearchOutput {
    query: String,
    results: Vec<SearchResultItem>,
    status: String,
}

pub struct WebSearchSkill;

impl WebSearchSkill {
    pub async fn search(query: &str) -> Result<String, String> {
        let clean_query = query.trim();
        if clean_query.is_empty() {
            return Err("No search query provided.".to_string());
        }

        let script_path = "python-bridge/web_search.py";
        let python_bin = if cfg!(target_os = "windows") { "python" } else { "python3" };

        let output = Command::new(python_bin)
            .arg(script_path)
            .arg(clean_query)
            .output()
            .map_err(|e| format!("Failed to execute python web_search.py: {}", e))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("web_search.py failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed: SearchOutput = serde_json::from_str(stdout.trim())
            .map_err(|e| format!("Failed to parse web search JSON output: {}", e))?;

        if parsed.results.is_empty() {
            Ok(format!("No search results found for '{}'.", clean_query))
        } else {
            let top_items: Vec<String> = parsed
                .results
                .iter()
                .take(3)
                .map(|r| format!("'{}': {}", r.title, r.body))
                .collect();

            Ok(format!(
                "Here are the top search results for '{}': {}",
                parsed.query,
                top_items.join(" | ")
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_web_search_empty_query() {
        let res = WebSearchSkill::search("").await;
        assert!(res.is_err());
    }
}
