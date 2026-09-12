use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
    Email,
}

#[derive(DeriveIden)]
enum Opportunities {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Applications {
    Table,
    UserId,
    OpportunityId,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // 1. Enforce unique index on users.email
        manager
            .create_index(
                Index::create()
                    .name("idx_users_email_unique")
                    .table(Users::Table)
                    .col(Users::Email)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // 2. Enforce composite unique index on applications(user_id, opportunity_id)
        manager
            .create_index(
                Index::create()
                    .name("idx_applications_user_opportunity_unique")
                    .table(Applications::Table)
                    .col(Applications::UserId)
                    .col(Applications::OpportunityId)
                    .unique()
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        // 3. Ensure foreign keys exist with CASCADE delete (PostgreSQL / SQLite compatible)
        let db_backend = manager.get_database_backend();
        if db_backend == sea_orm::DbBackend::Postgres {
            // Add foreign key constraint for applications.user_id -> users.id
            manager
                .alter_table(
                    Table::alter()
                        .table(Applications::Table)
                        .add_foreign_key(
                            TableForeignKey::new()
                                .name("fk_applications_user_id")
                                .from_tbl(Applications::Table)
                                .from_col(Applications::UserId)
                                .to_tbl(Users::Table)
                                .to_col(Users::Id)
                                .on_delete(ForeignKeyAction::Cascade)
                                .on_update(ForeignKeyAction::NoAction),
                        )
                        .to_owned(),
                )
                .await
                .ok(); // ok if already exists from create_table_from_entity

            // Add foreign key constraint for applications.opportunity_id -> opportunities.id
            manager
                .alter_table(
                    Table::alter()
                        .table(Applications::Table)
                        .add_foreign_key(
                            TableForeignKey::new()
                                .name("fk_applications_opportunity_id")
                                .from_tbl(Applications::Table)
                                .from_col(Applications::OpportunityId)
                                .to_tbl(Opportunities::Table)
                                .to_col(Opportunities::Id)
                                .on_delete(ForeignKeyAction::Cascade)
                                .on_update(ForeignKeyAction::NoAction),
                        )
                        .to_owned(),
                )
                .await
                .ok(); // ok if already exists from create_table_from_entity
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_index(
                Index::drop()
                    .name("idx_applications_user_opportunity_unique")
                    .table(Applications::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_users_email_unique")
                    .table(Users::Table)
                    .if_exists()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}
