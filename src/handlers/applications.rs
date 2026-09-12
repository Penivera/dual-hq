use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, Order, QueryFilter, QueryOrder, Set,
};

use crate::{
    db::AppState,
    entities::{
        application::{self, ApplicationStatus},
        opportunity::OpportunityStatus,
        user::UserRole,
        Application, Opportunity,
    },
    errors::{AppError, ErrorDetail},
    middleware::{AdminUser, AuthenticatedUser},
    schemas::application::{ApplicationCreate, ApplicationResponse, ApplicationStatusUpdate},
};

fn model_to_response(app: application::Model) -> ApplicationResponse {
    ApplicationResponse {
        id: app.id,
        user_id: app.user_id,
        opportunity_id: app.opportunity_id,
        cover_letter: app.cover_letter,
        status: app.status,
        applied_at: app.applied_at,
        updated_at: app.updated_at,
    }
}

#[utoipa::path(
    post,
    path = "/applications",
    tag = "Applications",
    security(("bearerAuth" = [])),
    request_body = ApplicationCreate,
    responses(
        (status = 201, description = "Application submitted successfully", body = ApplicationResponse),
        (status = 400, description = "Cannot apply to a closed opportunity", body = ErrorDetail),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 404, description = "Opportunity not found", body = ErrorDetail),
        (status = 409, description = "You have already applied to this opportunity", body = ErrorDetail)
    )
)]
pub async fn apply(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Json(payload): Json<ApplicationCreate>,
) -> Result<impl IntoResponse, AppError> {
    // 1. Check opportunity exists and is open
    let opp = Opportunity::find_by_id(payload.opportunity_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Opportunity not found".to_string()))?;

    if opp.status != OpportunityStatus::Open {
        return Err(AppError::BadRequest(
            "Cannot apply to a closed opportunity".to_string(),
        ));
    }

    // 2. Check for duplicate application
    let existing_app = Application::find()
        .filter(application::Column::UserId.eq(user.id))
        .filter(application::Column::OpportunityId.eq(payload.opportunity_id))
        .one(&state.db)
        .await?;

    if existing_app.is_some() {
        return Err(AppError::Conflict(
            "You have already applied to this opportunity".to_string(),
        ));
    }

    let now = chrono::Utc::now().into();
    let new_app = application::ActiveModel {
        user_id: Set(user.id),
        opportunity_id: Set(payload.opportunity_id),
        cover_letter: Set(payload.cover_letter),
        status: Set(ApplicationStatus::Pending),
        applied_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let app = new_app.insert(&state.db).await?;

    // Send confirmation email to applicant
    if let Ok(Some(applicant)) = crate::entities::User::find_by_id(user.id).one(&state.db).await {
        crate::email::send_application_submitted_email(
            state.config.clone(),
            applicant.email,
            applicant.full_name,
            opp.title,
            opp.company,
        );
    }

    Ok((StatusCode::CREATED, Json(model_to_response(app))))
}

#[utoipa::path(
    get,
    path = "/applications",
    tag = "Applications",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "List all applications (admin only)", body = [ApplicationResponse]),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Admin access required", body = ErrorDetail)
    )
)]
pub async fn list_applications(
    State(state): State<AppState>,
    _admin: AdminUser,
) -> Result<Json<Vec<ApplicationResponse>>, AppError> {
    let applications = Application::find()
        .order_by(application::Column::AppliedAt, Order::Desc)
        .all(&state.db)
        .await?;

    let response = applications.into_iter().map(model_to_response).collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/applications/me",
    tag = "Applications",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Current user's applications", body = [ApplicationResponse]),
        (status = 401, description = "Unauthorized", body = ErrorDetail)
    )
)]
pub async fn my_applications(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<ApplicationResponse>>, AppError> {
    let applications = Application::find()
        .filter(application::Column::UserId.eq(user.id))
        .order_by(application::Column::AppliedAt, Order::Desc)
        .all(&state.db)
        .await?;

    let response = applications.into_iter().map(model_to_response).collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/applications/{id}",
    tag = "Applications",
    security(("bearerAuth" = [])),
    params(
        ("id" = i32, Path, description = "Application ID")
    ),
    responses(
        (status = 200, description = "Get application by ID (owner or admin)", body = ApplicationResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Access denied", body = ErrorDetail),
        (status = 404, description = "Application not found", body = ErrorDetail)
    )
)]
pub async fn get_application(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> Result<Json<ApplicationResponse>, AppError> {
    let app = Application::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Application not found".to_string()))?;

    if app.user_id != user.id && user.role != UserRole::Admin {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    Ok(Json(model_to_response(app)))
}

#[utoipa::path(
    patch,
    path = "/applications/{id}/status",
    tag = "Applications",
    security(("bearerAuth" = [])),
    params(
        ("id" = i32, Path, description = "Application ID")
    ),
    request_body = ApplicationStatusUpdate,
    responses(
        (status = 200, description = "Application status updated", body = ApplicationResponse),
        (status = 400, description = "Invalid status transition", body = ErrorDetail),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Access denied", body = ErrorDetail),
        (status = 404, description = "Application not found", body = ErrorDetail)
    )
)]
pub async fn update_application_status(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
    Json(payload): Json<ApplicationStatusUpdate>,
) -> Result<Json<ApplicationResponse>, AppError> {
    let app = Application::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Application not found".to_string()))?;

    if user.role == UserRole::Admin {
        if app.status != ApplicationStatus::Pending {
            return Err(AppError::BadRequest(
                "Can only accept or reject pending applications".to_string(),
            ));
        }
        if payload.status != ApplicationStatus::Accepted
            && payload.status != ApplicationStatus::Rejected
        {
            return Err(AppError::BadRequest(
                "Admin can only set status to accepted or rejected".to_string(),
            ));
        }
    } else {
        // Regular user
        if app.user_id != user.id {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }
        if payload.status != ApplicationStatus::Withdrawn {
            return Err(AppError::BadRequest(
                "You can only withdraw your application".to_string(),
            ));
        }
        if app.status != ApplicationStatus::Pending {
            return Err(AppError::BadRequest(
                "Can only withdraw pending applications".to_string(),
            ));
        }
    }

    let mut active: application::ActiveModel = app.into();
    active.status = Set(payload.status);
    active.updated_at = Set(chrono::Utc::now().into());

    let updated = active.update(&state.db).await?;

    // Send status update notification email to applicant
    let db_clone = state.db.clone();
    let config_clone = state.config.clone();
    let user_id = updated.user_id;
    let opp_id = updated.opportunity_id;
    let status_str = format!("{:?}", updated.status);
    tokio::spawn(async move {
        if let (Ok(Some(applicant)), Ok(Some(opp))) = (
            crate::entities::User::find_by_id(user_id).one(&db_clone).await,
            Opportunity::find_by_id(opp_id).one(&db_clone).await,
        ) {
            crate::email::send_application_status_update_email(
                config_clone,
                applicant.email,
                applicant.full_name,
                opp.title,
                opp.company,
                status_str,
            );
        }
    });

    Ok(Json(model_to_response(updated)))
}

#[utoipa::path(
    delete,
    path = "/applications/{id}",
    tag = "Applications",
    security(("bearerAuth" = [])),
    params(
        ("id" = i32, Path, description = "Application ID")
    ),
    responses(
        (status = 204, description = "Application withdrawn/deleted (user only)"),
        (status = 400, description = "Can only delete pending applications", body = ErrorDetail),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Access denied", body = ErrorDetail),
        (status = 404, description = "Application not found", body = ErrorDetail)
    )
)]
pub async fn delete_application(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let app = Application::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Application not found".to_string()))?;

    if app.user_id != user.id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    if app.status != ApplicationStatus::Pending {
        return Err(AppError::BadRequest(
            "Can only delete pending applications".to_string(),
        ));
    }

    app.delete(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}
