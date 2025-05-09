use std::env;
use std::str::FromStr;
use tracing::{Level, info};
use tracing_subscriber::{
    fmt::{self, format::{FmtSpan, Format}},
    EnvFilter, layer::SubscriberExt, util::SubscriberInitExt,
};
use once_cell::sync::Lazy;
use uuid::Uuid;

use crate::settings::SETTINGS;

/// Constants for log levels
pub mod log_level {
    pub const TRACE: &str = "trace";
    pub const DEBUG: &str = "debug";
    pub const INFO: &str = "info";
    pub const WARN: &str = "warn";
    pub const ERROR: &str = "error";
}

/// Generate a unique request ID
pub fn generate_request_id() -> String {
    Uuid::new_v4().to_string()
}

/// Setup the logger with the specified configuration
pub fn setup() {
    // Set up the RUST_LOG environment variable if not already set
    if env::var_os("RUST_LOG").is_none() {
        let app_name = env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "tor_marketplace".to_string());
        let level = SETTINGS.logger.level.as_str();
        let env = format!("{app_name}={level},tower_http={level},axum={level}");

        env::set_var("RUST_LOG", env);
    }

    // Parse the log format from settings
    let log_format = match SETTINGS.logger.format.as_str() {
        "json" => Format::Json,
        _ => Format::Default,
    };

    // Set up the tracing subscriber with the specified format
    tracing_subscriber::registry()
        .with(
            fmt::layer()
                .with_timer(fmt::time::UtcTime::rfc_3339())
                .with_span_events(FmtSpan::CLOSE)
                .with_writer(std::io::stdout)
                .event_format(log_format)
        )
        .with(EnvFilter::from_default_env())
        .init();

    // Log the startup message
    let level = Level::from_str(&SETTINGS.logger.level).unwrap_or(Level::INFO);
    info!(
        target: "tor_marketplace",
        level = %level,
        format = %SETTINGS.logger.format,
        "Logger initialized"
    );
}
