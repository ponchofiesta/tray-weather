use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Location {
    pub id: u32,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub elevation: f64,
    pub feature_code: String,
    pub country_code: String,
    pub admin1_id: u32,
    pub admin2_id: u32,
    pub admin3_id: u32,
    pub admin4_id: u32,
    pub timezone: String,
    pub population: u32,
    pub postcodes: Vec<String>,
    pub country_id: u32,
    pub country: String,
    pub admin1: String,
    pub admin2: String,
    pub admin3: String,
    pub admin4: String,
}

#[derive(Deserialize, Debug)]
pub(crate) struct WeatherResponse {
    pub current_weather: CurrentWeather,
}

#[derive(Deserialize, Debug)]
pub(crate) struct CurrentWeather {
    pub temperature: f64,
    pub weathercode: i32,
}

impl CurrentWeather {
    pub fn description(&self) -> &str {
        match self.weathercode {
            0 => "Klarer Himmel",
            1 => "Überwiegend klar",
            2 => "Teilweise bewölkt",
            3 => "Bewölkt",
            45 => "Nebel",
            48 => "Reif-Nebel",
            51 => "Leichter Nieselregen",
            53 => "Mäßiger Nieselregen",
            55 => "Starker Nieselregen",
            56 => "Leichter gefrierender Nieselregen",
            57 => "Starker gefrierender Nieselregen",
            61 => "Leichter Regen",
            63 => "Mäßiger Regen",
            65 => "Starker Regen",
            66 => "Leichter gefrierender Regen",
            67 => "Starker gefrierender Regen",
            71 => "Leichter Schneefall",
            73 => "Mäßiger Schneefall",
            75 => "Starker Schneefall",
            77 => "Schneekörner",
            80 => "Leichte Regenschauer",
            81 => "Mäßige Regenschauer",
            82 => "Heftige Regenschauer",
            85 => "Leichte Schneeschauer",
            86 => "Starke Schneeschauer",
            95 => "Gewitter",
            96 => "Gewitter mit leichtem Hagel",
            99 => "Gewitter mit starkem Hagel",
            _ => "Unbekannte Wetterbedingungen",
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
