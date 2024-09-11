use std::{collections::HashMap, sync::mpsc::{channel, Sender}};

use eframe::egui::{self, TextEdit};
use log::debug;
use tray_icon::{menu::{Menu, MenuItem}, Icon, TrayIcon, TrayIconBuilder};

use crate::{weather::CurrentWeather, Settings, Result};


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

    pub fn set_weather(&self, weather: &CurrentWeather) -> Result<()> {
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

    pub fn set_error(&self, msg: &str) -> Result<()> {
        debug!("Set error: {}", msg);
        self.tray_icon.set_tooltip(Some(msg))?;
        self.tray_icon
            .set_icon(Icon::from_resource_name("exclamation-circle", None).ok())?;
        Ok(())
    }
}


pub(crate) struct SettingsWindow<T> {
    tx: Option<Sender<T>>,
    latitude: String,
    longitude: String,
}

impl<T> Default for SettingsWindow<T> {
    fn default() -> Self {
        Self {
            tx: None,
            latitude: String::new(),
            longitude: String::new(),
        }
    }
}

impl<T> SettingsWindow<T> {
    pub fn new(tx: Sender<T>, settings: &Settings) -> Self {
        SettingsWindow {
            tx: Some(tx),
            latitude: settings.latitude.clone(),
            longitude: settings.longitude.clone(),
        }
    }
}

impl eframe::App for SettingsWindow<Option<Settings>> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    let latitude_label = ui.label("Breite: ");
                    ui.add(TextEdit::singleline(&mut self.latitude).desired_width(60.0))
                        .labelled_by(latitude_label.id);
                });
                ui.vertical(|ui| {
                    let longitude_label = ui.label("Länge: ");
                    ui.add(TextEdit::singleline(&mut self.longitude).desired_width(60.0))
                        .labelled_by(longitude_label.id);
                });
            });
            ui.horizontal(|ui| {
                let save_button = ui.button("Save");
                let cancel_button = ui.button("Cancel");

                if save_button.clicked() {
                    if let Some(tx) = &self.tx {
                        let settings = Settings {
                            latitude: self.latitude.clone(),
                            longitude: self.longitude.clone(),
                        };
                        tx.send(Some(settings)).unwrap();
                    }
                } else if cancel_button.clicked() {
                    todo!("Close window");
                }
            });
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
        "Tray Weather: Settings",
        options,
        Box::new(|_cc| Ok(Box::new(settings_window))),
    )
    .unwrap();

    if let Ok(msg) = rx.try_recv() {
        return msg;
    } else {
        return None;
    }
}
