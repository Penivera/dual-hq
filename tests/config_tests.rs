use std::sync::Mutex;
use config::{File, FileFormat};
use internship_api::config::Config;

static ENV_MUTEX: Mutex<()> = Mutex::new(());

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner())
}

#[test]
fn test_config_struct_defaults() {
    let _lock = lock_env();

    // Isolated builder without .env or system environment sources
    let builder = Config::builder();
    let config: Config = builder.build().unwrap().try_deserialize().unwrap();

    assert_eq!(config.database_url, "postgresql://postgres:password@localhost:5432/internship_db");
    assert_eq!(config.jwt_secret, "super-secret-jwt-key-replace-in-production");
    assert_eq!(config.jwt_expiry_hours, 24);
    assert_eq!(config.server_host, "0.0.0.0");
    assert_eq!(config.server_port, 8000);
}

#[test]
fn test_config_load_from_env_or_file() {
    let _lock = lock_env();

    let config = Config::load().expect("Config::load should succeed");
    assert!(!config.database_url.is_empty());
    assert!(!config.jwt_secret.is_empty());
    assert!(config.jwt_expiry_hours > 0);
    assert!(!config.server_host.is_empty());
    assert!(config.server_port > 0);
}

#[test]
fn test_config_load_standard_env_vars() {
    let _lock = lock_env();

    let prev_db = std::env::var("DATABASE_URL").ok();
    let prev_jwt = std::env::var("JWT_SECRET").ok();
    let prev_expiry = std::env::var("JWT_EXPIRY_HOURS").ok();
    let prev_host = std::env::var("SERVER_HOST").ok();
    let prev_port = std::env::var("SERVER_PORT").ok();

    std::env::set_var("DATABASE_URL", "postgresql://app-user:secret@localhost:5432/app_db");
    std::env::set_var("JWT_SECRET", "custom-jwt-secret-value");
    std::env::set_var("JWT_EXPIRY_HOURS", "48");
    std::env::set_var("SERVER_HOST", "127.0.0.1");
    std::env::set_var("SERVER_PORT", "9000");

    let config = Config::load().expect("Config::load should read standard env vars");
    assert_eq!(config.database_url, "postgresql://app-user:secret@localhost:5432/app_db");
    assert_eq!(config.jwt_secret, "custom-jwt-secret-value");
    assert_eq!(config.jwt_expiry_hours, 48);
    assert_eq!(config.server_host, "127.0.0.1");
    assert_eq!(config.server_port, 9000);

    // Restore
    match prev_db { Some(v) => std::env::set_var("DATABASE_URL", v), None => std::env::remove_var("DATABASE_URL") }
    match prev_jwt { Some(v) => std::env::set_var("JWT_SECRET", v), None => std::env::remove_var("JWT_SECRET") }
    match prev_expiry { Some(v) => std::env::set_var("JWT_EXPIRY_HOURS", v), None => std::env::remove_var("JWT_EXPIRY_HOURS") }
    match prev_host { Some(v) => std::env::set_var("SERVER_HOST", v), None => std::env::remove_var("SERVER_HOST") }
    match prev_port { Some(v) => std::env::set_var("SERVER_PORT", v), None => std::env::remove_var("SERVER_PORT") }
}

#[test]
fn test_config_builder_custom_secret() {
    let _lock = lock_env();

    let toml_content = r#"
        jwt_secret = "custom-secret-key-123"
    "#;

    let builder = Config::builder()
        .add_source(File::from_str(toml_content, FileFormat::Toml));

    let config: Config = builder.build().unwrap().try_deserialize().unwrap();
    assert_eq!(config.jwt_secret, "custom-secret-key-123");
}

#[test]
fn test_config_load_app_prefix_override() {
    let _lock = lock_env();

    let prev_app_db = std::env::var("APP_DATABASE_URL").ok();
    let prev_app_port = std::env::var("APP_SERVER_PORT").ok();

    std::env::set_var("APP_DATABASE_URL", "postgresql://override:5432/db");
    std::env::set_var("APP_SERVER_PORT", "8888");

    let config = Config::load().expect("Config::load should allow APP_ prefix to override base env");
    assert_eq!(config.database_url, "postgresql://override:5432/db");
    assert_eq!(config.server_port, 8888);

    match prev_app_db { Some(v) => std::env::set_var("APP_DATABASE_URL", v), None => std::env::remove_var("APP_DATABASE_URL") }
    match prev_app_port { Some(v) => std::env::set_var("APP_SERVER_PORT", v), None => std::env::remove_var("APP_SERVER_PORT") }
}

#[test]
fn test_config_load_internship_prefix_override() {
    let _lock = lock_env();

    let prev_intern_db = std::env::var("INTERNSHIP_DATABASE_URL").ok();
    let prev_intern_port = std::env::var("INTERNSHIP_SERVER_PORT").ok();

    std::env::set_var("INTERNSHIP_DATABASE_URL", "postgresql://internship-db:5432/db");
    std::env::set_var("INTERNSHIP_SERVER_PORT", "8181");

    let config = Config::load().expect("Config::load should support INTERNSHIP_ prefix");
    assert_eq!(config.database_url, "postgresql://internship-db:5432/db");
    assert_eq!(config.server_port, 8181);

    match prev_intern_db { Some(v) => std::env::set_var("INTERNSHIP_DATABASE_URL", v), None => std::env::remove_var("INTERNSHIP_DATABASE_URL") }
    match prev_intern_port { Some(v) => std::env::set_var("INTERNSHIP_SERVER_PORT", v), None => std::env::remove_var("INTERNSHIP_SERVER_PORT") }
}

#[test]
fn test_config_builder_with_toml_source() {
    let _lock = lock_env();

    let toml_content = r#"
        database_url = "postgresql://toml-user:password@localhost:5432/toml_db"
        jwt_secret = "toml-secret-key"
        server_port = 8080
    "#;

    let builder = Config::builder()
        .add_source(File::from_str(toml_content, FileFormat::Toml));

    let config: Config = builder.build().unwrap().try_deserialize().unwrap();
    assert_eq!(config.database_url, "postgresql://toml-user:password@localhost:5432/toml_db");
    assert_eq!(config.jwt_secret, "toml-secret-key");
    assert_eq!(config.server_port, 8080);
    // Defaults apply for omitted values
    assert_eq!(config.server_host, "0.0.0.0");
    assert_eq!(config.jwt_expiry_hours, 24);
}
