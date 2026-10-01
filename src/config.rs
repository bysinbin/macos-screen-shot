use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub area_hotkey_modifier: String,
    pub area_hotkey_key: String,

    pub fullscreen_hotkey_modifier: String,
    pub fullscreen_hotkey_key: String,

    pub pin_hotkey_modifier: String,
    pub pin_hotkey_key: String,

    pub auto_copy_to_clipboard: bool,
    pub auto_save_to_folder: bool,
    pub save_directory: String,
    pub image_format: String,
    pub show_magnifier: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let desktop = dirs::desktop_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .to_string_lossy()
            .to_string();

        Self {
            area_hotkey_modifier: "ALT".to_string(), // ⌥A
            area_hotkey_key: "KeyA".to_string(),

            fullscreen_hotkey_modifier: "ALT".to_string(), // ⌥S
            fullscreen_hotkey_key: "KeyS".to_string(),

            pin_hotkey_modifier: "ALT".to_string(), // ⌥P
            pin_hotkey_key: "KeyP".to_string(),

            auto_copy_to_clipboard: true,
            auto_save_to_folder: false,
            save_directory: desktop,
            image_format: "PNG".to_string(),
            show_magnifier: true,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("macos_screenshot");
        let _ = fs::create_dir_all(&config_dir);
        config_dir.join("config.json")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                return cfg;
            }
        }
        let default_cfg = Self::default();
        let _ = default_cfg.save();
        default_cfg
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("JSON serialize error: {e}"))?;
        fs::write(&path, json).map_err(|e| format!("Save config error: {e}"))?;
        Ok(())
    }

    pub fn parse_modifier(s: &str) -> Option<Modifiers> {
        match s {
            "ALT" => Some(Modifiers::ALT),
            "CMD_SHIFT" => Some(Modifiers::META | Modifiers::SHIFT),
            "CTRL_ALT" => Some(Modifiers::CONTROL | Modifiers::ALT),
            "CMD_ALT" => Some(Modifiers::META | Modifiers::ALT),
            _ => Some(Modifiers::ALT),
        }
    }

    pub fn parse_code(s: &str) -> Code {
        match s {
            "KeyA" => Code::KeyA,
            "KeyB" => Code::KeyB,
            "KeyC" => Code::KeyC,
            "KeyD" => Code::KeyD,
            "KeyP" => Code::KeyP,
            "KeyR" => Code::KeyR,
            "KeyS" => Code::KeyS,
            "KeyX" => Code::KeyX,
            "Digit1" => Code::Digit1,
            "Digit2" => Code::Digit2,
            "Digit3" => Code::Digit3,
            "Digit4" => Code::Digit4,
            _ => Code::KeyA,
        }
    }

    pub fn to_area_hotkey(&self) -> HotKey {
        HotKey::new(
            Self::parse_modifier(&self.area_hotkey_modifier),
            Self::parse_code(&self.area_hotkey_key),
        )
    }

    pub fn to_fullscreen_hotkey(&self) -> HotKey {
        HotKey::new(
            Self::parse_modifier(&self.fullscreen_hotkey_modifier),
            Self::parse_code(&self.fullscreen_hotkey_key),
        )
    }

    pub fn to_pin_hotkey(&self) -> HotKey {
        HotKey::new(
            Self::parse_modifier(&self.pin_hotkey_modifier),
            Self::parse_code(&self.pin_hotkey_key),
        )
    }

    pub fn modifier_display(m: &str) -> &'static str {
        match m {
            "ALT" => "⌥ (Option)",
            "CMD_SHIFT" => "⌘ + ⇧ (Cmd + Shift)",
            "CTRL_ALT" => "⌃ + ⌥ (Ctrl + Option)",
            "CMD_ALT" => "⌘ + ⌥ (Cmd + Option)",
            _ => "⌥ (Option)",
        }
    }

    pub fn key_display(k: &str) -> &'static str {
        match k {
            "KeyA" => "A",
            "KeyB" => "B",
            "KeyC" => "C",
            "KeyD" => "D",
            "KeyP" => "P",
            "KeyR" => "R",
            "KeyS" => "S",
            "KeyX" => "X",
            "Digit1" => "1",
            "Digit2" => "2",
            "Digit3" => "3",
            "Digit4" => "4",
            _ => "A",
        }
    }

    pub fn hotkey_full_label(modifier: &str, key: &str) -> String {
        let mod_sym = match modifier {
            "ALT" => "⌥",
            "CMD_SHIFT" => "⌘⇧",
            "CTRL_ALT" => "⌃⌥",
            "CMD_ALT" => "⌘⌥",
            _ => "⌥",
        };
        format!("{}{}", mod_sym, Self::key_display(key))
    }
}
