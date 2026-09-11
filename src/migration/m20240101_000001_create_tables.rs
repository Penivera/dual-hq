use sea_orm::Schema;
use sea_orm_migration::prelude::*;

use crate::entities::{application, opportunity, user};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let schema = Schema::new(manager.get_database_backend());

        // Create PostgreSQL enums first if using Postgres backend
        if manager.get_database_backend() == sea_orm::DbBackend::Postgres {
            for stmt in schema.create_enum_from_entity(user::Entity) {
                let _ = manager.create_type(stmt).await;
            }
            for stmt in schema.create_enum_from_entity(opportunity::Entity) {
                let _ = manager.create_type(stmt).await;
            }
            for stmt in schema.create_enum_from_entity(application::Entity) {
                let _ = manager.create_type(stmt).await;
            }
        }

        // Entity-first table creation
        manager
            .create_table(
                schema
                    .create_table_from_entity(user::Entity)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                schema
                    .create_table_from_entity(opportunity::Entity)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                schema
                    .create_table_from_entity(application::Entity)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // Entity-first index creation
        for mut index in schema.create_index_from_entity(user::Entity) {
            let _ = manager.create_index(index.if_not_exists().to_owned()).await;
        }
        for mut index in schema.create_index_from_entity(opportunity::Entity) {
            let _ = manager.create_index(index.if_not_exists().to_owned()).await;
        }
        for mut index in schema.create_index_from_entity(application::Entity) {
            let _ = manager.create_index(index.if_not_exists().to_owned()).await;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(application::Entity)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(opportunity::Entity)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        manager
            .drop_table(
                Table::drop()
                    .table(user::Entity)
                    .if_exists()
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
