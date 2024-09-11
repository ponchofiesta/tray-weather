use std::sync::mpsc::{channel, Sender};

use eframe::egui::{self, TextEdit};

use crate::Settings;

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
