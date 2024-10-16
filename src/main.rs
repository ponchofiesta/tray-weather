#![windows_subsystem = "windows"]

mod app;
mod error;
mod gui;
mod settings;
mod weather;

use std::time::Duration;

use app::WeatherApp;
use async_winit::{event_loop::EventLoop, ThreadUnsafe};
use error::{Error, Result};
use gui::{show_settings_window, MenuMessage};
use log::{debug, trace};
use settings::Settings;
use tray_icon::menu::MenuEvent;

pub const PROGRAM_NAME: &str = "Tray Weather";

const UPDATE_INTERVAL: u64 = 60 * 15;

rust_i18n::i18n!("locales");

fn localization() {
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en"));
    debug!("Locale detected: {}", locale);
    rust_i18n::set_locale(&locale);
}

enum Message {
    Timer,
    Menu(MenuEvent),
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
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);

    let timer_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(UPDATE_INTERVAL)).await;
            let _ = timer_tx.send(Message::Timer).await;
        }
    });

    let menu_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(msg) = MenuEvent::receiver().recv() {
                let _ = menu_tx.send(Message::Menu(msg)).await;
            }
        }
    });

    event_loop.block_on(async move {
        loop {
            trace!("loop");
            if let Some(msg) = rx.recv().await {
                match msg {
                    Message::Menu(menuevent) => {
                        debug!("Menu event: {:?}", menuevent);
                        if menuevent.id()
                            == app
                                .tray_icon
                                .menu_items
                                .get(&MenuMessage::Update)
                                .unwrap()
                                .id()
                        {
                            app.update_weather().await.unwrap();
                        } else if menuevent.id()
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
                        } else if menuevent.id()
                            == app
                                .tray_icon
                                .menu_items
                                .get(&MenuMessage::Exit)
                                .unwrap()
                                .id()
                        {
                            window_target.exit().await;
                        }
                    }
                    Message::Timer => app.update_weather().await.unwrap(),
                }
            }
        }
    });
}
