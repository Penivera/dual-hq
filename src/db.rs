use std::sync::Arc;
use sea_orm::{Database, DatabaseConnection, DbErr};
use sea_orm_migration::MigratorTrait;

use crate::config::Config;
use crate::migration::Migrator;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub config: Arc<Config>,
}

pub async fn init_db(config: &Config) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(&config.database_url).await?;
    tracing::info!("Connected to database: {}", config.database_url);

    // Run migrations on startup
    Migrator::up(&db, None).await?;
    tracing::info!("Applied database migrations successfully");

    // Pre-seed initial administrator account so there is always an admin
    if let Err(err) = crate::seed::seed_initial_admin(&db, config).await {
        tracing::warn!("Could not seed initial admin user: {err}");
    }

    Ok(db)
}
