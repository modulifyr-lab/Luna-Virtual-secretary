use std::process::Command;

pub struct FileSearch;

impl FileSearch {
    pub fn search(query: &str) -> Result<Vec<String>, String> {
        // TODO: Shell out to `es.exe` (Everything CLI) via std::process::Command
        // TODO: Parse output lines as file paths
        let _cmd = Command::new("es.exe");
        Ok(vec![format!("Stub search result for {}", query)])
    }
}
