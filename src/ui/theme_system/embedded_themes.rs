use std::collections::HashMap;
use std::path::PathBuf;

pub const DARKMODE_JSON: &str = include_str!("../themes/darkmode.json");
pub const LIGHTMODE_JSON: &str = include_str!("../themes/lightmode.json");
pub const STRAWBERRY_JSON: &str = include_str!("../themes/strawberry.json");
pub const CAMPFIRE_JSON: &str = include_str!("../themes/campfire.json");

pub fn get_embedded_themes() -> HashMap<String, &'static str> {
    let mut themes = HashMap::new();
    themes.insert("darkmode".to_string(), DARKMODE_JSON);
    themes.insert("lightmode".to_string(), LIGHTMODE_JSON);
    themes.insert("strawberry".to_string(), STRAWBERRY_JSON);
    themes.insert("campfire".to_string(), CAMPFIRE_JSON);
    themes
}

pub fn get_user_themes_dir() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".bezy").join("themes")
}

pub fn user_themes_dir_exists() -> bool {
    get_user_themes_dir().exists()
}

pub fn load_theme_from_string(content: &str) -> Result<super::json_theme::JsonTheme, Box<dyn std::error::Error>> {
    let theme = serde_json::from_str(content)?;
    Ok(theme)
}