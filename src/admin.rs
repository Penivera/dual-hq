use std::path::PathBuf;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post},
    Json, Router,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use sea_orm_pro::{
    config::{ColumnCfg, JsonCfg, RawTableCfg, SiteCfg, TableCfg, ThemeCfg},
    ConfigParser,
};
use serde::{Deserialize, Serialize};
use tower_http::services::{ServeDir, ServeFile};

use crate::{
    db::AppState,
    entities::{
        application, opportunity,
        user::{self, UserRole},
        Application, Opportunity, User,
    },
    errors::AppError,
    handlers::auth::{create_jwt_token, verify_password},
    middleware::AuthenticatedUser,
};

/// Loads the SeaORM Pro configuration from the `pro_admin` directory,
/// falling back to programmatically generated config if files are missing.
pub fn load_admin_config() -> JsonCfg {
    // Try to find pro_admin folder
    let possible_paths = [
        "pro_admin",
        "internship-api/pro_admin",
        concat!(env!("CARGO_MANIFEST_DIR"), "/pro_admin"),
    ];

    for path in possible_paths {
        let p = std::path::Path::new(path);
        if p.exists() {
            match ConfigParser::new().load_config(path) {
                Ok(config) => {
                    tracing::info!("Loaded SeaORM Pro configuration from {}", path);
                    return config;
                }
                Err(err) => {
                    tracing::error!(
                        "Failed to deserialize SeaORM Pro configuration from {}: {:?}",
                        path,
                        err
                    );
                }
            }
        }
    }

    tracing::warn!("pro_admin folder not found or failed to deserialize, using default programmatic configuration");
    build_default_admin_config()
}

