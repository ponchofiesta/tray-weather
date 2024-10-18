use std::sync::Arc;

use auto_launch::AutoLaunch;
use log::debug;
use tokio::sync::Notify;
use tray_icon::menu::Menu;

use crate::{
    error::Result, gui::weather_tray_icon::WeatherTrayIcon, settings::Settings,
    weather::get_weather,
};

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
            Ok(weather) => self.tray_icon.set_weather(
                &self.settings.location,
                &self.settings.icon_theme,
                &weather,
            )?,
            Err(err) => self.tray_icon.set_error(&format!("Fehler: {}", err))?,
        };
        Ok(())
    }

    pub async fn update_settings(&mut self) -> Result<()> {
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

pub struct TaskGuard {
    notify: Arc<Notify>,
    handles: Vec<tokio::task::JoinHandle<()>>,
}

impl TaskGuard {
    pub fn new() -> Self {
        Self {
            notify: Arc::new(Notify::new()),
            handles: vec![],
        }
    }

    pub fn spawn<F>(&mut self, f: F)
    where
        F: FnOnce(Arc<Notify>) -> tokio::task::JoinHandle<()> + Send + 'static,
    {
        let notify = Arc::clone(&self.notify);
        let handle = f(notify);
        self.handles.push(handle);
    }
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        // Benachrichtige alle Tasks, dass sie stoppen sollen
        self.notify.notify_waiters();

        // Warte auf das Beenden der Tasks
        for handle in self.handles.drain(..) {
            let _ = tokio::runtime::Handle::current().block_on(handle);
        }
    }
}
