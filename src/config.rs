use config::{Config as ConfigBuilder, Environment, File, FileFormat};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_database_url")]
    pub database_url: String,
    #[serde(default = "default_jwt_secret")]
    pub jwt_secret: String,
    #[serde(default = "default_jwt_expiry_hours")]
    pub jwt_expiry_hours: i64,
    #[serde(default = "default_server_host")]
    pub server_host: String,
    #[serde(default = "default_server_port")]
    pub server_port: u16,
    #[serde(default = "default_admin_email")]
    pub admin_email: String,
    #[serde(default = "default_admin_password")]
    pub admin_password: String,
    #[serde(default = "default_admin_name")]
    pub admin_name: String,
    #[serde(default = "default_smtp_host")]
    pub smtp_host: String,
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
    #[serde(default = "default_smtp_user")]
    pub smtp_user: String,
    #[serde(default = "default_smtp_password")]
    pub smtp_password: String,
    #[serde(default = "default_smtp_from_email")]
    pub smtp_from_email: String,
    #[serde(default = "default_smtp_from_name")]
    pub smtp_from_name: String,
    #[serde(default = "default_smtp_enabled")]
    pub smtp_enabled: bool,
    #[serde(default = "default_app_base_url")]
    pub app_base_url: String,
    #[serde(default = "default_db_max_connections")]
    pub db_max_connections: u32,
    #[serde(default = "default_db_min_connections")]
    pub db_min_connections: u32,
    #[serde(default = "default_db_connect_timeout_secs")]
    pub db_connect_timeout_secs: u64,
    #[serde(default = "default_db_idle_timeout_secs")]
    pub db_idle_timeout_secs: u64,
}

fn default_db_max_connections() -> u32 {
    10
}

fn default_db_min_connections() -> u32 {
    2
}

fn default_db_connect_timeout_secs() -> u64 {
    5
}

fn default_db_idle_timeout_secs() -> u64 {
    600
}

fn default_database_url() -> String {
    "postgresql://postgres:password@localhost:5432/internship_db".to_string()
}

fn default_jwt_secret() -> String {
    "super-secret-jwt-key-replace-in-production".to_string()
}

fn default_jwt_expiry_hours() -> i64 {
    24
}

fn default_server_host() -> String {
    "0.0.0.0".to_string()
}

fn default_server_port() -> u16 {
    8000
}

fn default_admin_email() -> String {
    "admin@internship.local".to_string()
}

fn default_admin_password() -> String {
    "admin".to_string()
}

fn default_admin_name() -> String {
    "Admin User".to_string()
}

fn default_smtp_host() -> String {
    "smtp-relay.brevo.com".to_string()
}

fn default_smtp_port() -> u16 {
    587
}

fn default_smtp_user() -> String {
    String::new()
}

fn default_smtp_password() -> String {
    String::new()
}

fn default_smtp_from_email() -> String {
    String::new()
}

fn default_smtp_from_name() -> String {
    "Peni Demo".to_string()
}

fn default_smtp_enabled() -> bool {
    false
}

fn default_app_base_url() -> String {
    "http://localhost:8010".to_string()
}

impl Config {
    pub fn builder() -> config::builder::ConfigBuilder<config::builder::DefaultState> {
        ConfigBuilder::builder()
    }

    pub fn load() -> Result<Self, anyhow::Error> {
        dotenvy::dotenv().ok();

        // Support legacy Python FastAPI SECRET_KEY if JWT_SECRET is not explicitly provided
        if std::env::var("JWT_SECRET").is_err() {
            if let Ok(secret) = std::env::var("SECRET_KEY") {
                std::env::set_var("JWT_SECRET", secret);
            }
        }

        let builder = Config::builder()
            // Optional configuration and secret files
            .add_source(File::new("config.toml", FileFormat::Toml).required(false))
            .add_source(File::new("secrets.toml", FileFormat::Toml).required(false))
            .add_source(File::new("Secrets.toml", FileFormat::Toml).required(false))
            // Standard environment variables (DATABASE_URL, JWT_SECRET, SERVER_PORT, etc.)
            .add_source(
                Environment::default()
                    .separator("__")
                    .try_parsing(true)
                    .ignore_empty(true),
            )
            // Application-prefixed environment variables (e.g. APP_DATABASE_URL, APP_SERVER_PORT)
            .add_source(
                Environment::with_prefix("APP")
                    .prefix_separator("_")
                    .separator("__")
                    .try_parsing(true)
                    .ignore_empty(true),
            )
            // Project-prefixed environment variables (e.g. INTERNSHIP_DATABASE_URL)
            .add_source(
                Environment::with_prefix("INTERNSHIP")
                    .prefix_separator("_")
                    .separator("__")
                    .try_parsing(true)
                    .ignore_empty(true),
            );

        let config: Config = builder.build()?.try_deserialize()?;
        Ok(config)
    }
}
