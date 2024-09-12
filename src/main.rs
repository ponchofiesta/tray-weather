mod error;
mod gui;
mod settings;
mod weather;

use std::time::{Duration, Instant};

use async_winit::{event_loop::EventLoop, ThreadUnsafe};
use error::{Error, Result};
use gui::{show_settings_window, MenuMessage, WeatherTrayIcon};
use log::{debug, trace};
use reqwest;
use settings::Settings;
use tokio::time::sleep;
use tray_icon::menu::MenuEvent;
use weather::{CurrentWeather, WeatherResponse};

struct WeatherApp {
    settings: Settings,
    tray_icon: WeatherTrayIcon,
}

impl WeatherApp {
    fn new(settings: Settings) -> Result<Self> {
        let tray_icon = WeatherTrayIcon::new()?;
        Ok(WeatherApp {
            settings,
            tray_icon,
        })
    }

    async fn update_weather(&self) -> Result<()> {
        debug!("update_weather()");
        let weather = self.get_weather().await;
        match weather {
            Ok(weather) => self.tray_icon.set_weather(&weather)?,
            Err(err) => self.tray_icon.set_error(&format!("Fehler: {}", err))?,
        };
        Ok(())
    }

    async fn get_weather(&self) -> Result<CurrentWeather> {
        debug!("get_weather()");
        let url = format!(
            "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true",
            self.settings.latitude, self.settings.longitude
        );
        let response = reqwest::get(&url).await?.json::<WeatherResponse>().await?;
        Ok(response.current_weather)
    }

    async fn update_settings(&mut self, settings: Settings) -> Result<()> {
        self.settings = settings;
        self.update_weather().await?;
        Ok(())
    }
}

const UPDATE_INTERVAL: u64 = 60 * 15;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let mut settings = Settings::default();
    if settings.exists() {
        settings.load()?;
    } else {
        settings = show_settings_window(&settings).ok_or(Error::NoSettings)?;
        settings.save()?;
    }

    let mut app = WeatherApp::new(settings.clone())?;

    let event_loop: EventLoop<ThreadUnsafe> = EventLoop::new();
    let window_target = event_loop.window_target().clone();

    let mut last = Instant::now()
        .checked_sub(Duration::from_secs(UPDATE_INTERVAL)).ok_or(Error::Instant)?;

    event_loop.block_on(async move {
        loop {
            trace!("loop");
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                debug!("Menu event: {:?}", event);
                if event.id()
                    == app
                        .tray_icon
                        .menu_items
                        .get(&MenuMessage::Update)
                        .unwrap()
                        .id()
                {
                    app.update_weather().await.unwrap();
                } else if event.id()
                    == app
                        .tray_icon
                        .menu_items
                        .get(&MenuMessage::Config)
                        .unwrap()
                        .id()
                {
                    if let Some(settings) = show_settings_window(&settings) {
                        settings.save().expect("Could not save settings.");
                        app.update_settings(settings.clone()).await.unwrap();
                    }
                } else if event.id()
                    == app
                        .tray_icon
                        .menu_items
                        .get(&MenuMessage::Exit)
                        .unwrap()
                        .id()
                {
                    window_target.exit().await;
                }
            } else if last < Instant::now() - Duration::from_secs(UPDATE_INTERVAL) {
                last = Instant::now();
                app.update_weather().await.unwrap();
            }
            sleep(Duration::from_millis(500)).await;
        }
    });
}
