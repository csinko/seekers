use crate::constants;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

fn get_settings_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let config_dir = PathBuf::from(home)
        .join(".config")
        .join(constants::CONFIG_DIR_NAME);
    fs::create_dir_all(&config_dir).ok();
    config_dir.join(constants::SETTINGS_FILE)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub menu_bar_display: String,
    pub show_percent_symbol: bool,
    pub progress_style: String,
    pub progress_length: u8,
    pub refresh_interval: u32,
    pub notify_session: u32,
    pub notify_weekly: u32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            menu_bar_display: "session".to_string(),
            show_percent_symbol: true,
            progress_style: "circles".to_string(),
            progress_length: 10,
            refresh_interval: 15,
            notify_session: 80,
            notify_weekly: 80,
        }
    }
}

pub struct SettingsManager;

impl SettingsManager {
    pub fn new() -> Self {
        Self
    }

    pub fn save(&self, settings: &AppSettings) -> Result<(), Box<dyn std::error::Error>> {
        let path = get_settings_path();
        let json = serde_json::to_string_pretty(settings)?;
        fs::write(&path, &json)?;
        Ok(())
    }

    pub fn load(&self) -> Result<AppSettings, Box<dyn std::error::Error>> {
        let path = get_settings_path();
        if !path.exists() {
            return Ok(AppSettings::default());
        }

        let json = fs::read_to_string(&path)?;
        let settings: AppSettings = serde_json::from_str(&json)?;
        Ok(settings)
    }
}
