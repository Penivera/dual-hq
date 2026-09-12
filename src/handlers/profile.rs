use axum::{
    extract::{Path, State},
    Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use url::Url;

use crate::{
    db::AppState,
    entities::{profile, Profile},
    errors::{AppError, ErrorDetail},
    middleware::{ApplicantUser, RecruiterOrAdminUser},
    schemas::profile::{ProfileResponse, ProfileUpdate},
};

fn validate_cv_url(cv_url: &str) -> Result<(), AppError> {
    let parsed = Url::parse(cv_url)
        .map_err(|_| AppError::Validation("cv_url must be a valid URL".to_string()))?;

    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err(AppError::Validation(
            "cv_url must have http or https scheme".to_string(),
        ));
    }

    Ok(())
}

#[utoipa::path(
    get,
    path = "/profile/me",
    tag = "Profile",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Current user profile", body = ProfileResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail)
    )
)]
pub async fn get_my_profile(
    State(state): State<AppState>,
    applicant: ApplicantUser,
) -> Result<Json<ProfileResponse>, AppError> {
    let existing = Profile::find()
        .filter(profile::Column::UserId.eq(applicant.0.id))
        .one(&state.db)
        .await?;

    let p = match existing {
        Some(prof) => prof,
        None => {
            let new_prof = profile::ActiveModel {
                user_id: Set(applicant.0.id),
                bio: Set(None),
                cv_url: Set(None),
                updated_at: Set(chrono::Utc::now().into()),
                ..Default::default()
            };
            new_prof.insert(&state.db).await?
        }
    };

    Ok(Json(ProfileResponse {
        id: p.id,
        user_id: p.user_id,
        bio: p.bio,
        cv_url: p.cv_url,
        updated_at: p.updated_at,
    }))
}

#[utoipa::path(
    put,
    path = "/profile/me",
    tag = "Profile",
    security(("bearerAuth" = [])),
    request_body = ProfileUpdate,
    responses(
        (status = 200, description = "Profile updated successfully", body = ProfileResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 422, description = "Validation error", body = ErrorDetail)
    )
)]
pub async fn update_my_profile(
    State(state): State<AppState>,
    applicant: ApplicantUser,
    Json(payload): Json<ProfileUpdate>,
) -> Result<Json<ProfileResponse>, AppError> {
    if let Some(ref url) = payload.cv_url {
        if !url.trim().is_empty() {
            validate_cv_url(url.trim())?;
        }
    }

    let existing = Profile::find()
        .filter(profile::Column::UserId.eq(applicant.0.id))
        .one(&state.db)
        .await?;

    let updated = match existing {
        Some(prof) => {
            let mut active: profile::ActiveModel = prof.into();
            if payload.bio.is_some() {
                active.bio = Set(payload.bio);
            }
            if payload.cv_url.is_some() {
                active.cv_url = Set(payload.cv_url);
            }
            active.updated_at = Set(chrono::Utc::now().into());
            active.update(&state.db).await?
        }
        None => {
            let new_prof = profile::ActiveModel {
                user_id: Set(applicant.0.id),
                bio: Set(payload.bio),
                cv_url: Set(payload.cv_url),
                updated_at: Set(chrono::Utc::now().into()),
                ..Default::default()
            };
            new_prof.insert(&state.db).await?
        }
    };

    Ok(Json(ProfileResponse {
        id: updated.id,
        user_id: updated.user_id,
        bio: updated.bio,
        cv_url: updated.cv_url,
        updated_at: updated.updated_at,
    }))
}

#[utoipa::path(
    get,
    path = "/profile/{user_id}",
    tag = "Profile",
    security(("bearerAuth" = [])),
    params(
        ("user_id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Applicant profile", body = ProfileResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Forbidden (recruiter or admin only)", body = ErrorDetail),
        (status = 404, description = "Profile not found", body = ErrorDetail)
    )
)]
pub async fn get_profile_by_user_id(
    State(state): State<AppState>,
    _auth: RecruiterOrAdminUser,
    Path(user_id): Path<i32>,
) -> Result<Json<ProfileResponse>, AppError> {
    let p = Profile::find()
        .filter(profile::Column::UserId.eq(user_id))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Profile not found for this user".to_string()))?;

    Ok(Json(ProfileResponse {
        id: p.id,
        user_id: p.user_id,
        bio: p.bio,
        cv_url: p.cv_url,
        updated_at: p.updated_at,
    }))
}