/// Fallback programmatic configuration for SeaORM Pro
pub fn build_default_admin_config() -> JsonCfg {
    let mut config = JsonCfg {
        site: SiteCfg {
            theme: ThemeCfg {
                title: "SeaORM Pro - Internship System".to_string(),
                logo: "https://www.sea-ql.org/favicon.ico".to_string(),
                login_banner: "https://www.sea-ql.org/img/SeaQL%20logo.png".to_string(),
            },
            menu: Default::default(),
        },
        dashboard: Default::default(),
        raw_tables: Default::default(),
        composite_tables: Default::default(),
    };

    // Users table
    let user_table = RawTableCfg {
        table: TableCfg {
            all_columns: true,
            title: Some("Users".to_string()),
            hidden_columns: vec!["hashed_password".to_string()],
            columns: vec![
                ColumnCfg {
                    title: Some("ID".to_string()),
                    field: "id".to_string(),
                    width: Some(80),
                    ..Default::default()
                },
                ColumnCfg {
                    title: Some("Full Name".to_string()),
                    field: "full_name".to_string(),
                    ..Default::default()
                },
                ColumnCfg {
                    title: Some("Email".to_string()),
                    field: "email".to_string(),
                    ..Default::default()
                },
                ColumnCfg {
                    title: Some("Role".to_string()),
                    field: "role".to_string(),
                    ..Default::default()
                },
                ColumnCfg {
                    title: Some("Created At".to_string()),
                    field: "created_at".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    };
    config.raw_tables.insert("users".to_string(), user_table);

    // Opportunities table
    let opp_table = RawTableCfg {
        table: TableCfg {
            all_columns: true,
            title: Some("Opportunities".to_string()),
            columns: vec![
                ColumnCfg {
                    title: Some("ID".to_string()),
                    field: "id".to_string(),
                    width: Some(80),
                    ..Default::default()
                },
                ColumnCfg {
                    title: Some("Title".to_string()),
                    field: "title".to_string(),
                    ..Default::default()
                },
                ColumnCfg {
                    title: Some("Company".to_string()),
                    field: "company".to_string(),
                    ..Default::default()
                },
                ColumnCfg {
                    title: Some("Location".to_string()),
                    field: "location".to_string(),
                    ..Default::default()
                },
                ColumnCfg {
                    title: Some("Type".to_string()),
                    field: "type".to_string(),
                    ..Default::default()
                },
                ColumnCfg {
                    title: Some("Status".to_string()),
                    field: "status".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    };
    config
        .raw_tables
        .insert("opportunities".to_string(), opp_table);

    // Applications table
    let app_table = RawTableCfg {
        table: TableCfg {
            all_columns: true,
            title: Some("Applications".to_string()),
            columns: vec![
                ColumnCfg {
                    title: Some("ID".to_string()),
                    field: "id".to_string(),
                    width: Some(80),
                    ..Default::default()
                },
                ColumnCfg {
                    title: Some("User ID".to_string()),
                    field: "user_id".to_string(),
                    ..Default::default()
                },
                ColumnCfg {
                    title: Some("Opportunity ID".to_string()),
                    field: "opportunity_id".to_string(),
                    ..Default::default()
                },
                ColumnCfg {
                    title: Some("Status".to_string()),
                    field: "status".to_string(),
                    ..Default::default()
                },
                ColumnCfg {
                    title: Some("Applied At".to_string()),
                    field: "applied_at".to_string(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        },
        ..Default::default()
    };
    config
        .raw_tables
        .insert("applications".to_string(), app_table);

    config
}

#[derive(Serialize)]
pub struct AdminStats {
    pub users_count: u64,
    pub opportunities_count: u64,
    pub applications_count: u64,
}

#[derive(Deserialize)]
pub struct UpdateRoleRequest {
    pub role: UserRole,
}

#[derive(Deserialize)]
pub struct AdminLoginRequest {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: String,
}

#[derive(Serialize)]
pub struct AdminLoginResponse {
    pub status: String,
    #[serde(rename = "type")]
    pub login_type: String,
    #[serde(rename = "currentAuthority")]
    pub current_authority: String,
    pub token: String,
}

#[derive(Serialize)]
pub struct CurrentUserData {
    pub name: String,
    pub avatar: String,
    pub userid: String,
    pub email: String,
    pub access: String,
}

#[derive(Serialize)]
pub struct CurrentUserResponse {
    pub success: bool,
    pub data: CurrentUserData,
}

static ADMIN_CONFIG: std::sync::OnceLock<JsonCfg> = std::sync::OnceLock::new();

pub fn get_cached_admin_config() -> &'static JsonCfg {
    ADMIN_CONFIG.get_or_init(load_admin_config)
}

async fn admin_stats(State(state): State<AppState>) -> Result<Json<AdminStats>, AppError> {
    let (users_count, opportunities_count, applications_count) = tokio::try_join!(
        User::find().count(&state.db),
        Opportunity::find().count(&state.db),
        Application::find().count(&state.db)
    )?;

    Ok(Json(AdminStats {
        users_count,
        opportunities_count,
        applications_count,
    }))
}

pub async fn get_admin_config() -> Json<&'static JsonCfg> {
    Json(get_cached_admin_config())
}

pub async fn admin_dashboard() -> Json<serde_json::Value> {
    Json(serde_json::json!([]))
}

pub async fn admin_graphql(
    State(state): State<AppState>,
    Json(mut body): Json<serde_json::Value>,
) -> Json<async_graphql::Response> {
    if body.get("variables").is_none() {
        if let Some(var) = body.get_mut("variable") {
            let var = var.take();
            body["variables"] = var;
        }
    }
    let req: async_graphql::Request = match serde_json::from_value(body) {
        Ok(r) => r,
        Err(err) => {
            tracing::error!("Failed to parse GraphQL request: {err}");
            return Json(async_graphql::Response::from_errors(vec![
                async_graphql::ServerError::new(err.to_string(), None),
            ]));
        }
    };
    let schema = crate::graphql::get_admin_schema();
    let req = req.data(state.db.clone()).data(state.config.clone());
    let res = schema.execute(req).await;
    Json(res)
}

pub async fn admin_account_login(
    State(state): State<AppState>,
    Json(payload): Json<AdminLoginRequest>,
) -> Result<Json<AdminLoginResponse>, AppError> {
    let email = payload
        .username
        .as_deref()
        .or(payload.email.as_deref())
        .unwrap_or("");

    let user = User::find()
        .filter(user::Column::Email.eq(email))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid credentials".to_string()))?;

    if !verify_password(&payload.password, &user.hashed_password) {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let role_str = match user.role {
        UserRole::Admin => "admin",
        UserRole::Applicant => "applicant",
    };

    let token = create_jwt_token(
        user.id,
        role_str,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    Ok(Json(AdminLoginResponse {
        status: "ok".to_string(),
        login_type: "account".to_string(),
        current_authority: role_str.to_string(),
        token,
    }))
}

pub async fn current_user_info(
    State(state): State<AppState>,
    user: AuthenticatedUser,
) -> Result<Json<CurrentUserResponse>, AppError> {
    let db_user = User::find_by_id(user.id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let role_str = match db_user.role {
        UserRole::Admin => "admin",
        UserRole::Applicant => "user",
    };

    Ok(Json(CurrentUserResponse {
        success: true,
        data: CurrentUserData {
            name: db_user.full_name,
            avatar: "https://gw.alipayobjects.com/zos/antfincdn/XAosXuNZyF/BiazfanxmamNRoxxVxka.png".to_string(),
            userid: db_user.id.to_string(),
            email: db_user.email,
            access: role_str.to_string(),
        },
    }))
}

pub async fn admin_logout() -> impl IntoResponse {
    Json(serde_json::json!({
        "data": {},
        "success": true
    }))
}

pub async fn admin_notices() -> impl IntoResponse {
    Json(serde_json::json!({
        "data": [],
        "success": true
    }))
}

async fn list_admin_users(State(state): State<AppState>) -> Result<Json<Vec<user::Model>>, AppError> {
    let users = User::find()
        .order_by_asc(user::Column::Id)
        .all(&state.db)
        .await?;
    Ok(Json(users))
}

async fn update_user_role(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<Json<user::Model>, AppError> {
    let user = User::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let mut active: user::ActiveModel = user.into();
    active.role = Set(payload.role);
    let updated = active.update(&state.db).await?;
    Ok(Json(updated))
}

async fn list_admin_opportunities(
    State(state): State<AppState>,
) -> Result<Json<Vec<opportunity::Model>>, AppError> {
    let opps = Opportunity::find()
        .order_by_desc(opportunity::Column::Id)
        .all(&state.db)
        .await?;
    Ok(Json(opps))
}

async fn delete_admin_opportunity(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let opp = Opportunity::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Opportunity not found".to_string()))?;

    Application::delete_many()
        .filter(application::Column::OpportunityId.eq(id))
        .exec(&state.db)
        .await?;

    opp.delete(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_admin_applications(
    State(state): State<AppState>,
) -> Result<Json<Vec<application::Model>>, AppError> {
    let apps = Application::find()
        .order_by_desc(application::Column::Id)
        .all(&state.db)
        .await?;
    Ok(Json(apps))
}

async fn delete_admin_application(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    let app = Application::find_by_id(id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("Application not found".to_string()))?;

    app.delete(&state.db).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Locates the `assets/admin` directory across standard locations
pub fn find_assets_dir() -> PathBuf {
    let candidates = [
        PathBuf::from("assets/admin"),
        PathBuf::from("internship-api/assets/admin"),
        PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/admin")),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    PathBuf::from("assets/admin")
}

/// Creates the SeaORM Pro admin router serving static assets and API routes
pub fn create_admin_router() -> Router<AppState> {
    let assets_dir = find_assets_dir();
    let index_file = assets_dir.join("index.html");
    let favicon_file = assets_dir.join("favicon.ico");

    Router::new()
        // Serve index.html on root /admin and /admin/
        .route_service("/", ServeFile::new(index_file.clone()))
        // Config endpoint called by SeaORM Pro frontend
        .route("/config", get(get_admin_config))
        .route_service("/favicon.ico", ServeFile::new(favicon_file))
        // Stats & management APIs for entities
        .route("/api/stats", get(admin_stats))
        .route("/api/users", get(list_admin_users))
        .route("/api/users/{id}/role", post(update_user_role))
        .route("/api/opportunities", get(list_admin_opportunities))
        .route("/api/opportunities/{id}", delete(delete_admin_opportunity))
        .route("/api/applications", get(list_admin_applications))
        .route("/api/applications/{id}", delete(delete_admin_application))
        // Serve SeaORM Pro frontend assets with SPA fallback to index.html
        .fallback_service(
            ServeDir::new(assets_dir).fallback(ServeFile::new(index_file)),
        )
}
