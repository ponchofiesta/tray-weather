#![windows_subsystem = "windows"]

mod app;
mod error;
mod gui;
mod settings;
mod weather;

use std::{
    sync::{mpsc::channel, Arc, Mutex},
    thread::{sleep, spawn},
    time::Duration,
};

use app::WeatherApp;
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

rust_i18n::i18n!("locales");

fn localization() {
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en"));
    debug!("Locale detected: {}", locale);
    rust_i18n::set_locale(&locale);
}

enum Message {
    Update,
    ShowSettings,
    ShowForecast,
    Quit,
}

enum MenuId {
    Update,
    Settings,
    Quit,
}

impl ToString for MenuId {
    fn to_string(&self) -> String {
        use MenuId::*;
        String::from(match self {
            Update => "update",
            Settings => "settings",
            Quit => "quit",
        })
    }
}

fn main() -> Result<()> {
    env_logger::init();
    localization();

    // Load app settings
    let mut settings = Settings::default();
    if let Err(_) = settings.load() {
        settings = show_settings_window(&settings).ok_or(Error::NoSettings)?;
        settings.save()?;
    }

    let update_interval = Arc::new(Mutex::new(settings.update_interval));

    // show_forecast_window(&settings).unwrap();
    // return Ok(());

    // Build tray menu
    let item_update = MenuItem::with_id(MenuId::Update, t!("update"), true, None);
    let item_config = MenuItem::with_id(MenuId::Settings, t!("settings"), true, None);
    let item_exit = MenuItem::with_id(MenuId::Quit, t!("quit"), true, None);
    let menu = Menu::with_items(&[&item_update, &item_config, &item_exit])?;

    let mut app = WeatherApp::new(settings, menu)?;

    let event_loop: EventLoop<ThreadUnsafe> = EventLoop::new();
    let window_target = event_loop.window_target().clone();
    let (tx, mut rx) = channel();
    // let mut task_guard = TaskGuard::new();

    let sleep_update_interval = update_interval.clone();

    // Ticker for update interval
    let timer_tx = tx.clone();
    spawn(move || loop {
        sleep(Duration::from_secs(
            *sleep_update_interval.lock().unwrap() * 60,
        ));
        trace!("Timer task sleeped. Ticking...");
        let _ = timer_tx.send(Message::Update);
    });
    // task_guard.spawn(|notify| {
    //     spawn(async move {
    //         loop {
    //             tokio::select! {
    //                 _ = notify.notified() => {
    //                     trace!("Timer task notified. Exiting...");
    //                     break;
    //                 }
    //                 _ = tokio::time::sleep(Duration::from_secs(*sleep_update_interval.lock().unwrap() * 60)) => {
    //                     trace!("Timer task sleeped. Ticking...");
    //                     let _ = timer_tx.send(Message::Update).await;
    //                 }
    //             }
    //         }
    //     })
    // });

    // Proxy for tray events
    let tray_tx = tx.clone();
    spawn(move || loop {
        if let Ok(event) = TrayIconEvent::receiver().recv() {
            println!("tray event");
            let msg = match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => Message::ShowForecast,
                _ => continue,
            };
            let _ = tray_tx.send(msg);
        }
    });
    // tokio::spawn(async move {
    //     loop {
    //         if let Ok(event) = TrayIconEvent::receiver().recv() {
    //             println!("tray event");
    //             let msg = match event {
    //                 TrayIconEvent::Click {
    //                     button: MouseButton::Left,
    //                     button_state: MouseButtonState::Up,
    //                     ..
    //                 } => Message::ShowForecast,
    //                 _ => continue,
    //             };
    //             let _ = tray_tx.send(msg).await;
    //         }
    //     }
    // });

    // Proxy for menu events
    let menu_tx = tx.clone();
    spawn(move || loop {
        if let Ok(event) = MenuEvent::receiver().recv() {
            let msg = if event.id() == MenuId::Update.to_string() {
                Message::Update
            } else if event.id() == MenuId::Settings.to_string() {
                Message::ShowSettings
            } else if event.id() == MenuId::Quit.to_string() {
                Message::Quit
            } else {
                continue;
            };
            let _ = menu_tx.send(msg);
        }
    });
    // tokio::spawn(async move {
    //     loop {
    //         if let Ok(event) = MenuEvent::receiver().recv() {
    //             let msg = if event.id() == MenuId::Update.to_string() {
    //                 Message::Update
    //             } else if event.id() == MenuId::Settings.to_string() {
    //                 Message::ShowSettings
    //             } else if event.id() == MenuId::Quit.to_string() {
    //                 Message::Quit
    //             } else {
    //                 continue;
    //             };
    //             let _ = menu_tx.send(msg).await;
    //         }
    //     }
    // });

    // Initial weather update
    let _ = futures_lite::future::block_on(app.update_weather());

    let setting_update_interval = update_interval.clone();

    // Run main event loop
    event_loop.block_on(async move {
        loop {
            trace!("eventloop iteration starts");
            if let Ok(msg) = rx.recv() {
                match msg {
                    Message::Update => app.update_weather().await.unwrap(),
                    Message::ShowSettings => {
                        if let Some(new_settings) = show_settings_window(&app.settings) {
                            app.settings.update(&new_settings);
                            app.settings.save().expect("Could not save settings.");
                            *setting_update_interval.lock().unwrap() = app.settings.update_interval;
                            app.update_settings().await.unwrap();
                        }
                    }
                    Message::ShowForecast => show_forecast_window(&app.settings).unwrap(),
                    Message::Quit => window_target.exit().await,
                }
            }
        }
    });
}
