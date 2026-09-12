use std::sync::Arc;
use std::time::Duration;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

use crate::config::Config;
use crate::migration::Migrator;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Arc<Config>,
}

pub async fn init_db(config: &Config) -> Result<DatabaseConnection, DbErr> {
    let mut opt = ConnectOptions::new(&config.database_url);
    opt.max_connections(25)
        .min_connections(2)
        .connect_timeout(Duration::from_secs(15))
        .acquire_timeout(Duration::from_secs(15))
        .idle_timeout(Duration::from_secs(300))
        .max_lifetime(Duration::from_secs(1800));

    let db = Database::connect(opt).await?;
    tracing::info!("Connected to database successfully");

    // Run migrations on startup
    Migrator::up(&db, None).await?;
    tracing::info!("Applied database migrations successfully");

    // Pre-seed initial administrator account so there is always an admin
    if let Err(err) = crate::seed::seed_initial_admin(&db, config).await {
        tracing::warn!("Could not seed initial admin user: {err}");
    }

    Ok(db)
}
