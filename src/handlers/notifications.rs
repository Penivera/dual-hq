use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set,
};

use crate::{
    db::AppState,
    entities::{notification, Notification},
    errors::{AppError, ErrorDetail},
    middleware::AuthenticatedUser,
    schemas::notification::NotificationResponse,
};

#[utoipa::path(
    get,
    path = "/notifications",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "User notifications list", body = Vec<NotificationResponse>),
        (status = 401, description = "Unauthorized", body = ErrorDetail)
    )
)]
pub async fn list_notifications(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<NotificationResponse>>, AppError> {
    let notifications = Notification::find()
        .filter(notification::Column::UserId.eq(user.id))
        .order_by_asc(notification::Column::Read)
        .order_by_desc(notification::Column::CreatedAt)
        .all(&state.db)
        .await?;

    let response = notifications
        .into_iter()
        .map(|n| NotificationResponse {
            id: n.id,
            user_id: n.user_id,
            title: n.title,
            body: n.body,
            read: n.read,
            created_at: n.created_at,
        })
        .collect();

    Ok(Json(response))
}

#[utoipa::path(
    patch,
    path = "/notifications/{id}/read",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    params(
        ("id" = i32, Path, description = "Notification ID")
    ),
    responses(
        (status = 200, description = "Notification marked as read", body = NotificationResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 404, description = "Notification not found", body = ErrorDetail)
    )
)]
pub async fn mark_notification_read(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> Result<Json<NotificationResponse>, AppError> {
    let notif = Notification::find_by_id(id)
        .filter(notification::Column::UserId.eq(user.id))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Notification not found".to_string()))?;

    let mut active: notification::ActiveModel = notif.into();
    active.read = Set(true);
    let updated = active.update(&state.db).await?;

    Ok(Json(NotificationResponse {
        id: updated.id,
        user_id: updated.user_id,
        title: updated.title,
        body: updated.body,
        read: updated.read,
        created_at: updated.created_at,
    }))
}

#[utoipa::path(
    patch,
    path = "/notifications/read-all",
    tag = "Notifications",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "All notifications marked as read"),
        (status = 401, description = "Unauthorized", body = ErrorDetail)
    )
)]
pub async fn mark_all_notifications_read(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<StatusCode, AppError> {
    Notification::update_many()
        .filter(notification::Column::UserId.eq(user.id))
        .filter(notification::Column::Read.eq(false))
        .col_expr(notification::Column::Read, Expr::value(true))
        .exec(&state.db)
        .await?;

    Ok(StatusCode::OK)
}
