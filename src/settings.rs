use std::{fs, path::PathBuf};

use directories::ProjectDirs;
use serde::{Deserialize, Serialize};


#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Settings {
    pub latitude: String,
    pub longitude: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            latitude: Default::default(),
            longitude: Default::default(),
        }
    }
}

impl Settings {
    fn get_path(&self) -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("de", "osor", "TrayWeather") {
            let config_dir = proj_dirs.config_dir();
            return config_dir.join("settings.toml");
        } else {
            panic!("Failed to get settings directory.");
        }
    }

    pub fn exists(&self) -> bool {
        self.get_path().exists()
    }

    pub fn load(&mut self) {
        let settings_string = fs::read_to_string(self.get_path()).unwrap();
        let settings: Settings = toml::from_str(&settings_string).expect("Could not read settings file");
        *self = settings;
    }

    pub fn save(&self) {
        let path = self.get_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("Could not create settings directory.");
        }
        let settings_string = toml::to_string_pretty(&self).unwrap();
        fs::write(path, settings_string).unwrap();
    }
}