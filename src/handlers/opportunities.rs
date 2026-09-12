use std::collections::HashMap;

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, EntityTrait, ModelTrait, Order, PaginatorTrait,
    QueryFilter, QueryOrder, QuerySelect, Set,
};

use crate::{
    db::AppState,
    entities::{
        category,
        opportunity::{self, OpportunityStatus},
        user::UserRole,
        Application, Category, Opportunity,
    },
    errors::{AppError, ErrorDetail},
    middleware::RecruiterOrAdminUser,
    schemas::{
        auth::Claims,
        opportunity::{
            OpportunityCreate, OpportunityFullUpdate, OpportunityResponse, PaginationQuery,
        },
    },
};

fn extract_optional_user(headers: &HeaderMap, secret: &str) -> Option<(i32, String)> {
    let auth_header = headers.get(header::AUTHORIZATION)?.to_str().ok()?;
    let token = auth_header
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.strip_prefix("bearer "))?;

    let mut validation = Validation::default();
    validation.validate_exp = true;

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .ok()?;

    let user_id = token_data.claims.sub.parse::<i32>().ok()?;
    Some((user_id, token_data.claims.role))
}

fn model_to_response(
    opp: opportunity::Model,
    category_name: Option<String>,
    application_count: Option<i64>,
) -> OpportunityResponse {
    OpportunityResponse {
        id: opp.id,
        title: opp.title,
        description: opp.description,
        company: opp.company,
        location: opp.location,
        type_: opp.type_,
        status: opp.status,
        created_by: opp.created_by,
        category_id: opp.category_id,
        category_name,
        deadline: opp.deadline,
        application_count,
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
        (status = 200, description = "List opportunities with filters and pagination", body = Vec<OpportunityResponse>)
    )
)]
pub async fn list_opportunities(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<Vec<OpportunityResponse>>, AppError> {
    let caller = extract_optional_user(&headers, &state.config.jwt_secret);

    let page = query.page.unwrap_or(1).max(1);
    let per_page = query.per_page.or(query.limit).unwrap_or(20).clamp(1, 100);
    let offset = query.skip.unwrap_or((page - 1) * per_page);

    let mut select = Opportunity::find();

    // Only show open opportunities unless admin
    let is_admin = caller.as_ref().map(|(_, role)| role == "admin").unwrap_or(false);
    if !is_admin {
        select = select.filter(opportunity::Column::Status.eq(OpportunityStatus::Open));
    }

    // Composable filter: category slug
    if let Some(ref cat_slug) = query.category {
        if !cat_slug.trim().is_empty() {
            let cat = Category::find()
                .filter(category::Column::Slug.eq(cat_slug.trim()))
                .one(&state.db)
                .await?;
            match cat {
                Some(c) => {
                    select = select.filter(opportunity::Column::CategoryId.eq(c.id));
                }
                None => {
                    // Slug didn't match any category -> return empty list
                    return Ok(Json(Vec::new()));
                }
            }
        }
    }

    // Composable filter: search query across title, description, company
    if let Some(ref s) = query.search {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            select = select.filter(
                Condition::any()
                    .add(opportunity::Column::Title.contains(trimmed))
                    .add(opportunity::Column::Description.contains(trimmed))
                    .add(opportunity::Column::Company.contains(trimmed)),
            );
        }
    }

    // Composable filter: type
    if let Some(ref t) = query.type_ {
        select = select.filter(opportunity::Column::Type.eq(t.clone()));
    }

    // Composable filter: location
    if let Some(ref loc) = query.location {
        let trimmed = loc.trim();
        if !trimmed.is_empty() {
            select = select.filter(opportunity::Column::Location.contains(trimmed));
        }
    }

    let opportunities = select
        .order_by(opportunity::Column::CreatedAt, Order::Desc)
        .offset(offset)
        .limit(per_page)
        .all(&state.db)
        .await?;

    // Bulk resolve category names
    let cat_ids: Vec<i32> = opportunities
        .iter()
        .filter_map(|o| o.category_id)
        .collect();

    let categories_map: HashMap<i32, String> = if !cat_ids.is_empty() {
        Category::find()
            .filter(category::Column::Id.is_in(cat_ids))
            .all(&state.db)
            .await?
            .into_iter()
            .map(|c| (c.id, c.name))
            .collect()
    } else {
        HashMap::new()
    };

    // Bulk count applications if caller is admin or recruiter
    let mut app_counts: HashMap<i32, i64> = HashMap::new();
    if let Some((caller_id, ref role)) = caller {
        let relevant_opp_ids: Vec<i32> = opportunities
            .iter()
            .filter(|o| role == "admin" || (role == "recruiter" && o.created_by == Some(caller_id)))
            .map(|o| o.id)
            .collect();

        for opp_id in relevant_opp_ids {
            let count = Application::find()
                .filter(crate::entities::application::Column::OpportunityId.eq(opp_id))
                .count(&state.db)
                .await? as i64;
            app_counts.insert(opp_id, count);
        }
    }

    let response = opportunities
        .into_iter()
        .map(|opp| {
            let cat_name = opp.category_id.and_then(|id| categories_map.get(&id).cloned());
            let app_count = app_counts.get(&opp.id).copied();
            model_to_response(opp, cat_name, app_count)
        })
        .collect();

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
        (status = 404, description = "Opportunity not found", body = ErrorDetail)
    )
)]
pub async fn get_opportunity(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i32>,
) -> Result<Json<OpportunityResponse>, AppError> {
    let opp = Opportunity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Opportunity not found".to_string()))?;

    let cat_name = match opp.category_id {
        Some(cid) => Category::find_by_id(cid)
            .one(&state.db)
            .await?
            .map(|c| c.name),
        None => None,
    };

    let caller = extract_optional_user(&headers, &state.config.jwt_secret);
    let app_count = match caller {
        Some((caller_id, ref role))
            if role == "admin" || (role == "recruiter" && opp.created_by == Some(caller_id)) =>
        {
            let count = Application::find()
                .filter(crate::entities::application::Column::OpportunityId.eq(opp.id))
                .count(&state.db)
                .await? as i64;
            Some(count)
        }
        _ => None,
    };

    Ok(Json(model_to_response(opp, cat_name, app_count)))
}

