use log::debug;
use tray_icon::{menu::Menu, TrayIcon, TrayIconBuilder};

use crate::error::Result;
use crate::weather::{get_icon, CurrentWeather, Location};

use super::IconTheme;

pub(crate) struct WeatherTrayIcon {
    pub tray_icon: TrayIcon,
}

impl WeatherTrayIcon {
    pub fn new(menu: Menu) -> Result<Self> {
        debug!("Building tray menu");
        Ok(WeatherTrayIcon {
            tray_icon: TrayIconBuilder::new()
                .with_menu(Box::new(menu))
                .with_menu_on_left_click(false)
                .build()?,
        })
    }

    pub fn set_weather(
        &self,
        location: &Location,
        icon_theme: &IconTheme,
        weather: &CurrentWeather,
    ) -> Result<()> {
        debug!("Set weather: {:?}", &weather);
        let icon_path = format!(
            "weathericons/{}/ico/{}.ico",
            icon_theme.to_string(),
            weather.icon_name()
        );
        let icon = get_icon(&icon_path)?;
        self.tray_icon.set_icon(Some(icon))?;
        self.tray_icon.set_tooltip(Some(format!(
            "{}: {} - {}",
            location.name,
            weather.temperature,
            weather.description()
        )))?;
        Ok(())
    }

    pub fn set_error(&self, msg: &str) -> Result<()> {
        debug!("Set error: {}", msg);
        self.tray_icon.set_tooltip(Some(msg))?;
        let icon = get_icon("tabler-icons/exclamation-circle.ico")?;
        self.tray_icon.set_icon(Some(icon))?;
        Ok(())
    }
}
