use config::{Config, ConfigError, Environment, File};
use once_cell::sync::Lazy;
use serde::Deserialize;
use std::{env, fmt};

pub static SETTINGS: Lazy<Settings> =
    Lazy::new(|| Settings::new().expect("Failed to setup settings"));

#[derive(Debug, Clone, Deserialize)]
pub struct Server {
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Logger {
    pub level: String,
    #[serde(default = "default_log_format")]
    pub format: String,
    #[serde(default)]
    pub request_id_header: bool,
}

fn default_log_format() -> String {
    "text".to_string()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub url: String,
    pub pool_size: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Auth {
    pub secret: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Tor {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_tor_service_dir")]
    pub service_dir: String,
}

fn default_tor_service_dir() -> String {
    "./tor_service".to_string()
}

// Remove the #[allow(dead_code)] attribute from the Settings struct when all the fields are being
// used.
#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
pub struct Settings {
    pub environment: String,
    pub server: Server,
    pub logger: Logger,
    pub database: Database,
    pub auth: Auth,
    pub tor: Tor,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let mut builder = Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{run_mode}")).required(false))
            .add_source(File::with_name("config/local").required(false))
            .add_source(Environment::default().separator("__"));

        // Some cloud services like Heroku exposes a randomly assigned port in
        // the PORT env var and there is no way to change the env var name.
        if let Ok(port) = env::var("PORT") {
            builder = builder.set_override("server.port", port)?;
        }

        builder
            .build()?
            // Deserialize (and thus freeze) the entire configuration.
            .try_deserialize()
    }
}

impl fmt::Display for Server {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "http://localhost:{}", &self.port)
    }
}
