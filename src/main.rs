#![windows_subsystem = "windows"]

mod app;
mod error;
mod gui;
mod settings;
mod weather;

use std::time::Duration;

use app::{TaskGuard, WeatherApp};
use async_winit::{event_loop::EventLoop, ThreadUnsafe};
use error::{Error, Result};
use gui::{forecast_window::show_forecast_window, settings_window::show_settings_window};
use log::{debug, trace};
use rust_i18n::t;
use settings::Settings;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    MouseButton, MouseButtonState, TrayIconEvent,
};

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
    Tray(TrayIconEvent),
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    localization();

    // Load app settings
    let mut settings = Settings::default();
    if let Err(_) = settings.load() {
        settings = show_settings_window(&settings).ok_or(Error::NoSettings)?;
        settings.save()?;
    }

    // show_forecast_window(&settings).unwrap();
    // return Ok(());

    // Build tray menu
    let item_update = MenuItem::new(t!("update"), true, None);
    let item_config = MenuItem::new(t!("settings"), true, None);
    let item_exit = MenuItem::new(t!("quit"), true, None);
    let menu = Menu::with_items(&[&item_update, &item_config, &item_exit])?;

    let mut app = WeatherApp::new(settings, menu)?;

    let event_loop: EventLoop<ThreadUnsafe> = EventLoop::new();
    let window_target = event_loop.window_target().clone();
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    let mut task_guard = TaskGuard::new();

    // Ticker for update interval
    let timer_tx = tx.clone();
    task_guard.spawn(|notify| {
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = notify.notified() => {
                        trace!("Timer task notified. Exiting...");
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_secs(UPDATE_INTERVAL)) => {
                        trace!("Timer task sleeped. Ticking...");
                        let _ = timer_tx.send(Message::Timer).await;
                    }
                }
            }
        })
    });

    // Proxy for tray events
    let tray_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(msg) = TrayIconEvent::receiver().recv() {
                println!("tray event");
                let _ = tray_tx.send(Message::Tray(msg)).await;
            }
        }
    });

    // Proxy for menu events
    let menu_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(msg) = MenuEvent::receiver().recv() {
                let _ = menu_tx.send(Message::Menu(msg)).await;
            }
        }
    });

    // Initial weather update
    app.update_weather().await?;

    // Run main event loop
    event_loop.block_on(async move {
        loop {
            trace!("eventloop iteration starts");
            if let Some(msg) = rx.recv().await {
                match msg {
                    // An item of tray menu was clicked
                    Message::Menu(menuevent) => {
                        debug!("Menu event: {:?}", menuevent);
                        if menuevent.id() == item_update.id() {
                            app.update_weather().await.unwrap();
                        } else if menuevent.id() == item_config.id() {
                            if let Some(new_settings) = show_settings_window(&app.settings) {
                                app.settings.update(&new_settings);
                                app.settings.save().expect("Could not save settings.");
                                app.update_settings().await.unwrap();
                            }
                        } else if menuevent.id() == item_exit.id() {
                            window_target.exit().await;
                        }
                    }

                    // Tray icon was clicked
                    Message::Tray(TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    }) => show_forecast_window(&app.settings).unwrap(),

                    // The timer ticked
                    Message::Timer => app.update_weather().await.unwrap(),

                    _ => (),
                }
            }
        }
    });
}
