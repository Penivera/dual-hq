use sea_orm::entity::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct NotificationResponse {
    pub id: i32,
    pub user_id: i32,
    pub title: String,
    pub body: String,
    pub read: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTimeWithTimeZone,
}
