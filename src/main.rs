#![windows_subsystem = "windows"]

mod app;
mod error;
mod gui;
mod settings;
mod weather;

use std::{sync::mpsc::channel, time::Duration};

use app::{TaskGuard, WeatherApp};
use async_winit::{event_loop::EventLoop, ThreadUnsafe};
use betrayer::{ClickType, Menu, MenuItem, TrayEvent, TrayIconBuilder};
use error::{Error, Result};
use gui::{forecast_window::show_forecast_window, settings_window::show_settings_window};
use log::{debug, trace};
use rust_i18n::t;
use settings::Settings;
use tokio::runtime::Runtime;
// use tray_icon::{
//     menu::{Menu, MenuEvent, MenuItem},
//     MouseButton, MouseButtonState, TrayIconEvent,
// };

pub const PROGRAM_NAME: &str = "Tray Weather";

const UPDATE_INTERVAL: u64 = 60 * 15;

rust_i18n::i18n!("locales");

fn localization() {
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en"));
    debug!("Locale detected: {}", locale);
    rust_i18n::set_locale(&locale);
}

#[derive(Clone)]
enum Message {
    Update,
    Settings,
    Forecast,
    Quit,
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
    // let item_update = MenuItem::new(t!("update"), true, None);
    // let item_config = MenuItem::new(t!("settings"), true, None);
    // let item_exit = MenuItem::new(t!("quit"), true, None);
    // let menu = Menu::with_items(&[&item_update, &item_config, &item_exit])?;
    let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    // let mut task_guard = TaskGuard::new();

    let tray_tx = tx.clone();

    let tray = TrayIconBuilder::new()
        .with_tooltip("Change Brightness")
        .with_menu(Menu::new([
            MenuItem::button("Update", Message::Update),
            MenuItem::button("Settings", Message::Settings),
            MenuItem::button("Quit", Message::Quit),
        ]))
        .build(move |event| {
            println!("tray");
            let _ = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(tray_tx.send(match event {
                    TrayEvent::Tray(ClickType::Left) => Some(Message::Forecast),
                    TrayEvent::Menu(e) => Some(e),
                    _ => None,
                }));
            ()
        })
        .unwrap();

    let mut app = WeatherApp::new(settings, tray)?;

    let event_loop: EventLoop<ThreadUnsafe> = EventLoop::new();
    let window_target = event_loop.window_target().clone();
    // let mut task_guard = TaskGuard::new();

    // Ticker for update interval
    // let timer_tx = tx.clone();
    // task_guard.spawn(|notify| {
    //     tokio::spawn(async move {
    //         loop {
    //             tokio::select! {
    //                 _ = notify.notified() => {
    //                     trace!("Timer task notified. Exiting...");
    //                     break;
    //                 }
    //                 _ = tokio::time::sleep(Duration::from_secs(UPDATE_INTERVAL)) => {
    //                     trace!("Timer task sleeped. Ticking...");
    //                     let _ = timer_tx.send(Some(Message::Update));
    //                 }
    //             }
    //         }
    //     })
    // });

    // Proxy for tray events
    // let tray_tx = tx.clone();
    // tokio::spawn(async move {
    //     loop {
    //         if let Ok(msg) = TrayIconEvent::receiver().recv() {
    //             println!("tray event");
    //             let _ = tray_tx.send(Message::Tray(msg)).await;
    //         }
    //     }
    // });

    // Proxy for menu events
    // let menu_tx = tx.clone();
    // tokio::spawn(async move {
    //     loop {
    //         if let Ok(msg) = MenuEvent::receiver().recv() {
    //             let _ = menu_tx.send(Message::Menu(msg)).await;
    //         }
    //     }
    // });

    // Initial weather update
    app.update_weather().await?;

    // Run main event loop
    event_loop.block_on(async move {
        loop {
            trace!("eventloop iteration starts");
            if let Some(Some(msg)) = rx.recv().await {
                match msg {
                    Message::Update => app.update_weather().await.unwrap(),
                    Message::Settings => {
                        if let Some(new_settings) = show_settings_window(&app.settings) {
                            app.settings.update(&new_settings);
                            app.settings.save().expect("Could not save settings.");
                            app.update_settings().await.unwrap();
                        }
                    }
                    Message::Forecast => show_forecast_window(&app.settings).unwrap(),
                    Message::Quit => window_target.exit().await,
                }
            }
        }
    });
}
