use std::sync::mpsc::{channel, Receiver, Sender};

use iced::Application;
use rust_i18n::t;

use crate::{
    settings::Settings,
    weather::{search_location, Location},
    Result, PROGRAM_NAME,
};

use super::IconTheme;

enum SettingsScreen {
    Home,
    Location,
}

pub(crate) struct SettingsWindow {
    tx_window: Option<Sender<Option<Settings>>>,
    rx_locations: Receiver<Result<Vec<Location>>>,
    tx_locations: Sender<Result<Vec<Location>>>,
    location: Location,
    location_name: String,
    found_locations: Option<Vec<Location>>,
    icon_theme: IconTheme,
    autorun_enabled: bool,
    screen: SettingsScreen,
}

impl Default for SettingsWindow {
    fn default() -> Self {
        let locations_channel = channel();
        Self {
            tx_window: None,
            rx_locations: locations_channel.1,
            tx_locations: locations_channel.0,
            location: Default::default(),
            location_name: "".into(),
            found_locations: None,
            icon_theme: IconTheme::Monochrome,
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
            icon_theme: settings.icon_theme.clone(),
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
        ui.heading(t!("settings_heading"));

        setting_entry(ui, t!("location"), |ui| {
            let text = if self.location.id != 0 {
                self.location.to_human_readable()
            } else {
                t!("empty_location").into()
            };
            if ui.button(text).clicked() {
                self.screen = SettingsScreen::Location;
            }
        });

        setting_entry(ui, t!("icon_theme"), |ui| {
            ComboBox::from_id_source("icon_theme")
                .selected_text(&self.icon_theme.to_string())
                .show_ui(ui, |ui| {
                    IconTheme::iterator().cloned().for_each(|icon_theme| {
                        let text = icon_theme.to_string();
                        ui.selectable_value(&mut self.icon_theme, icon_theme, text);
                    });
                });
        });

        setting_entry(ui, t!("autostart", name = PROGRAM_NAME), |ui| {
            ui.add(Checkbox::without_text(&mut self.autorun_enabled));
        });

        ui.horizontal(|ui| {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::BOTTOM), |ui| {
                // reversed because of right_to_left
                let cancel_button = ui.button(t!("dialog.cancel"));
                let save_button = ui.button(t!("dialog.save"));

                if save_button.clicked() {
                    if let Some(tx) = &self.tx_window {
                        let settings = Settings {
                            location: self.location.clone(),
                            icon_theme: self.icon_theme.clone(),
                            autorun_enabled: self.autorun_enabled,
                        };
                        tx.send(Some(settings)).unwrap();
                    }
                    self.close_window(ctx);
                } else if cancel_button.clicked() {
                    self.close_window(ctx);
                }
            });
        });
    }

    fn location_screen(&mut self, ui: &mut Ui) {
        match self.rx_locations.try_recv() {
            Ok(response) => match response {
                Ok(found_locations) => self.found_locations = Some(found_locations),
                Err(e) => todo!("Could not get locations: {}", e),
            },
            Err(_) => (),
        }

        ui.heading(t!("location_heading"));

        ui.horizontal(|ui| {
            let location_label = ui.label(t!("location"));

            ui.add(
                TextEdit::singleline(&mut self.location_name)
                    .desired_width(80.0)
                    .margin(egui::Margin::symmetric(12.0, 8.0)),
            )
            .labelled_by(location_label.id);

            if ui.button(t!("search_location")).clicked() {
                let name: String = self.location_name.clone();
                let tx = self.tx_locations.clone();
                tokio::spawn(async move {
                    let results = search_location(&name, "de").await;
                    tx.send(results).unwrap();
                });
            }
        });

        ui.separator();

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
        // Styling
        let mut style: egui::Style = (*ctx.style()).clone();
        style.spacing.button_padding = egui::vec2(12.0, 8.0);
        style.spacing.item_spacing = egui::vec2(16.0, 8.0);
        ctx.set_style(style.clone());
        let frame = egui::Frame::none()
            .inner_margin(egui::Margin::same(16.0))
            .fill(style.visuals.panel_fill);

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            match self.screen {
                SettingsScreen::Home => self.settings_screen(ctx, ui),
                SettingsScreen::Location => self.location_screen(ui),
            };
        });
    }
}

fn setting_entry<R>(
    ui: &mut Ui,
    label: impl Into<egui::WidgetText>,
    add_contents: impl FnOnce(&mut Ui) -> R,
) {
    egui::Frame::default()
        .fill(egui::Color32::from_rgb(250, 250, 250))
        .stroke(egui::Stroke::new(
            0.5,
            egui::Color32::from_rgb(220, 220, 220),
        ))
        .rounding(egui::Rounding::same(4.0))
        .inner_margin(egui::Margin::symmetric(16.0, 20.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::left_to_right(egui::Align::TOP), |ui| {
                    ui.label(label);
                });
                ui.add_space(ui.available_width());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), add_contents);
            });
        });
}

pub(crate) fn show_settings_window(settings: &Settings) -> Option<Settings> {
    let (tx, rx) = channel::<Option<Settings>>();
    let settings_window = SettingsWindow::new(tx.clone(), settings);

    iced::application(
        SettingsWindow::title,
        SettingsWindow::update,
        SettingsWindow::view,
    )
    .centered()
    .run_with(settings_window)
    .ok()?;

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
