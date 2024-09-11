mod gui;
mod settings;
mod weather;

use std::{
    fmt::Display,
    time::{Duration, Instant},
};

use async_winit::{event_loop::EventLoop, ThreadUnsafe};
use gui::{show_settings_window, MenuMessage, WeatherTrayIcon};
use log::{debug, trace};
use reqwest;
use settings::Settings;
use tokio::time::sleep;
use tray_icon::menu::MenuEvent;
use weather::{CurrentWeather, WeatherResponse};

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    NoWeatherInfo,
    TrayIconMenuError(tray_icon::menu::Error),
    TrayIconError(tray_icon::Error),
    ReqwestError(reqwest::Error),
    IoError(std::io::Error),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;

        match self {
            NoWeatherInfo => write!(f, "Unbekannte Wetterbedingungen"),
            TrayIconMenuError(err) => write!(f, "TrayIconMenuError: {}", err),
            TrayIconError(err) => write!(f, "TrayIconError: {}", err),
            ReqwestError(err) => write!(f, "RequestError: {}", err),
            IoError(io_error) => write!(f, "{io_error}"),
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::IoError(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::ReqwestError(value)
    }
}

impl From<tray_icon::Error> for Error {
    fn from(value: tray_icon::Error) -> Self {
        Error::TrayIconError(value)
    }
}

impl From<tray_icon::menu::Error> for Error {
    fn from(value: tray_icon::menu::Error) -> Self {
        Error::TrayIconMenuError(value)
    }
}


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
async fn main() {
    env_logger::init();

    let mut settings = Settings::default();
    if settings.exists() {
        settings.load();
    } else {
        let new_settings = show_settings_window(&settings);
        if new_settings.is_none() {
            return;
        }
        settings = new_settings.unwrap();
        settings.save();
    }

    // Wolfsburg Ehmen
    // let location = (52.397120, 10.700460);
    let mut app = WeatherApp::new(settings.clone()).unwrap();

    let event_loop: EventLoop<ThreadUnsafe> = EventLoop::new();
    let window_target = event_loop.window_target().clone();

    let mut last = Instant::now()
        .checked_sub(Duration::from_secs(UPDATE_INTERVAL))
        .unwrap();

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
                        settings.save();
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
