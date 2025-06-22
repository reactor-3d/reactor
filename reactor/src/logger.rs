use std::error::Error;
use std::path::{Path, PathBuf};
use std::string::ParseError;
use std::{fs, io};

use config_load::config::builder::DefaultState;
use config_load::config::{ConfigBuilder, Environment};
use config_load::{ConfigLoader, FileLocation, Load};
use serde::{Deserialize, Serialize};
use tracing::subscriber::{SetGlobalDefaultError, set_global_default};
use tracing_subscriber::fmt::Layer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

#[derive(Debug, thiserror::Error)]
pub enum LoggerError {
    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    SetGlobal(#[from] SetGlobalDefaultError),
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct LoggerConfig {
    #[serde(default = "LoggerConfig::default_filter")]
    pub filter: String,

    pub path: Option<PathBuf>,

    #[serde(default = "LoggerConfig::default_print_to_stdout")]
    pub print_to_stdout: bool,

    #[serde(default = "LoggerConfig::default_compact")]
    pub compact: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            filter: Self::default_filter(),
            path: None,
            print_to_stdout: Self::default_print_to_stdout(),
            compact: Self::default_compact(),
        }
    }
}

impl LoggerConfig {
    pub fn default_filter() -> String {
        "info".into()
    }

    pub const fn default_print_to_stdout() -> bool {
        true
    }

    pub const fn default_compact() -> bool {
        true
    }

    pub fn load(config_file: Option<PathBuf>) -> config_load::Result<Self> {
        ConfigLoader::default()
            .add(
                FileLocation::first_some_path()
                    .from_env("REACTOR_LOGGER_CONFIG")
                    .from_home(Path::new(".reactor").join("Logger.toml")),
            )
            .exclude_not_exists()
            .add(
                FileLocation::first_some_path()
                    .from_file(config_file)
                    .from_cwd_and_parents_exists("Logger.toml"),
            )
            .load()
    }
}

impl Load for LoggerConfig {
    fn load(config_builder: ConfigBuilder<DefaultState>) -> config_load::Result<Self> {
        // Add in settings from the environment (with a prefix of DOVER)
        // Eg.. `REACTOR_LOGGER_FILTER=info reactor` would set the `filter` key
        let config = config_builder
            .add_source(
                Environment::with_prefix("REACTOR_LOGGER")
                    .separator("_")
                    .try_parsing(true),
            )
            .build()?;
        config.try_deserialize()
    }
}

pub fn init(settings: &LoggerConfig) -> Result<(), LoggerError> {
    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|from_env_err| {
        if let Some(parse_err) = from_env_err
            .source()
            .and_then(|source| source.downcast_ref::<ParseError>())
        {
            // we cannot use the `error!` macro here because the logger is not ready yet.
            eprintln!("Logger failed to parse filter from env: {parse_err}");
        }
        EnvFilter::builder().parse_lossy(&settings.filter)
    });

    let file_layer = if let Some(path) = &settings.path {
        let file = fs::File::create(path)?;
        Some(Layer::default().with_writer(file))
    } else {
        None
    };

    let subscriber = Registry::default().with(filter_layer);
    if settings.compact {
        set_global_default(
            subscriber.with(file_layer.map(|layer| layer.compact())).with(
                settings
                    .print_to_stdout
                    .then(|| Layer::default().with_writer(io::stdout).compact()),
            ),
        )?;
    } else {
        set_global_default(
            subscriber.with(file_layer).with(
                settings
                    .print_to_stdout
                    .then(|| Layer::default().with_writer(io::stdout)),
            ),
        )?;
    }

    Ok(())
}
