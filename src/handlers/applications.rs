use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, Order,
    QueryFilter, QueryOrder, QuerySelect, Set,
};

use crate::{
    db::AppState,
    entities::{
        application::{self, ApplicationStatus},
        opportunity::OpportunityStatus,
        profile,
        user::UserRole,
        Application, Opportunity, Profile,
    },
    errors::{AppError, ErrorDetail},
    middleware::{ApplicantUser, AuthenticatedUser, RecruiterOrAdminUser},
    notifications::send_notification,
    schemas::application::{
        ApplicationCreate, ApplicationDetailResponse, ApplicationResponse, ApplicationStatusUpdate,
    },
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

async fn build_application_detail(
    db: &DatabaseConnection,
    app: application::Model,
    base_url: &str,
) -> Result<ApplicationDetailResponse, AppError> {
    let opp = Opportunity::find_by_id(app.opportunity_id).one(db).await?;
    let (opp_title, opp_company) = match opp {
        Some(o) => (o.title, o.company),
        None => ("Deleted Opportunity".to_string(), "Unknown".to_string()),
    };

    let prof = Profile::find()
        .filter(profile::Column::UserId.eq(app.user_id))
        .one(db)
        .await?;

    let cv_url = prof.as_ref().and_then(|p| p.cv_url.clone());
    let profile_url = Some(format!("{}/profile/{}", base_url.trim_end_matches('/'), app.user_id));

    Ok(ApplicationDetailResponse {
        id: app.id,
        user_id: app.user_id,
        opportunity_id: app.opportunity_id,
        opportunity_title: opp_title,
        company: opp_company,
        cover_letter: app.cover_letter,
        status: app.status,
        applied_at: app.applied_at,
        updated_at: app.updated_at,
        applicant_cv_url: cv_url,
        applicant_profile_url: profile_url,
    })
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
        (status = 403, description = "Applicant access required", body = ErrorDetail),
        (status = 404, description = "Opportunity not found", body = ErrorDetail),
        (status = 409, description = "You have already applied to this opportunity", body = ErrorDetail)
    )
)]
pub async fn apply(
    State(state): State<AppState>,
    applicant_user: ApplicantUser,
    Json(payload): Json<ApplicationCreate>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = applicant_user.0.id;

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
        .filter(application::Column::UserId.eq(user_id))
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
        user_id: Set(user_id),
        opportunity_id: Set(payload.opportunity_id),
        cover_letter: Set(payload.cover_letter),
        status: Set(ApplicationStatus::Pending),
        applied_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let app = new_app.insert(&state.db).await?;

    // Notification to recruiter if opportunity has creator
    if let Some(creator_id) = opp.created_by {
        send_notification(
            state.db.clone(),
            creator_id,
            "New Application Received".to_string(),
            format!("A new application has been submitted for {}", opp.title),
        );
    }

    // Asynchronously dispatch confirmation email without blocking the response
    let db_clone = state.db.clone();
    let config_clone = state.config.clone();
    let opp_title = opp.title;
    let opp_company = opp.company;
    tokio::spawn(async move {
        if let Ok(Some(applicant)) = crate::entities::User::find_by_id(user_id).one(&db_clone).await {
            crate::email::send_application_submitted_email(
                config_clone,
                applicant.email,
                applicant.full_name,
                opp_title,
                opp_company,
            );
        }
    });

    Ok((StatusCode::CREATED, Json(model_to_response(app))))
}

