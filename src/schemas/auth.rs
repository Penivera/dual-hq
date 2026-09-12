use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::entities::user::{UserRole, UserStatus};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UserCreate {
    #[schema(example = "Jane Doe")]
    pub full_name: String,
    #[schema(example = "jane@example.com")]
    pub email: String,
    #[schema(example = "secret123")]
    pub password: String,
    #[schema(default = "applicant")]
    pub role: Option<UserRole>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: i32,
    pub full_name: String,
    pub email: String,
    pub role: UserRole,
    pub status: UserStatus,
    pub is_verified: bool,
    #[schema(value_type = String, format = DateTime)]
    pub created_at: DateTime<FixedOffset>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct VerifyEmailQuery {
    pub token: String,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ResendVerificationRequest {
    pub email: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct VerificationResponse {
    pub message: String,
    pub is_verified: bool,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    #[schema(example = "jane@example.com")]
    pub email: Option<String>,
    #[schema(example = "jane@example.com")]
    pub username: Option<String>,
    #[schema(example = "secret123")]
    pub password: String,
}

impl LoginRequest {
    pub fn get_email(&self) -> &str {
        self.email
            .as_deref()
            .or(self.username.as_deref())
            .unwrap_or("")
    }
}

#[derive(Debug, Serialize, ToSchema)]
pub struct Token {
    pub access_token: String,
    pub token_type: String,
    pub expires_in: i64,
    pub refresh_token: String,
    pub token: String,
}

impl Token {
    pub fn new(access_token: String, refresh_token: String, expires_in: i64) -> Self {
        Self {
            token: access_token.clone(),
            access_token,
            token_type: "bearer".to_string(),
            expires_in,
            refresh_token,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}
