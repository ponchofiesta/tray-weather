use auto_launch::AutoLaunch;
use log::debug;
use tray_icon::menu::Menu;

use crate::{error::Result, gui::WeatherTrayIcon, settings::Settings, weather::get_weather};

pub(crate) struct WeatherApp {
    pub settings: Settings,
    pub tray_icon: WeatherTrayIcon,
}

impl WeatherApp {
    pub fn new(settings: Settings, menu: Menu) -> Result<Self> {
        let tray_icon = WeatherTrayIcon::new(menu)?;
        Ok(WeatherApp {
            settings,
            tray_icon,
        })
    }

    pub async fn update_weather(&self) -> Result<()> {
        debug!("update_weather()");
        let weather = get_weather(&self.settings.location).await;
        match weather {
            Ok(weather) => self
                .tray_icon
                .set_weather(&self.settings.location, &weather)?,
            Err(err) => self.tray_icon.set_error(&format!("Fehler: {}", err))?,
        };
        Ok(())
    }

    pub async fn update_settings(&mut self, settings: Settings) -> Result<()> {
        self.settings = settings;
        self.set_autorun(self.settings.autorun_enabled)?;
        self.update_weather().await?;
        Ok(())
    }

    pub fn set_autorun(&self, autorun_enabled: bool) -> Result<()> {
        let path = std::env::current_exe()?;
        let auto = AutoLaunch::new("Tray Weather", &path.to_string_lossy(), &[] as &[&str]);
        let is_enabled = auto.is_enabled()?;

        if autorun_enabled && !is_enabled {
            auto.enable()?;
        } else if !autorun_enabled && is_enabled {
            auto.disable()?;
        }
        Ok(())
    }
}
