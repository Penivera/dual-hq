use sea_orm::entity::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct CategoryCreate {
    #[schema(example = "Software Engineering")]
    pub name: String,
    #[schema(example = "software-engineering")]
    pub slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CategoryResponse {
    pub id: i32,
    pub name: String,
    pub slug: String,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTimeWithTimeZone,
}
