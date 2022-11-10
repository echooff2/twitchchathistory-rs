extern crate config as configlib;

use std::{error::Error, fmt::Display};

use derivative::Derivative;
use error_stack::IntoReport;
use log::LevelFilter;
use serde_derive::Deserialize;
use tokio::sync::RwLock;

// TODO try https://crates.io/crates/const-default instead
pub static CONFIG: RwLock<Config> = RwLock::const_new(Config::const_default());

#[derive(Deserialize, Derivative)]
#[derivative(Default)]
pub struct Config {
    pub database: DatabaseConfig,
    pub twitch_api: TwitchApi,
    pub channels: Vec<String>,
    #[serde(default)]
    pub log_level: LogLevelFilter,
    #[derivative(Default(value = "3"))]
    pub database_op_retry_limit: u8,
}

impl Config {
    const fn const_default() -> Self {
        Config {
            database: DatabaseConfig::const_default(),
            twitch_api: TwitchApi {
                client_id: String::new(),
                client_secret: String::new(),
            },
            channels: vec![],
            log_level: LogLevelFilter::const_default(),
            database_op_retry_limit: 3,
        }
    }
}

#[derive(Default, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub username: String,
    pub password: String,
    pub db: String,
}

impl DatabaseConfig {
    const fn const_default() -> Self {
        Self {
            url: String::new(),
            username: String::new(),
            password: String::new(),
            db: String::new(),
        }
    }
}

#[derive(Default, Deserialize)]
pub struct TwitchApi {
    pub client_id: String,
    pub client_secret: String,
}

#[repr(usize)]
#[derive(Deserialize, Clone, Copy)]
pub enum LogLevelFilter {
    /// A level lower than all log levels.
    Off,
    /// Corresponds to the `Error` log level.
    Error,
    /// Corresponds to the `Warn` log level.
    Warn,
    /// Corresponds to the `Info` log level.
    Info,
    /// Corresponds to the `Debug` log level.
    Debug,
    /// Corresponds to the `Trace` log level.
    Trace,
}

impl Default for LogLevelFilter {
    fn default() -> Self {
        Self::const_default()
    }
}

impl LogLevelFilter {
    const fn const_default() -> Self {
        LogLevelFilter::Info
    }
}

impl From<LogLevelFilter> for LevelFilter {
    fn from(level: LogLevelFilter) -> Self {
        match level {
            LogLevelFilter::Off => LevelFilter::Off,
            LogLevelFilter::Error => LevelFilter::Error,
            LogLevelFilter::Warn => LevelFilter::Warn,
            LogLevelFilter::Info => LevelFilter::Info,
            LogLevelFilter::Debug => LevelFilter::Debug,
            LogLevelFilter::Trace => LevelFilter::Trace,
        }
    }
}

type FieldName = String;
type ParseErrorCause = String;
type OptionalMessage = Option<String>;

#[derive(Debug)]
pub enum ConfigError {
    MissingField(FieldName),
    FileParseError(ParseErrorCause),
    FieldTypeError(Option<FieldName>),
    Other(OptionalMessage),
}

impl Error for ConfigError {}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingField(field) => write!(f, "Missing field {} in config", field),
            Self::FileParseError(cause) => write!(f, "Misconfigured config file: {}", cause),
            Self::FieldTypeError(field) => write!(
                f,
                "Field {} has wrong value",
                field.as_ref().map(|v| &v[..]).unwrap_or_default()
            ),
            Self::Other(msg) => match msg {
                Some(msg) => write!(f, "Config error: {msg}"),
                None => write!(
                    f,
                    "Unexpect error occured while loading configuration. Please try again!"
                ),
            },
        }
    }
}

impl From<&configlib::ConfigError> for ConfigError {
    fn from(v: &configlib::ConfigError) -> Self {
        match v {
            configlib::ConfigError::NotFound(field) => ConfigError::MissingField(field.clone()),
            configlib::ConfigError::FileParse { cause, .. } => {
                ConfigError::FileParseError(cause.to_string())
            }
            configlib::ConfigError::Type { key, .. } => ConfigError::FieldTypeError(key.clone()),
            configlib::ConfigError::Message(message) => ConfigError::Other(Some(message.clone())),
            _ => ConfigError::Other(None),
        }
    }
}

#[macro_export]
macro_rules! get_config_blocking {
    () => {
        crate::config::CONFIG.blocking_read()
    };
}

#[macro_export]
macro_rules! get_config_async {
    () => {
        crate::config::CONFIG.read()
    };
}

pub async fn load() -> error_stack::Result<(), ConfigError> {
    *CONFIG.write().await = load_config()?;

    Ok(())
}

pub fn load_blocking() -> error_stack::Result<(), ConfigError> {
    *CONFIG.blocking_write() = load_config()?;

    Ok(())
}

fn load_config() -> error_stack::Result<Config, ConfigError> {
    let config = configlib::Config::builder()
        .add_source(configlib::File::with_name("config.toml"))
        .add_source(configlib::Environment::with_prefix("TCH"))
        .build()
        .into_report(); // todo not error out when config.toml is not present

    let config = config.map_err(|err| {
        let ctx = err.current_context().into();

        err.change_context(ctx)
    })?;

    let config = config.try_deserialize::<Config>().into_report();

    let config = config.map_err(|err| {
        let ctx = err.current_context().into();

        err.change_context(ctx)
    });

    config
}
