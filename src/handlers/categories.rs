use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};

use crate::{
    db::AppState,
    entities::{category, Category},
    errors::{AppError, ErrorDetail},
    middleware::AdminUser,
    schemas::category::{CategoryCreate, CategoryResponse},
};

fn slugify(text: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = true;

    for ch in text.chars() {
        if ch.is_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_dash = false;
        } else if !last_was_dash {
            slug.push('-');
            last_was_dash = true;
        }
    }

    if slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "category".to_string()
    } else {
        slug
    }
}

#[utoipa::path(
    get,
    path = "/categories",
    tag = "Categories",
    responses(
        (status = 200, description = "List all categories", body = Vec<CategoryResponse>)
    )
)]
pub async fn list_categories(
    State(state): State<AppState>,
) -> Result<Json<Vec<CategoryResponse>>, AppError> {
    let categories = Category::find()
        .order_by_asc(category::Column::Name)
        .all(&state.db)
        .await?;

    let response = categories
        .into_iter()
        .map(|c| CategoryResponse {
            id: c.id,
            name: c.name,
            slug: c.slug,
            created_at: c.created_at,
        })
        .collect();

    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/categories",
    tag = "Categories",
    security(("bearerAuth" = [])),
    request_body = CategoryCreate,
    responses(
        (status = 201, description = "Category created", body = CategoryResponse),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Forbidden (admin only)", body = ErrorDetail),
        (status = 409, description = "Category name or slug already exists", body = ErrorDetail),
        (status = 422, description = "Validation error", body = ErrorDetail)
    )
)]
pub async fn create_category(
    State(state): State<AppState>,
    _admin: AdminUser,
    Json(payload): Json<CategoryCreate>,
) -> Result<impl IntoResponse, AppError> {
    if payload.name.trim().is_empty() {
        return Err(AppError::Validation("Category name cannot be empty".to_string()));
    }

    let slug = payload
        .slug
        .filter(|s| !s.trim().is_empty())
        .map(|s| slugify(&s))
        .unwrap_or_else(|| slugify(&payload.name));

    let existing = Category::find()
        .filter(
            sea_orm::Condition::any()
                .add(category::Column::Name.eq(&payload.name))
                .add(category::Column::Slug.eq(&slug)),
        )
        .one(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::Conflict("Category with this name or slug already exists".to_string()));
    }

    let new_cat = category::ActiveModel {
        name: Set(payload.name),
        slug: Set(slug),
        created_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };

    let inserted = new_cat.insert(&state.db).await?;

    let response = CategoryResponse {
        id: inserted.id,
        name: inserted.name,
        slug: inserted.slug,
        created_at: inserted.created_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    delete,
    path = "/categories/{id}",
    tag = "Categories",
    security(("bearerAuth" = [])),
    params(
        ("id" = i32, Path, description = "Category ID")
    ),
    responses(
        (status = 204, description = "Category deleted"),
        (status = 401, description = "Unauthorized", body = ErrorDetail),
        (status = 403, description = "Forbidden (admin only)", body = ErrorDetail),
        (status = 404, description = "Category not found", body = ErrorDetail)
    )
)]
pub async fn delete_category(
    State(state): State<AppState>,
    _admin: AdminUser,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let cat = Category::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Category not found".to_string()))?;

    category::Entity::delete_by_id(cat.id).exec(&state.db).await?;

    Ok(StatusCode::NO_CONTENT)
}
