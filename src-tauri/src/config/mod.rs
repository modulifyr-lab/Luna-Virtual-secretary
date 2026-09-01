use serde::{Deserialize, Serialize};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub groq_api_key: String,
    pub hotkey_binding: String,
    pub whisper_model_path: String,
    pub kokoro_model_path: String,
    pub ollama_base_url: String,
    pub heavy_gpu_apps: Vec<String>,
    pub db_path: String,
    pub fact_extraction_interval: usize,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            groq_api_key: String::new(),
            hotkey_binding: "Ctrl+Alt+Space".to_string(),
            whisper_model_path: "models/ggml-base.en.bin".to_string(),
            kokoro_model_path: "models/kokoro-v0_88.onnx".to_string(),
            ollama_base_url: "http://localhost:11434".to_string(),
            heavy_gpu_apps: vec!["minecraft.exe".to_string(), "cyberpunk2077.exe".to_string()],
            db_path: "luna_memory.db".to_string(),
            fact_extraction_interval: 10,
        }
    }
}

pub fn get_config_path() -> PathBuf {
    if let Some(mut path) = dirs::config_dir() {
        path.push("Luna");
        path.push("config.toml");
        path
    } else {
        PathBuf::from("config.toml")
    }
}

pub fn load_config() -> AppConfig {
    let path = get_config_path();
    load_config_from_path(&path)
}

