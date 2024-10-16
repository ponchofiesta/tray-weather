use std::fmt::Display;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    AutoLaunch(Box<dyn std::error::Error>),
    Io(std::io::Error),
    NoSettings,
    Other(Box<dyn std::error::Error>),
    Reqwest(reqwest::Error),
    TomlDe(toml::de::Error),
    TomlSer(toml::ser::Error),
    TrayIcon(tray_icon::Error),
    TrayIconMenu(tray_icon::menu::Error),
}

unsafe impl Sync for Error {}
unsafe impl Send for Error {}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Error::*;

        match self {
            AutoLaunch(err) => write!(f, "AutoLaunchError: {}", err),
            Io(io_error) => write!(f, "{io_error}"),
            NoSettings => write!(f, "No Settings were provided."),
            Other(err) => write!(f, "Other error: {}", err),
            Reqwest(err) => write!(f, "RequestError: {}", err),
            TomlDe(err) => write!(f, "TomlDeError: {}", err),
            TomlSer(err) => write!(f, "TomlSerError: {}", err),
            TrayIcon(err) => write!(f, "TrayIconError: {}", err),
            TrayIconMenu(err) => write!(f, "TrayIconMenuError: {}", err),
        }
    }
}

impl Error {
    pub fn other<E>(error: E) -> Error
    where
        E: Into<Box<dyn std::error::Error + Send + Sync>>,
    {
        Self::Other(error.into())
    }
}

impl From<auto_launch::Error> for Error {
    fn from(value: auto_launch::Error) -> Self {
        Error::AutoLaunch(Box::new(value))
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Error::Reqwest(value)
    }
}

impl From<tray_icon::Error> for Error {
    fn from(value: tray_icon::Error) -> Self {
        Error::TrayIcon(value)
    }
}

impl From<tray_icon::menu::Error> for Error {
    fn from(value: tray_icon::menu::Error) -> Self {
        Error::TrayIconMenu(value)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(value: toml::ser::Error) -> Self {
        Error::TomlSer(value)
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Error::TomlDe(value)
    }
}
