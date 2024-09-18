use auto_launch::AutoLaunch;
use log::debug;
use reqwest::Url;

use crate::{error::{Error, Result}, gui::WeatherTrayIcon, settings::Settings, weather::{CurrentWeather, WeatherResponse}};

pub(crate) struct WeatherApp {
    pub settings: Settings,
    pub tray_icon: WeatherTrayIcon,
}

impl WeatherApp {
    pub fn new(settings: Settings) -> Result<Self> {
        let tray_icon = WeatherTrayIcon::new()?;
        Ok(WeatherApp {
            settings,
            tray_icon,
        })
    }

    pub async fn update_weather(&self) -> Result<()> {
        debug!("update_weather()");
        let weather = self.get_weather().await;
        match weather {
            Ok(weather) => self.tray_icon.set_weather(&weather)?,
            Err(err) => self.tray_icon.set_error(&format!("Fehler: {}", err))?,
        };
        Ok(())
    }

    pub async fn get_weather(&self) -> Result<CurrentWeather> {
        debug!("get_weather()");
        let params = [
            ("latitude", self.settings.latitude.as_str()),
            ("longitude", &self.settings.longitude.as_str()),
            ("current_weather", "true"),
        ];
        let url = Url::parse_with_params("https://api.open-meteo.com/v1/forecast", &params)
            .map_err(|e| Error::other(e))?;
        let response = reqwest::get(url).await?.json::<WeatherResponse>().await?;
        Ok(response.current_weather)
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
        if autorun_enabled {
            auto.enable()?;
        } else {
            auto.disable()?;
        }
        Ok(())
    }
}
