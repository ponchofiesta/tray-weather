use std::sync::Arc;

use auto_launch::AutoLaunch;
use betrayer::TrayIcon;
use log::{debug, trace};
use tokio::sync::Notify;
// use tray_icon::menu::Menu;

use crate::{
    error::Result,
    gui::IconTheme,
    settings::Settings,
    weather::{get_current_weather, get_icon, CurrentWeather, Location},
    Message,
};

pub(crate) struct WeatherApp {
    pub settings: Settings,
    pub tray: TrayIcon<Message>,
}

impl WeatherApp {
    pub fn new(settings: Settings, tray: TrayIcon<Message>) -> Result<Self> {
        Ok(WeatherApp { settings, tray })
    }

    pub async fn update_weather(&self) -> Result<()> {
        debug!("update_weather()");
        let weather = get_current_weather(&self.settings.location).await;
        trace!("{:?}", weather);
        match weather {
            Ok(weather) => {
                self.set_weather(&self.settings.location, &self.settings.icon_theme, &weather)?
            }
            Err(err) => self.set_error(&format!("Fehler: {}", err))?,
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
        self.tray.set_icon(Some(icon));
        // self.tray.set_tooltip(Some(format!(
        //     "{}: {} - {}",
        //     location.name,
        //     weather.temperature,
        //     weather.description()
        // )));
        Ok(())
    }

    pub fn set_error(&self, msg: &str) -> Result<()> {
        debug!("Set error: {}", msg);
        // self.tray.set_tooltip(Some(msg));
        let icon = get_icon("tabler-icons/exclamation-circle.ico")?;
        self.tray.set_icon(Some(icon));
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
