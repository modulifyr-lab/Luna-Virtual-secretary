use serde::{Deserialize, Serialize};
use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub groq_api_key: String,
    pub hotkey_binding: String,
    pub whisper_model_path: String,
    pub kokoro_model_path: String,
    pub ollama_base_url: String,
    pub heavy_gpu_apps: Vec<String>,
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
        }
    }
}

pub fn load_config() -> AppConfig {
    // TODO: Load configuration from file or environment variables
    AppConfig::default()
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

    #[test]
    fn test_parse_hotkey() {
        let hk = parse_hotkey("Ctrl+Alt+Space");
        assert!(hk.is_ok());
    }
}