pub fn load_config_from_path(path: &Path) -> AppConfig {
    let default_config = AppConfig::default();

    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(toml_str) = toml::to_string_pretty(&default_config) {
            if let Err(e) = fs::write(path, toml_str) {
                eprintln!("[Config] Failed to write default config to {:?}: {}", path, e);
            } else {
                println!("[Config] Created default config file at {:?}", path);
            }
        }
        return default_config;
    }

    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[Config] Failed to read config file at {:?}: {}, using defaults", path, e);
            return default_config;
        }
    };

    let parsed_val: Result<toml::Value, _> = toml::from_str(&contents);
    let table = match parsed_val {
        Ok(toml::Value::Table(t)) => t,
        _ => {
            eprintln!("[Config] Failed to parse TOML from {:?}, falling back to defaults", path);
            return default_config;
        }
    };

    AppConfig {
        groq_api_key: table
            .get("groq_api_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or(default_config.groq_api_key),
        hotkey_binding: table
            .get("hotkey_binding")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or(default_config.hotkey_binding),
        whisper_model_path: table
            .get("whisper_model_path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or(default_config.whisper_model_path),
        kokoro_model_path: table
            .get("kokoro_model_path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or(default_config.kokoro_model_path),
        ollama_base_url: table
            .get("ollama_base_url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or(default_config.ollama_base_url),
        heavy_gpu_apps: table
            .get("heavy_gpu_apps")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or(default_config.heavy_gpu_apps),
        db_path: table
            .get("db_path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or(default_config.db_path),
        fact_extraction_interval: table
            .get("fact_extraction_interval")
            .and_then(|v| v.as_integer())
            .and_then(|i| if i >= 0 { Some(i as usize) } else { None })
            .unwrap_or(default_config.fact_extraction_interval),
    }
}

pub fn parse_hotkey(binding: &str) -> Result<HotKey, String> {
    if let Ok(hk) = HotKey::from_str(binding) {
        return Ok(hk);
    }

    let parts: Vec<&str> = binding.split('+').map(|s| s.trim()).collect();
    let mut mods = Modifiers::empty();
    let mut code: Option<Code> = None;

    for part in parts {
        let lower = part.to_lowercase();
        match lower.as_str() {
            "ctrl" | "control" => mods |= Modifiers::CONTROL,
            "alt" => mods |= Modifiers::ALT,
            "shift" => mods |= Modifiers::SHIFT,
            "super" | "win" | "meta" | "cmd" => mods |= Modifiers::SUPER,
            "space" => code = Some(Code::Space),
            "tab" => code = Some(Code::Tab),
            "enter" | "return" => code = Some(Code::Enter),
            "backquote" | "`" => code = Some(Code::Backquote),
            s if s.starts_with('f') && s.len() <= 3 => {
                if let Ok(n) = s[1..].parse::<u8>() {
                    code = match n {
                        1 => Some(Code::F1),
                        2 => Some(Code::F2),
                        3 => Some(Code::F3),
                        4 => Some(Code::F4),
                        5 => Some(Code::F5),
                        6 => Some(Code::F6),
                        7 => Some(Code::F7),
                        8 => Some(Code::F8),
                        9 => Some(Code::F9),
                        10 => Some(Code::F10),
                        11 => Some(Code::F11),
                        12 => Some(Code::F12),
                        _ => None,
                    };
                }
            }
            s if s.len() == 1 => {
                let ch = s.chars().next().unwrap();
                if ch.is_ascii_alphabetic() {
                    let uppercase = ch.to_ascii_uppercase();
                    code = match uppercase {
                        'A' => Some(Code::KeyA),
                        'B' => Some(Code::KeyB),
                        'C' => Some(Code::KeyC),
                        'D' => Some(Code::KeyD),
                        'E' => Some(Code::KeyE),
                        'F' => Some(Code::KeyF),
                        'G' => Some(Code::KeyG),
                        'H' => Some(Code::KeyH),
                        'I' => Some(Code::KeyI),
                        'J' => Some(Code::KeyJ),
                        'K' => Some(Code::KeyK),
                        'L' => Some(Code::KeyL),
                        'M' => Some(Code::KeyM),
                        'N' => Some(Code::KeyN),
                        'O' => Some(Code::KeyO),
                        'P' => Some(Code::KeyP),
                        'Q' => Some(Code::KeyQ),
                        'R' => Some(Code::KeyR),
                        'S' => Some(Code::KeyS),
                        'T' => Some(Code::KeyT),
                        'U' => Some(Code::KeyU),
                        'V' => Some(Code::KeyV),
                        'W' => Some(Code::KeyW),
                        'X' => Some(Code::KeyX),
                        'Y' => Some(Code::KeyY),
                        'Z' => Some(Code::KeyZ),
                        _ => None,
                    };
                } else if ch.is_ascii_digit() {
                    code = match ch {
                        '0' => Some(Code::Digit0),
                        '1' => Some(Code::Digit1),
                        '2' => Some(Code::Digit2),
                        '3' => Some(Code::Digit3),
                        '4' => Some(Code::Digit4),
                        '5' => Some(Code::Digit5),
                        '6' => Some(Code::Digit6),
                        '7' => Some(Code::Digit7),
                        '8' => Some(Code::Digit8),
                        '9' => Some(Code::Digit9),
                        _ => None,
                    };
                }
            }
            _ => {}
        }
    }

    let code = code.ok_or_else(|| format!("Invalid key code in binding '{}'", binding))?;
    let mods_opt = if mods.is_empty() { None } else { Some(mods) };
    Ok(HotKey::new(mods_opt, code))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_parse_hotkey() {
        let hk = parse_hotkey("Ctrl+Alt+Space");
        assert!(hk.is_ok());
    }

    #[test]
    fn test_load_config_creates_default_file_when_missing() {
        let temp_dir = std::env::temp_dir().join(format!("luna_cfg_test_{}", std::process::id()));
        let cfg_path = temp_dir.join("subfolder").join("config.toml");

        if cfg_path.exists() {
            let _ = fs::remove_file(&cfg_path);
        }

        let loaded = load_config_from_path(&cfg_path);
        assert_eq!(loaded.hotkey_binding, "Ctrl+Alt+Space");
        assert!(cfg_path.exists());

        let contents = fs::read_to_string(&cfg_path).unwrap();
        assert!(contents.contains("hotkey_binding = \"Ctrl+Alt+Space\""));
        assert!(contents.contains("heavy_gpu_apps ="));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_config_parses_file_and_fallbacks() {
        let temp_dir = std::env::temp_dir().join(format!("luna_cfg_parse_test_{}", std::process::id()));
        let cfg_path = temp_dir.join("config.toml");
        let _ = fs::create_dir_all(&temp_dir);

        let custom_toml = r#"
groq_api_key = "gsk_test123"
hotkey_binding = "Ctrl+Shift+L"
heavy_gpu_apps = ["game.exe"]
"#;
        fs::write(&cfg_path, custom_toml).unwrap();

        let loaded = load_config_from_path(&cfg_path);
        assert_eq!(loaded.groq_api_key, "gsk_test123");
        assert_eq!(loaded.hotkey_binding, "Ctrl+Shift+L");
        assert_eq!(loaded.heavy_gpu_apps, vec!["game.exe".to_string()]);
        // Unspecified fields fallback to defaults
        assert_eq!(loaded.ollama_base_url, "http://localhost:11434");
        assert_eq!(loaded.db_path, "luna_memory.db");
        assert_eq!(loaded.fact_extraction_interval, 10);

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
