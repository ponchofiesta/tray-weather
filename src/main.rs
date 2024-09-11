mod settings_window;
mod settings;

use std::{
    collections::HashMap, fmt::Display, process::exit, time::{Duration, Instant}
};

use async_winit::{event_loop::EventLoop, ThreadUnsafe};
use log::{debug, trace};
use reqwest;
use serde::Deserialize;
use settings::Settings;
use settings_window::show_settings_window;
use tokio::time::sleep;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

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

#[derive(Deserialize, Debug)]
struct WeatherResponse {
    current_weather: CurrentWeather,
}

#[derive(Deserialize, Debug)]
struct CurrentWeather {
    temperature: f64,
    weathercode: i32,
}

impl CurrentWeather {
    fn description(&self) -> &str {
        match self.weathercode {
            0 => "Klarer Himmel",
            1 => "Überwiegend klar",
            2 => "Teilweise bewölkt",
            3 => "Bewölkt",
            45 => "Nebel",
            48 => "Reif-Nebel",
            51 => "Leichter Nieselregen",
            53 => "Mäßiger Nieselregen",
            55 => "Starker Nieselregen",
            56 => "Leichter gefrierender Nieselregen",
            57 => "Starker gefrierender Nieselregen",
            61 => "Leichter Regen",
            63 => "Mäßiger Regen",
            65 => "Starker Regen",
            66 => "Leichter gefrierender Regen",
            67 => "Starker gefrierender Regen",
            71 => "Leichter Schneefall",
            73 => "Mäßiger Schneefall",
            75 => "Starker Schneefall",
            77 => "Schneekörner",
            80 => "Leichte Regenschauer",
            81 => "Mäßige Regenschauer",
            82 => "Heftige Regenschauer",
            85 => "Leichte Schneeschauer",
            86 => "Starke Schneeschauer",
            95 => "Gewitter",
            96 => "Gewitter mit leichtem Hagel",
            99 => "Gewitter mit starkem Hagel",
            _ => "Unbekannte Wetterbedingungen",
        }
    }

    fn icon_name(&self) -> &str {
        match self.weathercode {
            0 => "clearsky_day",
            1 => "fair_day",
            2 => "partlycloudy_day",
            3 => "cloudy",
            45 => "fog",
            48 => "fog",
            51 => "lightrain",
            53 => "lightrain",
            55 => "lightrain",
            56 => "lightrain",
            57 => "lightrain",
            61 => "lightrain",
            63 => "rain",
            65 => "heavyrain",
            66 => "lightrain",
            67 => "heavyrain",
            71 => "lightsnow",
            73 => "snow",
            75 => "heavysnow",
            77 => "lightsnow",
            80 => "lightrain",
            81 => "rain",
            82 => "heavyrain",
            85 => "lightsleet",
            86 => "heavysleet",
            95 => "heavyrainandthunder",
            96 => "sleetandthunder",
            99 => "heavysleetandthunder",
            _ => "exclamation-circle",
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
enum MenuMessage {
    Update,
    Config,
    Exit,
}

struct WeatherTrayIcon {
    tray_icon: TrayIcon,
    menu_items: HashMap<MenuMessage, MenuItem>,
}

impl WeatherTrayIcon {
    fn new() -> Result<Self> {
        debug!("Building tray menu");
        let menu = Menu::new();
        let item_update = MenuItem::new("Aktualisieren", true, None);
        let item_config = MenuItem::new("Konfigurieren", true, None);
        let item_exit = MenuItem::new("Beenden", true, None);
        menu.append(&item_update)?;
        menu.append(&item_config)?;
        menu.append(&item_exit)?;

        let mut menu_items = HashMap::new();
        menu_items.insert(MenuMessage::Update, item_update);
        menu_items.insert(MenuMessage::Config, item_config);
        menu_items.insert(MenuMessage::Exit, item_exit);

        Ok(WeatherTrayIcon {
            tray_icon: TrayIconBuilder::new().with_menu(Box::new(menu)).build()?,
            menu_items,
        })
    }

    fn set_weather(&self, weather: &CurrentWeather) -> Result<()> {
        debug!("Set weather: {:?}", &weather);
        self.tray_icon
            .set_icon(Icon::from_resource_name(weather.icon_name(), None).ok())?;
        self.tray_icon.set_tooltip(Some(format!(
            "{} - {}",
            weather.temperature,
            weather.description()
        )))?;
        Ok(())
    }

    fn set_error(&self, msg: &str) -> Result<()> {
        debug!("Set error: {}", msg);
        self.tray_icon.set_tooltip(Some(msg))?;
        self.tray_icon
            .set_icon(Icon::from_resource_name("exclamation-circle", None).ok())?;
        Ok(())
    }
}

struct WeatherApp {
    location: (f32, f32),
    tray_icon: WeatherTrayIcon,
}

impl WeatherApp {
    fn new(location: (f32, f32)) -> Result<Self> {
        let tray_icon = WeatherTrayIcon::new()?;
        Ok(WeatherApp {
            location,
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
            self.location.0, self.location.1
        );
        let response = reqwest::get(&url).await?.json::<WeatherResponse>().await?;
        Ok(response.current_weather)
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
    let location = (52.397120, 10.700460);
    let app = WeatherApp::new(location).unwrap();

    let event_loop: EventLoop<ThreadUnsafe> = EventLoop::new();
    let window_target = event_loop.window_target().clone();

    let mut last = Instant::now().checked_sub(Duration::from_secs(UPDATE_INTERVAL)).unwrap();

    event_loop.block_on(async move {
        let menu_items = &app.tray_icon.menu_items;
        let update_menu_id = menu_items.get(&MenuMessage::Update).unwrap().id();
        let config_menu_id = menu_items.get(&MenuMessage::Config).unwrap().id();
        let exit_menu_id = menu_items.get(&MenuMessage::Exit).unwrap().id();
        loop {
            trace!("loop");
            if let Ok(event) = MenuEvent::receiver().try_recv() {
                debug!("Menu event: {:?}", event);
                if event.id() == update_menu_id {
                    app.update_weather().await.unwrap();
                } else if event.id() == config_menu_id {
                    if let Some(settings) = show_settings_window(&settings) {
                        settings.save();
                    }
                } else if event.id() == exit_menu_id {
                    window_target.exit().await;
                }
            } else if last < Instant::now() - Duration::from_secs(UPDATE_INTERVAL) {
                last = Instant::now();
                app.update_weather().await.unwrap();
            }
            sleep(Duration::from_millis(1000)).await;
        }
    });
}
