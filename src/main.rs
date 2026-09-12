use std::sync::Arc;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use internship_api::{
    admin,
    config::Config,
    db::{init_db, AppState},
    routes::{self, ApiDoc},
};
use std::time::Duration;
use serde_json::json;
use tower_http::{
    compression::CompressionLayer,
    cors::CorsLayer,
    services::ServeFile,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

async fn fallback_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "detail": "Not found" })),
    )
}

async fn health_handler(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl IntoResponse {
    let db_healthy = state.db.ping().await.is_ok();
    let status_code = if db_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status_code,
        Json(json!({
            "status": if db_healthy { "healthy" } else { "degraded" },
            "database": if db_healthy { "connected" } else { "disconnected" },
            "version": env!("CARGO_PKG_VERSION")
        })),
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "internship_api=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::load()?;
    tracing::info!("Starting Internship Application API on {}:{}", config.server_host, config.server_port);

    // Initialize Database and run migrations
    let db = match init_db(&config).await {
        Ok(db) => db,
        Err(err) => {
            tracing::error!("Failed to connect to database or run migrations: {err}");
            return Err(err.into());
        }
    };

    let app_state = AppState {
        db,
        config: Arc::new(config.clone()),
    };

    // Swagger UI mounted at /docs
    let openapi = ApiDoc::openapi();
    let swagger_router = SwaggerUi::new("/docs").url("/api-docs/openapi.json", openapi);

    // Build overall application
    let app = Router::new()
        // Mount Swagger UI
        .merge(swagger_router)
        // Root redirect
        .route(
            "/",
            get(|| async {
                Json(json!({
                    "name": "Internship Application System API",
                    "version": "1.0.0",
                    "docs": "/docs",
                    "admin": "/admin"
                }))
            }),
        )
        // Health check endpoints
        .route("/health", get(health_handler))
        .route("/api/health", get(health_handler))
        // Serve favicon.ico at root
        .route_service(
            "/favicon.ico",
            ServeFile::new(admin::find_assets_dir().join("favicon.ico")),
        )
        // Mount API routers (/auth, /opportunities, /applications)
        .merge(routes::create_router())
        // SeaORM Pro frontend authentication & profile endpoints
        .route("/api/auth/login", axum::routing::post(admin::admin_account_login))
        .route("/api/auth/email-login", axum::routing::post(admin::admin_account_login))
        .route("/api/login/account", axum::routing::post(admin::admin_account_login))
        .route("/api/login/outLogin", axum::routing::post(admin::admin_logout))
        .route("/api/user/current", get(admin::current_user_info))
        .route("/api/currentUser", get(admin::current_user_info))
        .route("/api/notices", get(admin::admin_notices))
        .route("/api/config", get(admin::get_admin_config))
        .route("/api/admin/config", get(admin::get_admin_config))
        .route(
            "/api/admin/dashboard",
            axum::routing::post(admin::admin_dashboard).get(admin::admin_dashboard),
        )
        .route("/api/graphql", axum::routing::post(admin::admin_graphql))
        // Mount SeaORM Pro Admin panel at /admin
        .nest("/admin", admin::create_admin_router())
        .route_service(
            "/admin/",
            ServeFile::new(admin::find_assets_dir().join("index.html")),
        )
        // Global 404 fallback with standard {"detail": "..."} shape
        .fallback(fallback_handler)
        // Shared state
        .with_state(app_state)
        // Middleware layers
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TimeoutLayer::with_status_code(StatusCode::REQUEST_TIMEOUT, Duration::from_secs(30)))
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", config.server_host, config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("Server listening on http://{}", addr);
    tracing::info!("Swagger UI available at http://{}/docs", addr);
    tracing::info!("Admin panel available at http://{}/admin", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
