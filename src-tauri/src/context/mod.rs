pub struct ForegroundContext;

impl ForegroundContext {
    pub fn get_active_app_name() -> Result<String, String> {
        // TODO: Call Windows API (GetForegroundWindow -> GetWindowThreadProcessId -> QueryFullProcessImageName)
        // TODO: Return active executable name (e.g., "chrome.exe", "minecraft.exe")
        Ok("explorer.exe".to_string())
    }

    pub fn is_gpu_heavy(heavy_apps: &[String]) -> bool {
        if let Ok(app) = Self::get_active_app_name() {
            let app_lower = app.to_lowercase();
            heavy_apps.iter().any(|h| app_lower.contains(&h.to_lowercase()))
        } else {
            false
        }
    }
}
