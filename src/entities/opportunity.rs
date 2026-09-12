use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "opportunitytype")]
#[serde(rename_all = "lowercase")]
pub enum OpportunityType {
    #[sea_orm(string_value = "internship")]
    Internship,
    #[sea_orm(string_value = "job")]
    Job,
}

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize, ToSchema,
)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "opportunitystatus")]
#[serde(rename_all = "lowercase")]
pub enum OpportunityStatus {
    #[sea_orm(string_value = "open")]
    Open,
    #[sea_orm(string_value = "closed")]
    Closed,
}

#[sea_orm::model]
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "opportunities")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub description: String,
    pub company: String,
    pub location: String,
    pub type_: OpportunityType,
    pub status: OpportunityStatus,
    pub created_by: Option<i32>,
    pub category_id: Option<i32>,
    pub deadline: Option<DateTimeWithTimeZone>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(has_many)]
    pub applications: HasMany<super::application::Entity>,
}

impl ActiveModelBehavior for ActiveModel {}
