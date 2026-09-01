use std::process::Command;

pub struct FileSearch;

impl FileSearch {
    pub fn search(query: &str) -> Result<String, String> {
        let clean_query = query.trim();
        if clean_query.is_empty() {
            return Err("No search term provided for file search.".to_string());
        }

        let output_res = Command::new("es.exe")
            .arg("-n")
            .arg("5")
            .arg(clean_query)
            .output();

        match output_res {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let paths: Vec<String> = stdout
                        .lines()
                        .map(|line| line.trim().to_string())
                        .filter(|line| !line.is_empty())
                        .collect();

                    if paths.is_empty() {
                        Ok(format!("No files found matching '{}'.", clean_query))
                    } else {
                        Ok(format!(
                            "Found {} file(s) matching '{}': {}",
                            paths.len(),
                            clean_query,
                            paths.join(", ")
                        ))
                    }
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    Err(format!("es.exe search failed: {}", stderr))
                }
            }
            Err(_e) => {
                // Return spoken error message explaining es.exe / Voidtools Everything availability
                Ok(format!("Could not execute es.exe (Voidtools Everything search CLI). Please ensure es.exe is installed and in PATH. Search query was '{}'.", clean_query))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_search_empty_query() {
        let res = FileSearch::search("");
        assert!(res.is_err());
    }

    #[test]
    fn test_file_search_handles_missing_exe_gracefully() {
        let res = FileSearch::search("testfile");
        assert!(res.is_ok()); // Should return natural explanatory message rather than hard crash
    }
}
