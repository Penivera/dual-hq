use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::entities::opportunity::{OpportunityStatus, OpportunityType};

#[derive(Debug, Deserialize, ToSchema)]
pub struct OpportunityCreate {
    #[schema(example = "Backend Intern")]
    pub title: String,
    #[schema(example = "Work on our Python/Rust microservices")]
    pub description: String,
    #[schema(example = "Acme Corp")]
    pub company: String,
    #[schema(example = "Remote")]
    pub location: String,
    #[serde(rename = "type")]
    pub type_: OpportunityType,
    pub category_id: Option<i32>,
    pub deadline: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OpportunityUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub company: Option<String>,
    pub location: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<OpportunityType>,
    pub status: Option<OpportunityStatus>,
    pub category_id: Option<i32>,
    pub deadline: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct OpportunityFullUpdate {
    #[schema(example = "Senior Rust Intern")]
    pub title: String,
    #[schema(example = "Build high performance microservices")]
    pub description: String,
    #[schema(example = "Acme Corp")]
    pub company: String,
    #[schema(example = "Remote")]
    pub location: String,
    #[serde(rename = "type")]
    pub type_: OpportunityType,
    pub status: OpportunityStatus,
    pub category_id: Option<i32>,
    pub deadline: Option<DateTime<FixedOffset>>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct OpportunityResponse {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub company: String,
    pub location: String,
    #[serde(rename = "type")]
    pub type_: OpportunityType,
    pub status: OpportunityStatus,
    pub created_by: Option<i32>,
    pub category_id: Option<i32>,
    pub category_name: Option<String>,
    #[schema(value_type = Option<String>, format = DateTime)]
    pub deadline: Option<DateTime<FixedOffset>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub application_count: Option<i64>,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<FixedOffset>,
    #[schema(value_type = String, format = DateTime)]
    pub updated_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, IntoParams)]
pub struct PaginationQuery {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub skip: Option<u64>,
    pub limit: Option<u64>,
    pub search: Option<String>,
    #[serde(rename = "type")]
    pub type_: Option<OpportunityType>,
    pub location: Option<String>,
    pub category: Option<String>,
}
