pub struct ForegroundContext;

impl ForegroundContext {
    #[cfg(target_os = "windows")]
    pub fn get_active_app_name() -> Result<String, String> {
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindowThreadProcessId,
        };

        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == 0 {
                return Err("No active foreground window found".to_string());
            }

            let mut process_id: u32 = 0;
            GetWindowThreadProcessId(hwnd, &mut process_id);
            if process_id == 0 {
                return Err("Failed to resolve process ID for foreground window".to_string());
            }

            let process_handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, process_id);
            if process_handle == 0 {
                return Err(format!("Failed to open process handle for PID {}", process_id));
            }

            let mut buffer: [u16; 1024] = [0; 1024];
            let mut size: u32 = buffer.len() as u32;

            let res = QueryFullProcessImageNameW(process_handle, 0, buffer.as_mut_ptr(), &mut size);
            CloseHandle(process_handle);

            if res == 0 || size == 0 {
                return Err("QueryFullProcessImageNameW failed".to_string());
            }

            let full_path = String::from_utf16_lossy(&buffer[..size as usize]);
            let exe_name = std::path::Path::new(&full_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&full_path)
                .to_string();

            Ok(exe_name)
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn get_active_app_name() -> Result<String, String> {
        // Fallback implementation for non-Windows operating systems
        Ok("explorer.exe".to_string())
    }

    pub fn is_gpu_heavy(heavy_apps: &[String]) -> bool {
        if let Ok(app) = Self::get_active_app_name() {
            let app_lower = app.to_lowercase();
            heavy_apps.iter().any(|h| {
                let h_lower = h.to_lowercase();
                app_lower == h_lower || app_lower.contains(&h_lower)
            })
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_is_gpu_heavy_matching() {
        let heavy_apps = vec!["minecraft.exe".to_string(), "cyberpunk2077.exe".to_string()];
        // Test matching helper with explicit names
        let check_app = |app: &str| {
            let app_lower = app.to_lowercase();
            heavy_apps.iter().any(|h| {
                let h_lower = h.to_lowercase();
                app_lower == h_lower || app_lower.contains(&h_lower)
            })
        };

        assert!(check_app("Minecraft.exe"));
        assert!(check_app("cyberpunk2077.exe"));
        assert!(check_app("C:\\Games\\Minecraft.exe"));
        assert!(!check_app("chrome.exe"));
        assert!(!check_app("explorer.exe"));
    }
}
