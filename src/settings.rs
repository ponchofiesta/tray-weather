use std::{fs, path::PathBuf};

use crate::{gui::IconTheme, weather::Location, Result};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub location: Location,
    pub icon_theme: IconTheme,
    #[serde(default)]
    pub autorun_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            location: Default::default(),
            icon_theme: IconTheme::Metno,
            autorun_enabled: false,
        }
    }
}

impl Settings {
    pub fn update(&mut self, new_settings: &Settings) {
        *self = new_settings.clone();
    }

    fn get_path(&self) -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("de", "osor", "TrayWeather") {
            let config_dir = proj_dirs.config_dir();
            return config_dir.join("settings.toml");
        } else {
            panic!("Failed to get settings directory.");
        }
    }

    pub fn load(&mut self) -> Result<()> {
        let settings_string = fs::read_to_string(self.get_path())?;
        let settings: Settings = toml::from_str(&settings_string)?;
        *self = settings;
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let path = self.get_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let settings_string = toml::to_string_pretty(&self)?;
        fs::write(path, settings_string)?;
        Ok(())
    }
}
