use std::process::Command;

pub struct AppLaunch;

impl AppLaunch {
    pub fn launch_app(app_name: &str) -> Result<String, String> {
        let clean_name = app_name.trim();
        if clean_name.is_empty() {
            return Err("No application name provided to launch.".to_string());
        }

        // Attempt 1: Try Raycast CLI binary if available in PATH
        let cli_res = Command::new("raycast.exe")
            .arg(clean_name)
            .output();

        if let Ok(output) = cli_res {
            if output.status.success() {
                return Ok(format!("Opening {} for you.", clean_name));
            }
        }

        // Attempt 2: Try raycast:// protocol URI launch via system launcher
        #[cfg(target_os = "windows")]
        let protocol_res = Command::new("cmd")
            .args(["/C", "start", &format!("raycast://search?q={}", clean_name)])
            .output();

        #[cfg(not(target_os = "windows"))]
        let protocol_res = Command::new("xdg-open")
            .arg(&format!("raycast://search?q={}", clean_name))
            .output();

        match protocol_res {
            Ok(output) if output.status.success() => {
                Ok(format!("Opening {} for you.", clean_name))
            }
            _ => Err(format!(
                "Could not launch Raycast to open '{}'. Please ensure Raycast is installed and available via CLI or raycast:// protocol URIs.",
                clean_name
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_launch_empty_name() {
        let res = AppLaunch::launch_app("");
        assert!(res.is_err());
    }

    #[test]
    fn test_app_launch_returns_result_type() {
        let res = AppLaunch::launch_app("Chrome");
        // Result is Ok if command runner / launcher succeeds or Err if binary/URI runner fails
        match res {
            Ok(msg) => assert!(msg.contains("Opening Chrome")),
            Err(err) => assert!(err.contains("Could not launch Raycast")),
        }
    }
}
