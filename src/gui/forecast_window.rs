use std::sync::mpsc::{channel, Receiver, Sender};

use chrono::{Duration, Local, NaiveDate};
use eframe::egui::{self, Color32, Layout, Margin, RichText, TextBuffer, Ui};
use log::trace;
use rust_i18n::t;

use crate::{
    error::{Error, Result},
    settings::Settings,
    weather::{get_forecast, WeatherResponse},
    PROGRAM_NAME,
};

pub(crate) struct ForecastWindow {
    pub loading: bool,
    pub settings: Settings,
    pub rx: Receiver<Result<WeatherResponse>>,
    pub tx: Sender<Result<WeatherResponse>>,
    pub weather_response: Option<WeatherResponse>,
}

impl ForecastWindow {
    fn new(settings: Settings) -> Self {
        Self {
            settings,
            ..Default::default()
        }
    }

    fn update_weather(&mut self) {
        trace!("update_weather()");
        self.loading = true;
        let tx = self.tx.clone();
        let location = self.settings.location.clone();
        tokio::spawn(async move {
            let forecast = get_forecast(&location).await;
            tx.send(forecast).unwrap();
        });
    }
}

impl Default for ForecastWindow {
    fn default() -> Self {
        let (tx, rx) = channel();
        Self {
            loading: false,
            settings: Default::default(),
            rx,
            tx,
            weather_response: None,
        }
    }
}

fn day(
    ui: &mut Ui,
    day: &str,
    // TODO: weather icon
    // weathericon: &[u8],
    max: &str,
    min: &str,
    wind_speed: &str,
    // TODO: rain icon
    // rain_icon: DynamicImage,
    rain: &str,
) {
    egui::Frame::none()
        .stroke(egui::Stroke::new(1.0, Color32::from_rgb(240, 240, 240)))
        .show(ui, |ui| {
            ui.vertical(|ui| {
                egui::Frame::none()
                    .inner_margin(Margin::symmetric(10.0, 10.0))
                    .fill(Color32::from_rgb(1, 178, 235))
                    .show(ui, |ui| {
                        ui.label(RichText::new(day).color(Color32::from_rgb(255, 255, 255)));
                    });

                // ui.image(weathericon);

                egui::Frame::none()
                    .inner_margin(Margin::symmetric(10.0, 10.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(t!("max")));
                    });

                egui::Frame::none()
                    .inner_margin(Margin::symmetric(10.0, 10.0))
                    .fill(Color32::from_rgb(64, 255, 255))
                    .show(ui, |ui| {
                        ui.label(RichText::new(max).size(20.0));
                    });

                egui::Frame::none()
                    .inner_margin(Margin::symmetric(10.0, 10.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(t!("min")));
                    });

                egui::Frame::none()
                    .inner_margin(Margin::symmetric(10.0, 10.0))
                    .fill(Color32::from_rgb(242, 242, 242))
                    .show(ui, |ui| {
                        ui.label(RichText::new(min).size(16.0));
                    });

                // wind dir

                egui::Frame::none()
                    .inner_margin(Margin::symmetric(10.0, 10.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(wind_speed).heading());
                    });

                // ui.image(rain_icon);

                egui::Frame::none()
                    .inner_margin(Margin::symmetric(10.0, 10.0))
                    .show(ui, |ui| {
                        ui.label(RichText::new(rain));
                    });
            });
        });
}

fn human_day(date: &NaiveDate) -> String {
    let today = Local::now().date_naive();
    if date == &today {
        String::from(t!("today"))
    } else if date == &(today + Duration::days(1)) {
        String::from(t!("tomorrow"))
    } else {
        // TODO: better date formatting
        format!("{}", date.format(t!("date_format").as_str()))
    }
}

impl eframe::App for ForecastWindow {
    fn update(&mut self, ctx: &eframe::egui::Context, frame: &mut eframe::Frame) {
        // Styling
        let mut style: egui::Style = (*ctx.style()).clone();
        // style.spacing.item_spacing = egui::vec2(16.0, 8.0);
        ctx.set_style(style.clone());

        let frame = egui::Frame::none()
            .inner_margin(egui::Margin::same(16.0))
            .fill(style.visuals.panel_fill);

        egui::CentralPanel::default().frame(frame).show(ctx, |ui| {
            if self.loading {
                match self.rx.try_recv() {
                    Ok(response) => {
                        self.loading = false;
                        match response {
                            Ok(weather_response) => self.weather_response = Some(weather_response),
                            Err(e) => todo!("Could not get forecast: {}", e),
                        }
                    }
                    Err(_) => (),
                };

                ui.with_layout(
                    Layout::centered_and_justified(egui::Direction::LeftToRight),
                    |ui| {
                        ui.label(t!("loading"));
                    },
                );
            } else {
                if let Some(ref weather_response) = self.weather_response {
                    // ui.label(format!("{weather_response:?}"));
                    // days forecast
                    ui.horizontal_top(|ui| {
                        if let Some(ref daily) = weather_response.daily {
                            for (i, time) in daily.time.iter().enumerate() {
                                day(
                                    ui,
                                    &human_day(&time),
                                    &daily.temperature_2m_max[i].to_string(),
                                    &daily.temperature_2m_min[i].to_string(),
                                    &daily.wind_speed_10m_max[i].to_string(),
                                    &daily.precipitation_sum[i].to_string(),
                                );
                            }
                        }
                    });
                }
            }
        });
    }
}

pub(crate) fn show_forecast_window(settings: &Settings) -> Result<()> {
    let mut forecast_window = ForecastWindow::new(settings.clone());
    forecast_window.update_weather();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([480.0, 320.0]),
        ..Default::default()
    };
    eframe::run_native(
        &t!("forecast_title", name = PROGRAM_NAME),
        options,
        Box::new(|_cc| Ok(Box::new(forecast_window))),
    )
    .map_err(|e| Error::other(format!("eframe::run_native() failed: {}", e)))
}
