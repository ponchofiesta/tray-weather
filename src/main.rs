#![windows_subsystem = "windows"]

// mod app;
mod error;
// mod gui;
mod settings;
mod weather;

use std::{
    fmt::Display,
    sync::{mpsc::channel, Arc, Mutex},
    thread::{sleep, spawn},
    time::Duration,
};

// use app::{TaskGuard, WeatherApp};
// use async_winit::{event_loop::EventLoop, ThreadUnsafe};
use error::{Error, Result};
// use gui::{forecast_window::show_forecast_window, settings_window::show_settings_window};
use log::{debug, trace};
use rust_i18n::t;
use settings::Settings;
use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem},
    MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent,
};
use vizia::prelude::*;

pub const PROGRAM_NAME: &str = "Tray Weather";

rust_i18n::i18n!("locales");

fn localization() {
    let locale = sys_locale::get_locale().unwrap_or_else(|| String::from("en"));
    debug!("Locale detected: {}", locale);
    rust_i18n::set_locale(&locale);
}

#[derive(Clone, PartialEq, Eq)]
enum Screen {
    None,
    Settings,
    Forecast,
}

impl Data for Screen {
    fn same(&self, other: &Self) -> bool {
        self.eq(other)
    }
}

#[derive(Lens)]
struct AppData {
    pub initialized: bool,
    pub visible: bool,
    pub title: String,
    pub screen: Screen,
}

impl AppData {
    pub fn new() -> Self {
        AppData {
            initialized: false,
            visible: false,
            screen: Screen::None,
            title: PROGRAM_NAME.to_owned(),
        }
    }
}

impl Model for AppData {
    fn event(&mut self, cx: &mut EventContext, event: &mut Event) {
        event.map(|app_event, meta| match app_event {
            // Message::Quit => cx.emit(WindowEvent::WindowClose),
            Message::Quit => (),
            Message::Update => todo!(),
            Message::ShowSettings { settings } => {
                self.screen = Screen::Settings;
                self.visible = true;
            }
            Message::SettingsClosed { settings } => {
                self.visible = false;
                self.screen = Screen::None;
            }
            Message::ShowForecast => {
                self.screen = Screen::Forecast;
                self.visible = true;
            }
        });
    }
}

enum Message {
    Update,
    ShowSettings { settings: Settings },
    SettingsClosed { settings: Settings },
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

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    localization();

    // Load app settings
    // let mut settings = Settings::default();
    // if let Err(_) = settings.load() {
    //     settings = show_settings_window(&settings).ok_or(Error::NoSettings)?;
    //     settings.save()?;
    // }
    let mut settings = Settings::default();
    let mut show_settings = false;
    if let Err(_) = settings.load() {
        // settings = show_settings_window(&settings).ok_or(Error::NoSettings)?;
        // settings.save()?;
        show_settings = true;
    }
    // let update_interval = Arc::new(Mutex::new(settings.update_interval));

    // show_forecast_window(&settings).unwrap();
    // return Ok(());

    // Build tray menu
    let item_update = MenuItem::with_id(MenuId::Update, t!("update"), true, None);
    let item_config = MenuItem::with_id(MenuId::Settings, t!("settings"), true, None);
    let item_exit = MenuItem::with_id(MenuId::Quit, t!("quit"), true, None);
    let menu = Menu::with_items(&[&item_update, &item_config, &item_exit])?;
    let tray = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_menu_on_left_click(false)
        .build()?;

    let (tx, rx) = channel();

