use std::sync::mpsc::{channel, Receiver};

use eframe::egui;
use rust_i18n::t;

use crate::{settings::Settings, PROGRAM_NAME};

pub(crate) struct ForecastWindow {}
impl ForecastWindow {
    fn new(settings: &Settings) -> Self {
        Self {}
    }
}

impl Default for ForecastWindow {
    fn default() -> Self {
        Self {}
    }
}

impl eframe::App for ForecastWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        todo!()
    }
}

pub(crate) fn show_forecast_window(settings: &Settings) -> Option<Settings> {
    let (tx, rx) = channel::<Option<Settings>>();
    let forecast_window = ForecastWindow::new(settings);

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([480.0, 320.0]),
        ..Default::default()
    };
    eframe::run_native(
        &t!("settings_title", name = PROGRAM_NAME),
        options,
        Box::new(|_cc| Ok(Box::new(forecast_window))),
    )
    .ok()?;

    if let Ok(msg) = rx.try_recv() {
        return msg;
    } else {
        return None;
    }
}
