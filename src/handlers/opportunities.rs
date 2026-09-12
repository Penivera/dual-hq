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
        OpportunityCreate, OpportunityResponse, OpportunityUpdate, PaginationQuery,
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
    let skip = query.skip.unwrap_or(0);
    let limit = query.limit.unwrap_or(20).clamp(1, 100);

    let opportunities = Opportunity::find()
        .filter(opportunity::Column::Status.eq(OpportunityStatus::Open))
        .order_by(opportunity::Column::CreatedAt, Order::Desc)
        .offset(skip)
        .limit(limit)
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
        (status = 403, description = "Admin access required", body = ErrorDetail)
    )
)]
pub async fn create_opportunity(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(payload): Json<OpportunityCreate>,
) -> Result<impl IntoResponse, AppError> {
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

    // Trigger notification hook for all registered student/applicant users
    let db_clone = state.db.clone();
    let config_clone = state.config.clone();
    let opp_id = opp.id;
    let title = opp.title.clone();
    let company = opp.company.clone();
    let location = opp.location.clone();
    let opp_type = format!("{:?}", opp.type_);
    tokio::spawn(async move {
        if let Ok(students) = crate::entities::User::find()
            .filter(crate::entities::user::Column::Role.eq(crate::entities::user::UserRole::Applicant))
            .all(&db_clone)
            .await
        {
            for student in students {
                crate::email::send_new_opportunity_notification_email(
                    config_clone.clone(),
                    student.email,
                    student.full_name,
                    title.clone(),
                    company.clone(),
                    location.clone(),
                    opp_type.clone(),
                    None,
                    opp_id,
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
    request_body = OpportunityUpdate,
    responses(
        (status = 200, description = "Opportunity updated (admin only)", body = OpportunityResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Admin access required", body = ErrorDetail),
        (status = 404, description = "Opportunity not found", body = ErrorDetail)
    )
)]
pub async fn update_opportunity(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<i32>,
    Json(payload): Json<OpportunityUpdate>,
) -> Result<Json<OpportunityResponse>, AppError> {
    let opp = Opportunity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Opportunity not found".to_string()))?;

    let mut active: opportunity::ActiveModel = opp.into();

    if let Some(title) = payload.title {
        active.title = Set(title);
    }
    if let Some(description) = payload.description {
        active.description = Set(description);
    }
    if let Some(company) = payload.company {
        active.company = Set(company);
    }
    if let Some(location) = payload.location {
        active.location = Set(location);
    }
    if let Some(opp_type) = payload.type_ {
        active.type_ = Set(opp_type);
    }
    if let Some(status) = payload.status {
        active.status = Set(status);
    }
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