#[utoipa::path(
    post,
    path = "/opportunities",
    tag = "Opportunities",
    security(("bearerAuth" = [])),
    request_body = OpportunityCreate,
    responses(
        (status = 201, description = "Opportunity created (recruiter or admin)", body = OpportunityResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Recruiter or admin access required", body = ErrorDetail),
        (status = 422, description = "Validation error on missing or empty fields", body = ErrorDetail)
    )
)]
pub async fn create_opportunity(
    State(state): State<AppState>,
    user: RecruiterOrAdminUser,
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
        created_by: Set(Some(user.0.id)),
        category_id: Set(payload.category_id),
        deadline: Set(payload.deadline),
        created_at: Set(now),
        updated_at: Set(now),
        ..Default::default()
    };

    let opp = new_opp.insert(&state.db).await?;

    let cat_name = match opp.category_id {
        Some(cid) => Category::find_by_id(cid)
            .one(&state.db)
            .await?
            .map(|c| c.name),
        None => None,
    };

    // Broadcast notification to applicants
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

    Ok((
        StatusCode::CREATED,
        Json(model_to_response(opp, cat_name, Some(0))),
    ))
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
        (status = 200, description = "Opportunity updated", body = OpportunityResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Forbidden (recruiter can only update own listings)", body = ErrorDetail),
        (status = 404, description = "Opportunity not found", body = ErrorDetail),
        (status = 422, description = "Validation error on missing or empty fields", body = ErrorDetail)
    )
)]
pub async fn update_opportunity(
    State(state): State<AppState>,
    user: RecruiterOrAdminUser,
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

    // Recruiters can only modify their own opportunities
    if user.0.role == UserRole::Recruiter && opp.created_by != Some(user.0.id) {
        return Err(AppError::Forbidden(
            "You can only modify your own opportunities".to_string(),
        ));
    }

    let mut active: opportunity::ActiveModel = opp.into();
    active.title = Set(payload.title);
    active.description = Set(payload.description);
    active.company = Set(payload.company);
    active.location = Set(payload.location);
    active.type_ = Set(payload.type_);
    active.status = Set(payload.status);
    if payload.category_id.is_some() {
        active.category_id = Set(payload.category_id);
    }
    if payload.deadline.is_some() {
        active.deadline = Set(payload.deadline);
    }
    active.updated_at = Set(chrono::Utc::now().into());

    let updated = active.update(&state.db).await?;

    let cat_name = match updated.category_id {
        Some(cid) => Category::find_by_id(cid)
            .one(&state.db)
            .await?
            .map(|c| c.name),
        None => None,
    };

    let app_count = Application::find()
        .filter(crate::entities::application::Column::OpportunityId.eq(updated.id))
        .count(&state.db)
        .await? as i64;

    Ok(Json(model_to_response(
        updated,
        cat_name,
        Some(app_count),
    )))
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
        (status = 204, description = "Opportunity deleted"),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Forbidden (recruiter can only delete own listings)", body = ErrorDetail),
        (status = 404, description = "Opportunity not found", body = ErrorDetail)
    )
)]
pub async fn delete_opportunity(
    State(state): State<AppState>,
    user: RecruiterOrAdminUser,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let opp = Opportunity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Opportunity not found".to_string()))?;

    // Recruiters can only delete their own opportunities
    if user.0.role == UserRole::Recruiter && opp.created_by != Some(user.0.id) {
        return Err(AppError::Forbidden(
            "You can only delete your own opportunities".to_string(),
        ));
    }

    crate::entities::Application::delete_many()
        .filter(crate::entities::application::Column::OpportunityId.eq(id))
        .exec(&state.db)
        .await?;

    opp.delete(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Automatically closes open opportunities whose deadlines have passed,
/// sends in-app notifications to recruiters, and returns the count of closed opportunities.
pub async fn expire_opportunities(db: &sea_orm::DatabaseConnection) -> Result<usize, sea_orm::DbErr> {
    use crate::notifications::send_notification;

    let now = chrono::Utc::now().fixed_offset();
    let expired_opps = Opportunity::find()
        .filter(opportunity::Column::Status.eq(OpportunityStatus::Open))
        .filter(opportunity::Column::Deadline.is_not_null())
        .filter(opportunity::Column::Deadline.lt(now))
        .all(db)
        .await?;

    let count = expired_opps.len();
    for opp in expired_opps {
        let opp_id = opp.id;
        let opp_title = opp.title.clone();
        let creator_id = opp.created_by;

        let mut active: opportunity::ActiveModel = opp.into();
        active.status = Set(OpportunityStatus::Closed);
        active.updated_at = Set(chrono::Utc::now().into());
        active.update(db).await?;

        if let Some(user_id) = creator_id {
            send_notification(
                db.clone(),
                user_id,
                "Opportunity Expired".to_string(),
                format!("Your opportunity '{opp_title}' (id: {opp_id}) has reached its deadline and was closed."),
            );
        }
    }

    tracing::info!(expired_count = count, "Opportunity auto-expiry job completed");
    Ok(count)
}
