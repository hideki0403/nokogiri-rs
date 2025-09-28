use crate::resource;
use config::{Config, File, FileFormat};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{fs, path::Path, process};

#[derive(Deserialize, Debug, Clone)]
pub struct IServer {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ISecurity {
    pub secret_key: String,
    pub block_non_global_ips: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct IPlugins {
    pub disabled: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ISentry {
    pub dsn: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct IDebug {
    pub log_level: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ICache {
    pub enabled: bool,
    pub host: String,
    pub port: u16,
    pub prefix: Option<String>,
    pub db: Option<u32>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AppConfig {
    pub server: IServer,
    pub security: ISecurity,
    pub plugins: IPlugins,
    pub cache: ICache,
    pub sentry: Option<ISentry>,
    pub debug: Option<IDebug>,
}

impl AppConfig {
    pub fn new() -> Result<Self, config::ConfigError> {
        if !Path::new("./config.toml").exists() {
            let default_config = resource::DEFAULT_CONFIG_TOML;
            fs::write("./config.toml", default_config).expect("Failed to create default config file");
            println!("Created configration file at ./config.toml. Please check it before running the application.");
            process::exit(0);
        }

        let config = Config::builder()
            .add_source(File::from_str(str::from_utf8(resource::DEFAULT_CONFIG_TOML).unwrap(), FileFormat::Toml))
            .add_source(File::with_name("./config.toml"))
            .build()
            .expect("Failed to load configuration");

        config.try_deserialize::<AppConfig>()
    }
}

pub static CONFIG: Lazy<AppConfig> = Lazy::new(|| AppConfig::new().expect("Failed to initialize application configuration"));
