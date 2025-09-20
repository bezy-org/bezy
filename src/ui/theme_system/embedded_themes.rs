use std::collections::HashMap;
use std::path::PathBuf;

pub const DARK_JSON: &str = include_str!("../themes/dark.json");
pub const LIGHT_JSON: &str = include_str!("../themes/light.json");
pub const STRAWBERRY_JSON: &str = include_str!("../themes/strawberry.json");
pub const CAMPFIRE_JSON: &str = include_str!("../themes/campfire.json");

pub fn get_embedded_themes() -> HashMap<String, &'static str> {
    let mut themes = HashMap::new();
    themes.insert("dark".to_string(), DARK_JSON);
    themes.insert("light".to_string(), LIGHT_JSON);
    themes.insert("strawberry".to_string(), STRAWBERRY_JSON);
    themes.insert("campfire".to_string(), CAMPFIRE_JSON);
    themes
}

pub fn get_user_themes_dir() -> PathBuf {
    let config_dir = dirs::config_dir().unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".config")
    });
    config_dir.join("bezy").join("themes")
}

pub fn user_themes_dir_exists() -> bool {
    get_user_themes_dir().exists()
}

pub fn load_theme_from_string(
    content: &str,
) -> Result<super::json_theme::JsonTheme, Box<dyn std::error::Error>> {
    let theme = serde_json::from_str(content)?;
    Ok(theme)
}
