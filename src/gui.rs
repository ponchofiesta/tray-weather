use std::{
    collections::HashMap,
    sync::mpsc::{channel, Receiver, Sender},
};

use eframe::egui::{self, Label, TextEdit, Ui};
use log::debug;
use reqwest::Url;
use rust_i18n::t;
use tray_icon::{
    menu::{Menu, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};

use crate::{
    error::Error,
    weather::{CurrentWeather, Location, Results},
    Result, Settings, PROGRAM_NAME,
};

#[derive(Debug, PartialEq, Eq, Hash)]
pub(crate) enum MenuMessage {
    Update,
    Config,
    Exit,
}

pub(crate) struct WeatherTrayIcon {
    pub tray_icon: TrayIcon,
    pub menu_items: HashMap<MenuMessage, MenuItem>,
}

impl WeatherTrayIcon {
    pub fn new() -> Result<Self> {
        debug!("Building tray menu");
        let menu = Menu::new();
        let item_update = MenuItem::new(t!("update"), true, None);
        let item_config = MenuItem::new(t!("settings"), true, None);
        let item_exit = MenuItem::new(t!("quit"), true, None);
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

    pub fn set_weather(&self, location: &Location, weather: &CurrentWeather) -> Result<()> {
        debug!("Set weather: {:?}", &weather);
        self.tray_icon
            .set_icon(Icon::from_resource_name(weather.icon_name(), None).ok())?;
        self.tray_icon.set_tooltip(Some(format!(
            "{}: {} - {}",
            location.name,
            weather.temperature,
            weather.description()
        )))?;
        Ok(())
    }

    pub fn set_error(&self, msg: &str) -> Result<()> {
        debug!("Set error: {}", msg);
        self.tray_icon.set_tooltip(Some(msg))?;
        self.tray_icon
            .set_icon(Icon::from_resource_name("exclamation-circle", None).ok())?;
        Ok(())
    }
}

enum SettingsScreen {
    Home,
    Location,
}

pub(crate) struct SettingsWindow {
    tx_window: Option<Sender<Option<Settings>>>,
    rx_locations: Option<Receiver<Result<Vec<Location>>>>,
    tx_locations: Option<Sender<Result<Vec<Location>>>>,
    location: Location,
    location_name: String,
    found_locations: Option<Vec<Location>>,
    autorun_enabled: bool,
    screen: SettingsScreen,
}

impl Default for SettingsWindow {
    fn default() -> Self {
        let locations_channel = channel();
        Self {
            tx_window: None,
            rx_locations: Some(locations_channel.1),
            tx_locations: Some(locations_channel.0),
            location: Default::default(),
            location_name: "".into(),
            found_locations: None,
            autorun_enabled: false,
            screen: SettingsScreen::Home,
        }
    }
}

impl SettingsWindow {
    pub fn new(tx: Sender<Option<Settings>>, settings: &Settings) -> Self {
        SettingsWindow {
            tx_window: Some(tx),
            location: settings.location.clone(),
            autorun_enabled: settings.autorun_enabled,
            screen: SettingsScreen::Home,
            ..Default::default()
        }
    }
}

impl SettingsWindow {
    fn close_window(&self, ctx: &egui::Context) {
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        });
    }
}

impl SettingsWindow {
    fn settings_screen(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let location_label = ui.label(t!("location"));
                ui.add(Label::new(self.location.to_human_readable()))
                    .labelled_by(location_label.id);

                if ui.button(t!("new_location")).clicked() {
                    self.screen = SettingsScreen::Location;
                }
            });
        });
        ui.horizontal(|ui| {
            ui.checkbox(
                &mut self.autorun_enabled,
                t!("autostart", name = PROGRAM_NAME),
            );
        });
        ui.horizontal(|ui| {
            let save_button = ui.button(t!("dialog.save"));
            let cancel_button = ui.button(t!("dialog.cancel"));

            if save_button.clicked() {
                if let Some(tx) = &self.tx_window {
                    let settings = Settings {
                        location: self.location.clone(),
                        autorun_enabled: self.autorun_enabled,
                    };
                    tx.send(Some(settings)).unwrap();
                }
                self.close_window(ctx);
            } else if cancel_button.clicked() {
                self.close_window(ctx);
            }
        });
    }

    fn location_screen(&mut self, ui: &mut Ui) {
        match &self.rx_locations {
            Some(rx) => match rx.try_recv() {
                Ok(response) => match response {
                    Ok(found_locations) => self.found_locations = Some(found_locations),
                    Err(e) => todo!("Could not get locations: {}", e),
                },
                Err(_) => (),
            },
            None => (),
        };

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                let location_label = ui.label(t!("location"));
                ui.add(TextEdit::singleline(&mut self.location_name).desired_width(80.0))
                    .labelled_by(location_label.id);
            });
        });
        ui.horizontal(|ui| {
            if ui.button(t!("search_location")).clicked() {
                let name: String = self.location_name.clone();
                let tx = match &self.tx_locations {
                    Some(tx) => tx.clone(),
                    None => panic!(),
                };
                tokio::spawn(async move {
                    let results = search_location(&name, "de").await;
                    tx.send(results).unwrap();
                });
            }
        });
        ui.horizontal(|ui| {
            if let Some(locations) = &self.found_locations {
                ui.vertical(|ui| {
                    for location in locations {
                        if ui.button(location.to_human_readable()).clicked() {
                            self.location = location.clone();
                            self.screen = SettingsScreen::Home;
                        }
                    }
                });
            }
        });
    }
}

impl eframe::App for SettingsWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match self.screen {
                SettingsScreen::Home => self.settings_screen(ctx, ui),
                SettingsScreen::Location => self.location_screen(ui),
            };
        });
    }
}

pub(crate) fn show_settings_window(settings: &Settings) -> Option<Settings> {
    let (tx, rx) = channel::<Option<Settings>>();
    let settings_window = SettingsWindow::new(tx.clone(), settings);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };
    eframe::run_native(
        &t!("settings_title", name = PROGRAM_NAME),
        options,
        Box::new(|_cc| Ok(Box::new(settings_window))),
    )
    .ok()?;

    if let Ok(msg) = rx.try_recv() {
        return msg;
    } else {
        return None;
    }
}

pub(crate) async fn search_location(name: &str, lang: &str) -> Result<Vec<Location>> {
    let params = [
        ("name", name),
        ("language", lang),
        ("count", "10"),
        ("format", "json"),
    ];
    let url = Url::parse_with_params("https://geocoding-api.open-meteo.com/v1/search", &params)
        .map_err(|e| Error::other(e))?;
    let response = reqwest::get(url).await?.json::<Results>().await?;
    Ok(response.results)
}
