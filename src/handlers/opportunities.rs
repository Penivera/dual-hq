use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, Order, QueryFilter, QueryOrder,
    QuerySelect, Set,
};

use crate::{
    db::AppState,
    entities::{
        opportunity::{self, OpportunityStatus},
        Opportunity,
    },
    errors::{AppError, ErrorDetail},
    middleware::{AdminUser, AuthenticatedUser},
    schemas::opportunity::{
        OpportunityCreate, OpportunityFullUpdate, OpportunityResponse, PaginationQuery,
    },
};

fn model_to_response(opp: opportunity::Model) -> OpportunityResponse {
    OpportunityResponse {
        id: opp.id,
        title: opp.title,
        description: opp.description,
        company: opp.company,
        location: opp.location,
        type_: opp.type_,
        status: opp.status,
        created_at: opp.created_at,
        updated_at: opp.updated_at,
    }
}

#[utoipa::path(
    get,
    path = "/opportunities",
    tag = "Opportunities",
    security(("bearerAuth" = [])),
    params(PaginationQuery),
    responses(
        (status = 200, description = "List open opportunities with pagination", body = [OpportunityResponse]),
        (status = 401, description = "Unauthorized", body = ErrorDetail)
    )
)]
pub async fn list_opportunities(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<Vec<OpportunityResponse>>, AppError> {
    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.or(query.limit).unwrap_or(20).clamp(1, 100);
    let offset = query.skip.unwrap_or((page - 1) * per_page);

    let opportunities = Opportunity::find()
        .filter(opportunity::Column::Status.eq(OpportunityStatus::Open))
        .order_by(opportunity::Column::CreatedAt, Order::Desc)
        .offset(offset)
        .limit(per_page)
        .all(&state.db)
        .await?;

    let response = opportunities.into_iter().map(model_to_response).collect();
    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/opportunities/{id}",
    tag = "Opportunities",
    security(("bearerAuth" = [])),
    params(
        ("id" = i32, Path, description = "Opportunity ID")
    ),
    responses(
        (status = 200, description = "Get opportunity by ID", body = OpportunityResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 404, description = "Opportunity not found", body = ErrorDetail)
    )
)]
pub async fn get_opportunity(
    State(state): State<AppState>,
    _user: AuthenticatedUser,
    Path(id): Path<i32>,
) -> Result<Json<OpportunityResponse>, AppError> {
    let opp = Opportunity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Opportunity not found".to_string()))?;

    Ok(Json(model_to_response(opp)))
}

#[utoipa::path(
    post,
    path = "/opportunities",
    tag = "Opportunities",
    security(("bearerAuth" = [])),
    request_body = OpportunityCreate,
    responses(
        (status = 201, description = "Opportunity created (admin only)", body = OpportunityResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Admin access required", body = ErrorDetail),
        (status = 422, description = "Validation error on missing or empty fields", body = ErrorDetail)
    )
)]
pub async fn create_opportunity(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(payload): Json<OpportunityCreate>,
) -> Result<impl IntoResponse, AppError> {
    if payload.title.trim().is_empty()
        || payload.description.trim().is_empty()
        || payload.company.trim().is_empty()
        || payload.location.trim().is_empty()
    {
        return Err(AppError::Validation(
            "Title, description, company, and location are required and cannot be empty".to_string(),
        ));
    }

    let now = chrono::Utc::now().into();
    let new_opp = opportunity::ActiveModel {
        title: Set(payload.title),
        description: Set(payload.description),
        company: Set(payload.company),
        location: Set(payload.location),
        type_: Set(payload.type_),
        status: Set(OpportunityStatus::Open),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let opp = new_opp.insert(&state.db).await?;

    // Trigger notification hook for registered student/applicant users (fire-and-forget background task)
    let db_clone = state.db.clone();
    let config_clone = state.config.clone();
    let opp_payload = crate::email::OpportunityNotificationPayload {
        title: opp.title.clone(),
        company: opp.company.clone(),
        location: opp.location.clone(),
        opp_type: format!("{:?}", opp.type_),
        stipend: None,
        opportunity_id: opp.id,
    };

    tokio::spawn(async move {
        if let Ok(students) = crate::entities::User::find()
            .filter(crate::entities::user::Column::Role.eq(crate::entities::user::UserRole::Applicant))
            .limit(1000)
            .all(&db_clone)
            .await
        {
            for student in students {
                crate::email::send_new_opportunity_notification_email(
                    config_clone.clone(),
                    student.email,
                    student.full_name,
                    opp_payload.clone(),
                );
            }
        }
    });

    Ok((StatusCode::CREATED, Json(model_to_response(opp))))
}

#[utoipa::path(
    put,
    path = "/opportunities/{id}",
    tag = "Opportunities",
    security(("bearerAuth" = [])),
    params(
        ("id" = i32, Path, description = "Opportunity ID")
    ),
    request_body = OpportunityFullUpdate,
    responses(
        (status = 200, description = "Opportunity updated (admin only)", body = OpportunityResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Admin access required", body = ErrorDetail),
        (status = 404, description = "Opportunity not found", body = ErrorDetail),
        (status = 422, description = "Validation error on missing or empty fields", body = ErrorDetail)
    )
)]
pub async fn update_opportunity(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<i32>,
    Json(payload): Json<OpportunityFullUpdate>,
) -> Result<Json<OpportunityResponse>, AppError> {
    if payload.title.trim().is_empty()
        || payload.description.trim().is_empty()
        || payload.company.trim().is_empty()
        || payload.location.trim().is_empty()
    {
        return Err(AppError::Validation(
            "Title, description, company, and location are required and cannot be empty".to_string(),
        ));
    }

    let opp = Opportunity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Opportunity not found".to_string()))?;

    let mut active: opportunity::ActiveModel = opp.into();
    active.title = Set(payload.title);
    active.description = Set(payload.description);
    active.company = Set(payload.company);
    active.location = Set(payload.location);
    active.type_ = Set(payload.type_);
    active.status = Set(payload.status);
    active.updated_at = Set(chrono::Utc::now().into());

    let updated = active.update(&state.db).await?;
    Ok(Json(model_to_response(updated)))
}

#[utoipa::path(
    delete,
    path = "/opportunities/{id}",
    tag = "Opportunities",
    security(("bearerAuth" = [])),
    params(
        ("id" = i32, Path, description = "Opportunity ID")
    ),
    responses(
        (status = 204, description = "Opportunity deleted (admin only)"),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Admin access required", body = ErrorDetail),
        (status = 404, description = "Opportunity not found", body = ErrorDetail)
    )
)]
pub async fn delete_opportunity(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let opp = Opportunity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Opportunity not found".to_string()))?;

    crate::entities::Application::delete_many()
        .filter(crate::entities::application::Column::OpportunityId.eq(id))
        .exec(&state.db)
        .await?;

    opp.delete(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}
