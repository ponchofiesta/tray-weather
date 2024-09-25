mod app;
mod error;
mod gui;
mod settings;
mod weather;

use std::time::{Duration, Instant};

use app::WeatherApp;
use async_winit::{event_loop::EventLoop, ThreadUnsafe};
use error::{Error, Result};
use gui::{show_settings_window, MenuMessage};
use log::{debug, trace};
use settings::Settings;
use tokio::time::sleep;
use tray_icon::menu::MenuEvent;

pub const PROGRAM_NAME: &str = "Tray Weather";

const UPDATE_INTERVAL: u64 = 60 * 15;

rust_i18n::i18n!("locales");

fn localization() {
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en"));
    debug!("Locale detected: {}", locale);
    rust_i18n::set_locale(&locale);
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    localization();

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
        .checked_sub(Duration::from_secs(UPDATE_INTERVAL))
        .ok_or(Error::Instant)?;

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
