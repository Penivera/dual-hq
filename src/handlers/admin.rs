use axum::{
    extract::{Path, State},
    Json,
};
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    db::AppState,
    entities::{
        refresh_token,
        user::{self, UserRole, UserStatus},
        RefreshToken, User,
    },
    errors::{AppError, ErrorDetail},
    middleware::AdminUser,
    notifications::send_notification,
    schemas::auth::UserResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct SuspendUserRequest {
    #[schema(example = "Violation of terms of service")]
    pub reason: Option<String>,
}

#[utoipa::path(
    get,
    path = "/admin/recruiters/pending",
    tag = "Admin",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "List pending recruiters", body = Vec<UserResponse>),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Forbidden (admin only)", body = ErrorDetail)
    )
)]
pub async fn list_pending_recruiters(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> Result<Json<Vec<UserResponse>>, AppError> {
    let recruiters = User::find()
        .filter(user::Column::Role.eq(UserRole::Recruiter))
        .filter(user::Column::Status.eq(UserStatus::Pending))
        .all(&state.db)
        .await?;

    let response = recruiters
        .into_iter()
        .map(|u| UserResponse {
            id: u.id,
            full_name: u.full_name,
            email: u.email,
            role: u.role,
            status: u.status,
            is_verified: u.is_verified,
            created_at: u.created_at,
        })
        .collect();

    Ok(Json(response))
}

#[utoipa::path(
    patch,
    path = "/admin/recruiters/{id}/approve",
    tag = "Admin",
    security(("bearerAuth" = [])),
    params(
        ("id" = i32, Path, description = "Recruiter User ID")
    ),
    responses(
        (status = 200, description = "Recruiter approved successfully", body = UserResponse),
        (status = 400, description = "User is not a recruiter or already active", body = ErrorDetail),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Forbidden (admin only)", body = ErrorDetail),
        (status = 404, description = "User not found", body = ErrorDetail)
    )
)]
pub async fn approve_recruiter(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<i32>,
) -> Result<Json<UserResponse>, AppError> {
    let target = User::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if target.role != UserRole::Recruiter {
        return Err(AppError::Validation("User is not a recruiter".to_string()));
    }

    let mut active: user::ActiveModel = target.into();
    active.status = Set(UserStatus::Active);
    let updated = active.update(&state.db).await?;

    // Brevo email trigger
    crate::email::send_recruiter_approved_email(
        state.config.clone(),
        updated.email.clone(),
        updated.full_name.clone(),
    );

    // In-app notification
    send_notification(
        state.db.clone(),
        updated.id,
        "Account Approved".to_string(),
        "Your recruiter account has been approved. You can now log in and post opportunities."
            .to_string(),
    );

    Ok(Json(UserResponse {
        id: updated.id,
        full_name: updated.full_name,
        email: updated.email,
        role: updated.role,
        status: updated.status,
        is_verified: updated.is_verified,
        created_at: updated.created_at,
    }))
}

#[utoipa::path(
    patch,
    path = "/admin/users/{id}/suspend",
    tag = "Admin",
    security(("bearerAuth" = [])),
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    request_body(content = Option<SuspendUserRequest>, description = "Optional suspension reason"),
    responses(
        (status = 200, description = "User suspended successfully", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Forbidden (admin only)", body = ErrorDetail),
        (status = 404, description = "User not found", body = ErrorDetail)
    )
)]
pub async fn suspend_user(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<i32>,
    body: Option<Json<SuspendUserRequest>>,
) -> Result<Json<UserResponse>, AppError> {
    let target = User::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let reason = body.and_then(|b| b.reason.clone());

    let mut active: user::ActiveModel = target.into();
    active.status = Set(UserStatus::Suspended);
    let updated = active.update(&state.db).await?;

    // Revoke all refresh tokens for user immediately
    let _ = RefreshToken::update_many()
        .filter(refresh_token::Column::UserId.eq(updated.id))
        .col_expr(refresh_token::Column::Revoked, Expr::value(true))
        .exec(&state.db)
        .await;

    // Brevo email trigger
    crate::email::send_user_suspended_email(
        state.config.clone(),
        updated.email.clone(),
        updated.full_name.clone(),
        reason,
    );

    // In-app notification
    send_notification(
        state.db.clone(),
        updated.id,
        "Account Suspended".to_string(),
        "Your account has been suspended by an administrator.".to_string(),
    );

    Ok(Json(UserResponse {
        id: updated.id,
        full_name: updated.full_name,
        email: updated.email,
        role: updated.role,
        status: updated.status,
        is_verified: updated.is_verified,
        created_at: updated.created_at,
    }))
}

#[utoipa::path(
    patch,
    path = "/admin/users/{id}/unsuspend",
    tag = "Admin",
    security(("bearerAuth" = [])),
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User unsuspended successfully", body = UserResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Forbidden (admin only)", body = ErrorDetail),
        (status = 404, description = "User not found", body = ErrorDetail)
    )
)]
pub async fn unsuspend_user(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<i32>,
) -> Result<Json<UserResponse>, AppError> {
    let target = User::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let mut active: user::ActiveModel = target.into();
    active.status = Set(UserStatus::Active);
    let updated = active.update(&state.db).await?;

    // Brevo email trigger
    crate::email::send_user_unsuspended_email(
        state.config.clone(),
        updated.email.clone(),
        updated.full_name.clone(),
    );

    // In-app notification
    send_notification(
        state.db.clone(),
        updated.id,
        "Account Reactivated".to_string(),
        "Your account has been reactivated. You can now log in.".to_string(),
    );

    Ok(Json(UserResponse {
        id: updated.id,
        full_name: updated.full_name,
        email: updated.email,
        role: updated.role,
        status: updated.status,
        is_verified: updated.is_verified,
        created_at: updated.created_at,
    }))
}
