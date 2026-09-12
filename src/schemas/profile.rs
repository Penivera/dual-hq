use sea_orm::entity::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, ToSchema)]
pub struct ProfileUpdate {
    #[schema(example = "Software engineer passionate about systems programming and cloud services.")]
    pub bio: Option<String>,
    #[schema(example = "https://example.com/resumes/johndoe.pdf")]
    pub cv_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ProfileResponse {
    pub id: i32,
    pub user_id: i32,
    pub bio: Option<String>,
    pub cv_url: Option<String>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTimeWithTimeZone,
}
