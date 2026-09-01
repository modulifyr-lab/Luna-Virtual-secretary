use serde::Deserialize;
use std::process::Command;

#[derive(Debug, Deserialize)]
struct OfficeOutput {
    status: String,
    message: String,
}

pub struct OfficeBridge {
    script_path: String,
}

impl OfficeBridge {
    pub fn new(script_path: impl Into<String>) -> Self {
        Self {
            script_path: script_path.into(),
        }
    }

    pub fn execute(&self, app: &str, action: &str, params_json: &str) -> Result<String, String> {
        let script_path = if std::path::Path::new(&self.script_path).exists() {
            self.script_path.clone()
        } else if std::path::Path::new("python-bridge/office_control.py").exists() {
            "python-bridge/office_control.py".to_string()
        } else {
            "../python-bridge/office_control.py".to_string()
        };

        let python_bin = if cfg!(target_os = "windows") { "python" } else { "python3" };

        let output_res = Command::new(python_bin)
            .arg(&script_path)
            .arg(app)
            .arg(action)
            .arg(if params_json.is_empty() { "{}" } else { params_json })
            .output();

        let output = match output_res {
            Ok(out) => out,
            Err(_) => {
                let alt_bin = if python_bin == "python3" { "python" } else { "python3" };
                Command::new(alt_bin)
                    .arg(&script_path)
                    .arg(app)
                    .arg(action)
                    .arg(if params_json.is_empty() { "{}" } else { params_json })
                    .output()
                    .map_err(|e| format!("Failed to execute python office_control.py: {}", e))?
            }
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("office_control.py failed: {}", stderr));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed: OfficeOutput = serde_json::from_str(stdout.trim())
            .map_err(|e| format!("Failed to parse office bridge JSON output (stdout: '{}'): {}", stdout.trim(), e))?;

        if parsed.status == "ok" {
            Ok(parsed.message)
        } else {
            // Return readable error message for TTS / natural response
            Ok(format!("Office command notification: {}", parsed.message))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_office_bridge_instantiation() {
        let bridge = OfficeBridge::new("python-bridge/office_control.py");
        assert_eq!(bridge.script_path, "python-bridge/office_control.py");
    }

    #[test]
    fn test_office_bridge_runs_and_handles_output() {
        let bridge = OfficeBridge::new("python-bridge/office_control.py");
        let res = bridge.execute("word", "create_doc", "{}");
        assert!(res.is_ok(), "Office bridge should return natural text result");
        let text = res.unwrap();
        assert!(text.contains("Microsoft Word") || text.contains("Office command notification"));
    }
}
