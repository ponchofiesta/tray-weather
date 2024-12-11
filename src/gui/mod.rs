use std::slice::Iter;

use serde::{Deserialize, Serialize};

pub mod forecast_window;
pub mod settings_window;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum IconTheme {
    Metno,
    Monochrome,
}

impl ToString for IconTheme {
    fn to_string(&self) -> String {
        match self {
            IconTheme::Metno => String::from("metno"),
            IconTheme::Monochrome => String::from("monochrome"),
        }
    }
}

impl IconTheme {
    pub fn iterator() -> Iter<'static, IconTheme> {
        use IconTheme::*;
        static ICON_THEMES: [IconTheme; 2] = [Metno, Monochrome];
        ICON_THEMES.iter()
    }
}
