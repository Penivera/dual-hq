use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Users {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum Categories {
    Table,
    Id,
    Name,
    Slug,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Opportunities {
    Table,
}

#[derive(DeriveIden)]
enum Profiles {
    Table,
    Id,
    UserId,
    Bio,
    CvUrl,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum RefreshTokens {
    Table,
    Id,
    UserId,
    Token,
    ExpiresAt,
    Revoked,
    CreatedAt,
}

#[derive(DeriveIden)]
enum Notifications {
    Table,
    Id,
    UserId,
    Title,
    Body,
    Read,
    CreatedAt,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let conn = manager.get_connection();
        let is_postgres = manager.get_database_backend() == sea_orm::DbBackend::Postgres;

        // 1. Update users table and Postgres enums
        if is_postgres {
            let _ = conn.execute_unprepared("ALTER TYPE userrole ADD VALUE IF NOT EXISTS 'recruiter'").await;
            let _ = conn.execute_unprepared("DO $$ BEGIN CREATE TYPE userstatus AS ENUM ('pending', 'active', 'suspended'); EXCEPTION WHEN duplicate_object THEN null; END $$;").await;
            let _ = conn.execute_unprepared("ALTER TABLE users ADD COLUMN IF NOT EXISTS status userstatus NOT NULL DEFAULT 'active'").await;
        } else {
            let _ = manager.alter_table(
                Table::alter()
                    .table(Users::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Alias::new("status"))
                            .string()
                            .not_null()
                            .default("active"),
                    )
                    .to_owned(),
            ).await;
        }

        // 2. Categories table
        manager
            .create_table(
                Table::create()
                    .table(Categories::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Categories::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Categories::Name).string().not_null().unique_key())
                    .col(ColumnDef::new(Categories::Slug).string().not_null().unique_key())
                    .col(
                        ColumnDef::new(Categories::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 3. Opportunities table updates
        manager
            .alter_table(
                Table::alter()
                    .table(Opportunities::Table)
                    .add_column_if_not_exists(ColumnDef::new(Alias::new("created_by")).integer().null())
                    .add_column_if_not_exists(ColumnDef::new(Alias::new("category_id")).integer().null())
                    .add_column_if_not_exists(ColumnDef::new(Alias::new("deadline")).timestamp_with_time_zone().null())
                    .to_owned(),
            )
            .await?;

        if is_postgres {
            let _ = manager
                .alter_table(
                    Table::alter()
                        .table(Opportunities::Table)
                        .add_foreign_key(
                            TableForeignKey::new()
                                .name("fk_opportunities_created_by")
                                .from_tbl(Opportunities::Table)
                                .from_col(Alias::new("created_by"))
                                .to_tbl(Users::Table)
                                .to_col(Users::Id)
                                .on_delete(ForeignKeyAction::SetNull)
                                .on_update(ForeignKeyAction::NoAction),
                        )
                        .to_owned(),
                )
                .await;

            let _ = manager
                .alter_table(
                    Table::alter()
                        .table(Opportunities::Table)
                        .add_foreign_key(
                            TableForeignKey::new()
                                .name("fk_opportunities_category_id")
                                .from_tbl(Opportunities::Table)
                                .from_col(Alias::new("category_id"))
                                .to_tbl(Categories::Table)
                                .to_col(Categories::Id)
                                .on_delete(ForeignKeyAction::SetNull)
                                .on_update(ForeignKeyAction::NoAction),
                        )
                        .to_owned(),
                )
                .await;
        }

        // 4. Profiles table
        manager
            .create_table(
                Table::create()
                    .table(Profiles::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Profiles::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Profiles::UserId).integer().not_null().unique_key())
                    .col(ColumnDef::new(Profiles::Bio).text().null())
                    .col(ColumnDef::new(Profiles::CvUrl).string().null())
                    .col(
                        ColumnDef::new(Profiles::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 5. RefreshTokens table
        manager
            .create_table(
                Table::create()
                    .table(RefreshTokens::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(RefreshTokens::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(RefreshTokens::UserId).integer().not_null())
                    .col(ColumnDef::new(RefreshTokens::Token).string().not_null().unique_key())
                    .col(ColumnDef::new(RefreshTokens::ExpiresAt).timestamp_with_time_zone().not_null())
                    .col(ColumnDef::new(RefreshTokens::Revoked).boolean().not_null().default(false))
                    .col(
                        ColumnDef::new(RefreshTokens::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        // 6. Notifications table
        manager
            .create_table(
                Table::create()
                    .table(Notifications::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Notifications::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Notifications::UserId).integer().not_null())
                    .col(ColumnDef::new(Notifications::Title).text().not_null())
                    .col(ColumnDef::new(Notifications::Body).text().not_null())
                    .col(ColumnDef::new(Notifications::Read).boolean().not_null().default(false))
                    .col(
                        ColumnDef::new(Notifications::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        let db_backend = manager.get_database_backend();
        if db_backend == sea_orm::DbBackend::Postgres {
            let _ = manager
                .alter_table(
                    Table::alter()
                        .table(Profiles::Table)
                        .add_foreign_key(
                            TableForeignKey::new()
                                .name("fk_profiles_user_id")
                                .from_tbl(Profiles::Table)
                                .from_col(Profiles::UserId)
                                .to_tbl(Users::Table)
                                .to_col(Users::Id)
                                .on_delete(ForeignKeyAction::Cascade)
                                .on_update(ForeignKeyAction::NoAction),
                        )
                        .to_owned(),
                )
                .await;

            let _ = manager
                .alter_table(
                    Table::alter()
                        .table(RefreshTokens::Table)
                        .add_foreign_key(
                            TableForeignKey::new()
                                .name("fk_refresh_tokens_user_id")
                                .from_tbl(RefreshTokens::Table)
                                .from_col(RefreshTokens::UserId)
                                .to_tbl(Users::Table)
                                .to_col(Users::Id)
                                .on_delete(ForeignKeyAction::Cascade)
                                .on_update(ForeignKeyAction::NoAction),
                        )
                        .to_owned(),
                )
                .await;

            let _ = manager
                .alter_table(
                    Table::alter()
                        .table(Notifications::Table)
                        .add_foreign_key(
                            TableForeignKey::new()
                                .name("fk_notifications_user_id")
                                .from_tbl(Notifications::Table)
                                .from_col(Notifications::UserId)
                                .to_tbl(Users::Table)
                                .to_col(Users::Id)
                                .on_delete(ForeignKeyAction::Cascade)
                                .on_update(ForeignKeyAction::NoAction),
                        )
                        .to_owned(),
                )
                .await;
        }

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(Notifications::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(RefreshTokens::Table).if_exists().to_owned()).await?;
        manager.drop_table(Table::drop().table(Profiles::Table).if_exists().to_owned()).await?;
        
        manager.alter_table(
            Table::alter()
                .table(Opportunities::Table)
                .drop_column(Alias::new("deadline"))
                .drop_column(Alias::new("category_id"))
                .drop_column(Alias::new("created_by"))
                .to_owned(),
        ).await?;

        manager.drop_table(Table::drop().table(Categories::Table).if_exists().to_owned()).await?;

        manager.alter_table(
            Table::alter()
                .table(Users::Table)
                .drop_column(Alias::new("status"))
                .to_owned(),
        ).await?;

        Ok(())
    }
}