#[utoipa::path(
    get,
    path = "/applications",
    tag = "Applications",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "List applications (recruiter or admin)", body = Vec<ApplicationDetailResponse>),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Recruiter or admin access required", body = ErrorDetail)
    )
)]
pub async fn list_applications(
    State(state): State<AppState>,
    user: RecruiterOrAdminUser,
) -> Result<Json<Vec<ApplicationDetailResponse>>, AppError> {
    let applications = if user.0.role == UserRole::Admin {
        Application::find()
            .order_by(application::Column::AppliedAt, Order::Desc)
            .limit(100)
            .all(&state.db)
            .await?
    } else {
        // Recruiter: only opportunities created by this recruiter
        let my_opps = Opportunity::find()
            .filter(crate::entities::opportunity::Column::CreatedBy.eq(user.0.id))
            .all(&state.db)
            .await?;
        let my_opp_ids: Vec<i32> = my_opps.into_iter().map(|o| o.id).collect();

        if my_opp_ids.is_empty() {
            return Ok(Json(Vec::new()));
        }

        Application::find()
            .filter(application::Column::OpportunityId.is_in(my_opp_ids))
            .order_by(application::Column::AppliedAt, Order::Desc)
            .limit(100)
            .all(&state.db)
            .await?
    };

    let mut response = Vec::with_capacity(applications.len());
    for app in applications {
        let detail = build_application_detail(&state.db, app, &state.config.app_base_url).await?;
        response.push(detail);
    }

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/applications/me",
    tag = "Applications",
    security(("bearerAuth" = [])),
    responses(
        (status = 200, description = "Current user's applications with opportunity details joined", body = Vec<ApplicationDetailResponse>),
        (status = 401, description = "Unauthorized", body = ErrorDetail)
    )
)]
pub async fn my_applications(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<ApplicationDetailResponse>>, AppError> {
    let applications = Application::find()
        .filter(application::Column::UserId.eq(user.id))
        .order_by(application::Column::AppliedAt, Order::Desc)
        .limit(100)
        .all(&state.db)
        .await?;

    let mut response = Vec::with_capacity(applications.len());
    for app in applications {
        let detail = build_application_detail(&state.db, app, &state.config.app_base_url).await?;
        response.push(detail);
    }

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
        (status = 200, description = "Get application detail by ID", body = ApplicationDetailResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Access denied", body = ErrorDetail),
        (status = 404, description = "Application not found", body = ErrorDetail)
    )
)]
pub async fn get_application(
    State(state): State<AppState>,
    user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> Result<Json<ApplicationDetailResponse>, AppError> {
    let app = Application::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Application not found".to_string()))?;

    if user.role == UserRole::Applicant && app.user_id != user.id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    if user.role == UserRole::Recruiter {
        let opp = Opportunity::find_by_id(app.opportunity_id)
            .one(&state.db)
            .await?
            .ok_or_else(|| AppError::NotFound("Associated opportunity not found".to_string()))?;

        if opp.created_by != Some(user.id) {
            return Err(AppError::Forbidden(
                "You can only view applications to your own opportunities".to_string(),
            ));
        }
    }

    let detail = build_application_detail(&state.db, app, &state.config.app_base_url).await?;
    Ok(Json(detail))
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
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Access denied", body = ErrorDetail),
        (status = 404, description = "Application not found", body = ErrorDetail),
        (status = 422, description = "Invalid status transition", body = ErrorDetail)
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

    let opp = Opportunity::find_by_id(app.opportunity_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Associated opportunity not found".to_string()))?;

    if user.role == UserRole::Admin {
        if payload.status != ApplicationStatus::Accepted
            && payload.status != ApplicationStatus::Rejected
        {
            return Err(AppError::Validation(
                "Admin can only transition status to accepted or rejected".to_string(),
            ));
        }
    } else if user.role == UserRole::Recruiter {
        if opp.created_by != Some(user.id) {
            return Err(AppError::Forbidden(
                "You can only manage applications to your own opportunities".to_string(),
            ));
        }
        if payload.status != ApplicationStatus::Accepted
            && payload.status != ApplicationStatus::Rejected
        {
            return Err(AppError::Validation(
                "Recruiters can only transition status to accepted or rejected".to_string(),
            ));
        }
    } else {
        // Applicant
        if app.user_id != user.id {
            return Err(AppError::Forbidden("Access denied".to_string()));
        }
        if payload.status != ApplicationStatus::Withdrawn {
            return Err(AppError::Validation(
                "Applicants can only transition status to withdrawn".to_string(),
            ));
        }
        if app.status != ApplicationStatus::Pending {
            return Err(AppError::Validation(
                "Only pending applications can be withdrawn".to_string(),
            ));
        }
    }

    let mut active: application::ActiveModel = app.into();
    active.status = Set(payload.status.clone());
    active.updated_at = Set(chrono::Utc::now().into());

    let updated = active.update(&state.db).await?;

    // In-app notifications
    match payload.status {
        ApplicationStatus::Accepted | ApplicationStatus::Rejected => {
            send_notification(
                state.db.clone(),
                updated.user_id,
                "Application Status Update".to_string(),
                format!(
                    "Your application for {} has been updated to {:?}",
                    opp.title, updated.status
                ),
            );
        }
        ApplicationStatus::Withdrawn => {
            if let Some(creator_id) = opp.created_by {
                send_notification(
                    state.db.clone(),
                    creator_id,
                    "Application Withdrawn".to_string(),
                    format!("An applicant has withdrawn their application for {}", opp.title),
                );
            }
        }
        _ => {}
    }

    // Fire-and-forget notification email to applicant if status accepted/rejected
    let db_clone = state.db.clone();
    let config_clone = state.config.clone();
    let user_id = updated.user_id;
    let opp_title = opp.title;
    let opp_company = opp.company;
    let status_str = format!("{:?}", updated.status);
    tokio::spawn(async move {
        if let Ok(Some(applicant)) = crate::entities::User::find_by_id(user_id).one(&db_clone).await {
            crate::email::send_application_status_update_email(
                config_clone,
                applicant.email,
                applicant.full_name,
                opp_title,
                opp_company,
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
        (status = 204, description = "Application deleted/withdrawn (applicant only)"),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Access denied", body = ErrorDetail),
        (status = 404, description = "Application not found", body = ErrorDetail),
        (status = 422, description = "Can only delete pending applications", body = ErrorDetail)
    )
)]
pub async fn delete_application(
    State(state): State<AppState>,
    applicant: ApplicantUser,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let app = Application::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Application not found".to_string()))?;

    if app.user_id != applicant.0.id {
        return Err(AppError::Forbidden("Access denied".to_string()));
    }

    if app.status != ApplicationStatus::Pending {
        return Err(AppError::Validation(
            "Can only delete pending applications".to_string(),
        ));
    }

    let opp = Opportunity::find_by_id(app.opportunity_id).one(&state.db).await?;
    if let Some(o) = opp {
        if let Some(creator_id) = o.created_by {
            send_notification(
                state.db.clone(),
                creator_id,
                "Application Withdrawn".to_string(),
                format!("An applicant has withdrawn their application for {}", o.title),
            );
        }
    }

    app.delete(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}
