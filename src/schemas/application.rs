use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entities::application::ApplicationStatus;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ApplicationCreate {
    #[schema(example = 1)]
    pub opportunity_id: i32,
    #[schema(example = "I am very interested in this role!")]
    pub cover_letter: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ApplicationStatusUpdate {
    pub status: ApplicationStatus,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApplicationResponse {
    pub id: i32,
    pub user_id: i32,
    pub opportunity_id: i32,
    pub cover_letter: Option<String>,
    pub status: ApplicationStatus,
    #[schema(value_type = String, format = DateTime)]
    pub applied_at: DateTime<FixedOffset>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApplicationDetailResponse {
    pub id: i32,
    pub user_id: i32,
    pub opportunity_id: i32,
    pub opportunity_title: String,
    pub company: String,
    pub cover_letter: Option<String>,
    pub status: ApplicationStatus,
    #[schema(value_type = String, format = DateTime)]
    pub applied_at: DateTime<FixedOffset>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<FixedOffset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applicant_cv_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applicant_profile_url: Option<String>,
}