    let menu_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(event) = MenuEvent::receiver().recv() {
                if event.id() == MenuId::Settings.to_string() {
                    let _ = menu_tx.send(Message::ShowSettings {
                        settings: Settings::default(),
                    });
                }
            }
        }
    });

    let tray_tx = tx.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(event) = TrayIconEvent::receiver().recv() {
                match event {
                    TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } => {
                        let _ = tray_tx.send(Message::ShowForecast);
                    }
                    _ => (),
                }
            }
        }
    });

    // let mut app = WeatherApp::new(settings, menu)?;

    // let event_loop: EventLoop<ThreadUnsafe> = EventLoop::new();
    // let window_target = event_loop.window_target().clone();
    // let (tx, mut rx) = tokio::sync::mpsc::channel(32);
    // let mut task_guard = TaskGuard::new();

    // let sleep_update_interval = update_interval.clone();

    // Initial weather update
    // app.update_weather().await?;

    // let setting_update_interval = update_interval.clone();

    // Run main event loop
    // event_loop.block_on(async move {
    //     loop {
    //         trace!("eventloop iteration starts");
    //         if let Some(msg) = rx.recv().await {
    //             match msg {
    //                 Message::Update => app.update_weather().await.unwrap(),
    //                 Message::ShowSettings => {
    //                     if let Some(new_settings) = show_settings_window(&app.settings) {
    //                         app.settings.update(&new_settings);
    //                         app.settings.save().expect("Could not save settings.");
    //                         *setting_update_interval.lock().unwrap() = app.settings.update_interval;
    //                         app.update_settings().await.unwrap();
    //                     }
    //                 }
    //                 Message::ShowForecast => show_forecast_window(&app.settings).unwrap(),
    //                 Message::Quit => window_target.exit().await,
    //             }
    //         }
    //     }
    // });
    let update_interval = Arc::new(Mutex::new(60));
    let sleep_update_interval = update_interval.clone();

    // let _ = Application::new(move |cx| {
    //     AppData::new().build(cx);

    //     // let mut app = WeatherApp::new(settings, menu);
    //     // let sleep_update_interval = sleep_update_interval.clone();

    //     cx.spawn(move |cx| loop {
    //         sleep(Duration::from_secs(
    //             *sleep_update_interval.lock().unwrap() * 60,
    //         ));
    //         let _ = cx.emit(Message::Update);
    //     });

    //     cx.spawn(move |cx| loop {
    //         if let Ok(event) = MenuEvent::receiver().recv() {
    //             let msg = if event.id() == MenuId::Update.to_string() {
    //                 Message::Update
    //             } else if event.id() == MenuId::Settings.to_string() {
    //                 Message::ShowSettings {
    //                     settings: settings.clone(),
    //                 }
    //             } else if event.id() == MenuId::Quit.to_string() {
    //                 Message::Quit
    //             } else {
    //                 continue;
    //             };
    //             let _ = cx.emit(msg);
    //         }
    //     });

    //     cx.spawn(|cx| loop {
    //         if let Ok(event) = TrayIconEvent::receiver().recv() {
    //             let msg = match event {
    //                 TrayIconEvent::Click {
    //                     button: MouseButton::Left,
    //                     button_state: MouseButtonState::Up,
    //                     ..
    //                 } => Message::ShowForecast,
    //                 _ => continue,
    //             };
    //             let _ = cx.emit(msg);
    //         }
    //     });

    //     Binding::new(cx, AppData::screen, |cx, screen| {
    //         match screen.get(cx) {
    //             Screen::Settings => settings_screen(cx),
    //             Screen::Forecast => forecast_screen(cx),
    //             Screen::None => {
    //                 VStack::new(cx, |cx| {});
    //             }
    //         };
    //     });
    // })
    // .title(AppData::title)
    // .visible(AppData::visible)
    // .inner_size((400, 200))
    // .run();

    loop {
        if let Ok(msg) = rx.recv() {
            match msg {
                Message::Update => todo!(),
                Message::ShowSettings { settings } => todo!(),
                Message::SettingsClosed { settings } => todo!(),
                Message::ShowForecast => todo!(),
                Message::Quit => todo!(),
            }
        }
    }

    Ok(())
}

fn win1() {
    let _ = Application::new(move |cx| {
        AppData::new().build(cx);

        // let mut app = WeatherApp::new(settings, menu);
        // let sleep_update_interval = sleep_update_interval.clone();

        Binding::new(cx, AppData::screen, |cx, screen| {
            match screen.get(cx) {
                Screen::Settings => settings_screen(cx),
                Screen::Forecast => forecast_screen(cx),
                Screen::None => {
                    VStack::new(cx, |cx| {});
                }
            };
        });
    })
    .title(AppData::title)
    .visible(AppData::visible)
    .inner_size((400, 200))
    .run();
}

fn win2() {
    let _ = Application::new(move |cx| {
        AppData::new().build(cx);

        // let mut app = WeatherApp::new(settings, menu);
        // let sleep_update_interval = sleep_update_interval.clone();

        Binding::new(cx, AppData::screen, |cx, screen| {
            match screen.get(cx) {
                Screen::Settings => settings_screen(cx),
                Screen::Forecast => forecast_screen(cx),
                Screen::None => {
                    VStack::new(cx, |cx| {});
                }
            };
        });
    })
    .title(AppData::title)
    .visible(AppData::visible)
    .inner_size((400, 200))
    .run();
}

pub fn forecast_screen(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Label::new(cx, "Forecast");
    });
}

pub fn settings_screen(cx: &mut Context) {
    VStack::new(cx, |cx| {
        Label::new(cx, "Settings");
        Button::new(cx, |cx| Label::new(cx, "Save")).on_press(|cx| {
            cx.emit(Message::SettingsClosed {
                settings: Settings::default(),
            })
        });
        Button::new(cx, |cx| {
            Label::new(cx, "Cancel").on_press(|cx| {
                cx.emit(Message::SettingsClosed {
                    settings: Settings::default(),
                })
            })
        });
    });
}
