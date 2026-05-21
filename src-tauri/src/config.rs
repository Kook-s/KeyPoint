use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub mouse_mode_enabled: bool,
    pub move_speed: i32,
    pub scroll_speed: i32,
    pub launch_at_startup: bool,
    #[serde(default)]
    pub keymap: KeyMap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMap {
    #[serde(default = "default_toggle_key")]
    pub toggle_key: String,
    #[serde(default = "default_toggle_modifiers")]
    pub toggle_modifiers: Vec<String>,
    #[serde(default = "default_move_up")]
    pub move_up: String,
    #[serde(default = "default_move_down")]
    pub move_down: String,
    #[serde(default = "default_move_left")]
    pub move_left: String,
    #[serde(default = "default_move_right")]
    pub move_right: String,
    #[serde(default = "default_left_click")]
    pub left_click: String,
    #[serde(default = "default_right_click")]
    pub right_click: String,
    #[serde(default = "default_scroll_modifier")]
    pub scroll_modifier: String,
}

impl Default for KeyMap {
    fn default() -> Self {
        Self {
            toggle_key: default_toggle_key(),
            toggle_modifiers: default_toggle_modifiers(),
            move_up: default_move_up(),
            move_down: default_move_down(),
            move_left: default_move_left(),
            move_right: default_move_right(),
            left_click: default_left_click(),
            right_click: default_right_click(),
            scroll_modifier: default_scroll_modifier(),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mouse_mode_enabled: false,
            move_speed: 16,
            scroll_speed: 4,
            launch_at_startup: false,
            keymap: KeyMap::default(),
        }
    }
}

pub fn read_config() -> io::Result<AppConfig> {
    let path = config_path()?;
    if !path.exists() {
        let config = AppConfig::default();
        write_config_to(&path, &config)?;
        return Ok(config);
    }

    let content = fs::read_to_string(path)?;
    let config: AppConfig = serde_json::from_str(&content).unwrap_or_default();
    Ok(sanitize_config(config))
}

pub fn sanitize_config(mut config: AppConfig) -> AppConfig {
    config.move_speed = config.move_speed.clamp(2, 64);
    config.scroll_speed = config.scroll_speed.clamp(1, 16);
    config.keymap = sanitize_keymap(config.keymap);
    config
}

pub fn write_config(config: &AppConfig) -> io::Result<()> {
    let path = config_path()?;
    let mut persisted = config.clone();
    persisted.mouse_mode_enabled = false;
    persisted = sanitize_config(persisted);
    write_config_to(&path, &persisted)
}

fn sanitize_keymap(keymap: KeyMap) -> KeyMap {
    let defaults = KeyMap::default();
    KeyMap {
        toggle_key: sanitize_key(&keymap.toggle_key).unwrap_or(defaults.toggle_key),
        toggle_modifiers: sanitize_modifiers(&keymap.toggle_modifiers),
        move_up: sanitize_key(&keymap.move_up).unwrap_or(defaults.move_up),
        move_down: sanitize_key(&keymap.move_down).unwrap_or(defaults.move_down),
        move_left: sanitize_key(&keymap.move_left).unwrap_or(defaults.move_left),
        move_right: sanitize_key(&keymap.move_right).unwrap_or(defaults.move_right),
        left_click: sanitize_key(&keymap.left_click).unwrap_or(defaults.left_click),
        right_click: sanitize_key(&keymap.right_click).unwrap_or(defaults.right_click),
        scroll_modifier: sanitize_key(&keymap.scroll_modifier).unwrap_or(defaults.scroll_modifier),
    }
}

fn sanitize_modifiers(modifiers: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for modifier in modifiers {
        let normalized = match modifier.trim().to_ascii_uppercase().as_str() {
            "COMMAND" | "META" | "CMD" => "Command",
            "CONTROL" | "CTRL" => "Control",
            "SHIFT" => "Shift",
            "ALT" | "OPTION" => "Alt",
            _ => continue,
        };
        if !result.iter().any(|value| value == normalized) {
            result.push(normalized.to_string());
        }
    }

    if result.is_empty() {
        default_toggle_modifiers()
    } else {
        result
    }
}

fn sanitize_key(value: &str) -> Option<String> {
    let key = normalize_key_code(value)?;
    if is_supported_key_code(&key) {
        Some(key)
    } else {
        None
    }
}

fn normalize_key_code(value: &str) -> Option<String> {
    let key = value.trim();
    if key.is_empty() {
        return None;
    }

    if key.len() == 1 {
        let mut chars = key.chars();
        let char = chars.next()?.to_ascii_uppercase();
        if char.is_ascii_alphabetic() {
            return Some(format!("Key{char}"));
        }
        if char.is_ascii_digit() {
            return Some(format!("Digit{char}"));
        }
    }

    Some(key.to_string())
}

fn is_supported_key_code(key: &str) -> bool {
    matches!(
        key,
        "Space"
            | "Enter"
            | "Tab"
            | "Escape"
            | "Backspace"
            | "Delete"
            | "ArrowUp"
            | "ArrowDown"
            | "ArrowLeft"
            | "ArrowRight"
            | "Home"
            | "End"
            | "PageUp"
            | "PageDown"
            | "Minus"
            | "Equal"
            | "BracketLeft"
            | "BracketRight"
            | "Backslash"
            | "Semicolon"
            | "Quote"
            | "Comma"
            | "Period"
            | "Slash"
            | "Backquote"
            | "NumpadDecimal"
            | "NumpadAdd"
            | "NumpadSubtract"
            | "NumpadMultiply"
            | "NumpadDivide"
            | "NumpadEnter"
    ) || key.strip_prefix("Key").is_some_and(|suffix| {
        suffix.len() == 1 && suffix.chars().all(|char| char.is_ascii_uppercase())
    }) || key
        .strip_prefix("Digit")
        .is_some_and(|suffix| suffix.len() == 1 && suffix.chars().all(|char| char.is_ascii_digit()))
        || key.strip_prefix("Numpad").is_some_and(|suffix| {
            suffix.len() == 1 && suffix.chars().all(|char| char.is_ascii_digit())
        })
        || key.strip_prefix('F').is_some_and(|suffix| {
            suffix
                .parse::<u8>()
                .is_ok_and(|number| (1..=12).contains(&number))
        })
}

fn default_toggle_key() -> String {
    "KeyZ".into()
}

fn default_toggle_modifiers() -> Vec<String> {
    #[cfg(target_os = "macos")]
    {
        vec!["Command".into(), "Shift".into()]
    }
    #[cfg(not(target_os = "macos"))]
    {
        vec!["Control".into(), "Shift".into()]
    }
}

fn default_move_up() -> String {
    "KeyW".into()
}

fn default_move_down() -> String {
    "KeyS".into()
}

fn default_move_left() -> String {
    "KeyA".into()
}

fn default_move_right() -> String {
    "KeyD".into()
}

fn default_left_click() -> String {
    "KeyJ".into()
}

fn default_right_click() -> String {
    "KeyL".into()
}

fn default_scroll_modifier() -> String {
    "KeyK".into()
}

fn write_config_to(path: &Path, config: &AppConfig) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)
}

fn config_path() -> io::Result<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "config directory not found"))?;
    Ok(base.join("KeyPoint").join("config.json"))
}
