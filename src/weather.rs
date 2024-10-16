use std::borrow::Cow;

use crate::error::{Error, Result};
use image::load_from_memory_with_format;
use log::debug;
use reqwest::Url;
use rust_embed::Embed;
use rust_i18n::t;
use serde::{Deserialize, Serialize};
use tray_icon::Icon;

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub(crate) struct Location {
    pub id: u32,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub feature_code: String,
    pub country_code: String,
    pub admin1_id: Option<u32>,
    pub admin2_id: Option<u32>,
    pub admin3_id: Option<u32>,
    pub admin4_id: Option<u32>,
    pub timezone: String,
    pub population: Option<u32>,
    pub postcodes: Option<Vec<String>>,
    pub country_id: u32,
    pub country: String,
    pub admin1: Option<String>,
    pub admin2: Option<String>,
    pub admin3: Option<String>,
    pub admin4: Option<String>,
}

impl Location {
    pub fn to_human_readable(&self) -> String {
        std::iter::once(self.name.as_str())
            .chain(
                [
                    self.admin1.as_ref(),
                    self.admin2.as_ref(),
                    self.admin3.as_ref(),
                    self.admin4.as_ref(),
                ]
                .iter()
                .filter_map(|&opt| opt.map(String::as_str)),
            )
            .collect::<Vec<_>>()
            .join(", ")
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct Results {
    pub results: Vec<Location>,
}

/// Representation for OpenMeteo REST weather response object
#[derive(Deserialize, Debug)]
pub(crate) struct WeatherResponse {
    pub current_weather: CurrentWeather,
}

/// Representation for OpenMeteo REST current_weather object
#[derive(Deserialize, Debug)]
pub(crate) struct CurrentWeather {
    pub temperature: f64,
    pub weathercode: i32,
}

impl CurrentWeather {
    /// Get a description string for Open Meteo weather code
    pub fn description(&self) -> Cow<'_, str> {
        match self.weathercode {
            0 => t!("weather.clear_sky"),
            1 => t!("weather.mainly_clear"),
            2 => t!("weather.partly_cloudy"),
            3 => t!("weather.overcast"),
            45 => t!("weather.fog"),
            48 => t!("weather.rime_fog"),
            51 => t!("weather.light_drizzle"),
            53 => t!("weather.moderate_drizzle"),
            55 => t!("weather.dense_drizzle"),
            56 => t!("weather.light_freezing_drizzle"),
            57 => t!("weather.dense_freezing_drizzle"),
            61 => t!("weather.light_rain"),
            63 => t!("weather.moderate_rain"),
            65 => t!("weather.heavy_rain"),
            66 => t!("weather.light_freezing_rain"),
            67 => t!("weather.heavy_freezing_rain"),
            71 => t!("weather.light_snow"),
            73 => t!("weather.moderate_snow"),
            75 => t!("weather.heavy_snow"),
            77 => t!("weather.snow_grains"),
            80 => t!("weather.light_rain_showers"),
            81 => t!("weather.moderate_rain_showers"),
            82 => t!("weather.heavy_rain_showers"),
            85 => t!("weather.light_snow_showers"),
            86 => t!("weather.heavy_snow_showers"),
            95 => t!("weather.thunderstorm"),
            96 => t!("weather.thurderstorm_with_light_hail"),
            99 => t!("weather.thunderstorm_with_heavy_hail"),
            _ => t!("weather.unknown"),
        }
    }

    pub fn icon_name(&self) -> &str {
        match self.weathercode {
            0 => "clearsky_day",
            1 => "fair_day",
            2 => "partlycloudy_day",
            3 => "cloudy",
            45 => "fog",
            48 => "fog",
            51 => "lightrain",
            53 => "lightrain",
            55 => "lightrain",
            56 => "lightrain",
            57 => "lightrain",
            61 => "lightrain",
            63 => "rain",
            65 => "heavyrain",
            66 => "lightrain",
            67 => "heavyrain",
            71 => "lightsnow",
            73 => "snow",
            75 => "heavysnow",
            77 => "lightsnow",
            80 => "lightrain",
            81 => "rain",
            82 => "heavyrain",
            85 => "lightsleet",
            86 => "heavysleet",
            95 => "heavyrainandthunder",
            96 => "sleetandthunder",
            99 => "heavysleetandthunder",
            _ => "exclamation-circle",
        }
    }
}

/// Search a location name on Open Meteo
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

/// Get current weather on Open Meteo for specific [Location]
pub async fn get_weather(location: &Location) -> Result<CurrentWeather> {
    debug!("get_weather()");
    let params = [
        ("latitude", location.latitude.to_string()),
        ("longitude", location.longitude.to_string()),
        ("current_weather", "true".into()),
    ];
    let url = Url::parse_with_params("https://api.open-meteo.com/v1/forecast", &params)
        .map_err(|e| Error::other(e))?;
    let response = reqwest::get(url).await?.json::<WeatherResponse>().await?;
    Ok(response.current_weather)
}

#[derive(Embed)]
#[folder = "assets"]
#[include = "*.ico"]
pub struct EmbeddedFiles;

/// Load [Icon] from embeded file
pub fn get_icon(path: &str) -> Result<Icon> {
    let bytes = EmbeddedFiles::get(path)
        .ok_or_else(|| Error::other(format!("Icon file {path} not found.")))?;
    let img =
        load_from_memory_with_format(&bytes.data, image::ImageFormat::Ico).map_err(Error::other)?;
    let rgba = img.to_rgba8();
    let raw = rgba.into_raw();
    let icon = Icon::from_rgba(raw, img.width(), img.height()).map_err(Error::other)?;
    Ok(icon)
}
