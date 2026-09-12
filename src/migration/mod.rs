pub use sea_orm_migration::prelude::*;

mod m20240101_000001_create_tables;
mod m20240101_000002_add_verification;
mod m20240101_000003_enforce_constraints;
mod m20240101_000004_v2_features;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20240101_000001_create_tables::Migration),
            Box::new(m20240101_000002_add_verification::Migration),
            Box::new(m20240101_000003_enforce_constraints::Migration),
            Box::new(m20240101_000004_v2_features::Migration),
        ]
    }
}
