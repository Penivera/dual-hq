use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "userrole")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
    #[sea_orm(string_value = "applicant")]
    Applicant,
    #[sea_orm(string_value = "admin")]
    Admin,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub full_name: String,
    #[sea_orm(unique, index)]
    pub email: String,
    pub hashed_password: String,
    pub role: UserRole,
    #[sea_orm(default_value = false)]
    pub is_verified: bool,
    pub verification_token: Option<String>,
    pub created_at: DateTimeWithTimeZone,
    #[sea_orm(has_many)]
    pub applications: HasMany<super::application::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
