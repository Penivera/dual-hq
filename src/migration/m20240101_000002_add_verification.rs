use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[derive(DeriveIden)]
enum Users {
    Table,
    IsVerified,
    VerificationToken,
}

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let alter_table = Table::alter()
            .table(Users::Table)
            .add_column_if_not_exists(
                ColumnDef::new(Users::IsVerified)
                    .boolean()
                    .not_null()
                    .default(false),
            )
            .add_column_if_not_exists(
                ColumnDef::new(Users::VerificationToken)
                    .string()
                    .null(),
            )
            .to_owned();

        manager.alter_table(alter_table).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Users::Table)
                    .drop_column(Users::VerificationToken)
                    .drop_column(Users::IsVerified)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
