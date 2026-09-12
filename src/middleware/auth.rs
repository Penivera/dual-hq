use axum::{
    extract::{FromRef, FromRequestParts},
    http::request::Parts,
};
use jsonwebtoken::{decode, DecodingKey, Validation};

use crate::{
    db::AppState,
    entities::user::UserRole,
    errors::AppError,
    schemas::auth::Claims,
};

#[derive(Debug, Clone)]
pub struct AuthenticatedUser {
    pub id: i32,
    pub role: UserRole,
}

#[derive(Debug, Clone)]
pub struct AdminUser(#[allow(dead_code)] pub AuthenticatedUser);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .ok_or_else(|| AppError::Unauthorized("Could not validate credentials".to_string()))?;

        if !auth_header.starts_with("Bearer ") {
            return Err(AppError::Unauthorized(
                "Could not validate credentials".to_string(),
            ));
        }

        let token = &auth_header[7..];
        let app_state = AppState::from_ref(state);

        let mut validation = Validation::default();
        validation.validate_exp = true;

        let token_data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(app_state.config.jwt_secret.as_bytes()),
            &validation,
        )
        .map_err(|_| AppError::Unauthorized("Could not validate credentials".to_string()))?;

        let user_id = token_data
            .claims
            .sub
            .parse::<i32>()
            .map_err(|_| AppError::Unauthorized("Could not validate credentials".to_string()))?;

        let role = match token_data.claims.role.as_str() {
            "admin" => UserRole::Admin,
            "recruiter" | "manager" => UserRole::Recruiter,
            "applicant" => UserRole::Applicant,
            _ => return Err(AppError::Unauthorized("Could not validate credentials".to_string())),
        };

        Ok(AuthenticatedUser { id: user_id, role })
    }
}

#[derive(Debug, Clone)]
pub struct ApplicantUser(pub AuthenticatedUser);

impl<S> FromRequestParts<S> for ApplicantUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;
        if user.role != UserRole::Applicant {
            return Err(AppError::Forbidden("Applicant access required".to_string()));
        }
        Ok(ApplicantUser(user))
    }
}

#[derive(Debug, Clone)]
pub struct RecruiterUser(pub AuthenticatedUser);

impl<S> FromRequestParts<S> for RecruiterUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;
        if user.role != UserRole::Recruiter {
            return Err(AppError::Forbidden("Recruiter access required".to_string()));
        }
        Ok(RecruiterUser(user))
    }
}

#[derive(Debug, Clone)]
pub struct RecruiterOrAdminUser(pub AuthenticatedUser);

impl<S> FromRequestParts<S> for RecruiterOrAdminUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;
        if user.role != UserRole::Recruiter && user.role != UserRole::Admin {
            return Err(AppError::Forbidden("Recruiter or admin access required".to_string()));
        }
        Ok(RecruiterOrAdminUser(user))
    }
}

impl<S> FromRequestParts<S> for AdminUser
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let user = AuthenticatedUser::from_request_parts(parts, state).await?;
        if user.role != UserRole::Admin {
            return Err(AppError::Forbidden("Admin access required".to_string()));
        }
        Ok(AdminUser(user))
    }
}
