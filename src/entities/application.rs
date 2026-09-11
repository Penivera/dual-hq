use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "applicationstatus")]
#[serde(rename_all = "lowercase")]
pub enum ApplicationStatus {
    #[sea_orm(string_value = "pending")]
    Pending,
    #[sea_orm(string_value = "accepted")]
    Accepted,
    #[sea_orm(string_value = "rejected")]
    Rejected,
    #[sea_orm(string_value = "withdrawn")]
    Withdrawn,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "applications")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    #[sea_orm(unique_key = "user_opportunity")]
    pub user_id: i32,
    #[sea_orm(unique_key = "user_opportunity")]
    pub opportunity_id: i32,
    pub cover_letter: Option<String>,
    pub status: ApplicationStatus,
    pub applied_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(
        belongs_to,
        from = "user_id",
        to = "id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    pub user: BelongsTo<super::user::Entity>,
    #[sea_orm(
        belongs_to,
        from = "opportunity_id",
        to = "id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    pub opportunity: BelongsTo<super::opportunity::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
