pub mod applications;
pub mod auth;
pub mod opportunities;

use axum::Router;
use utoipa::{
    openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme},
    Modify, OpenApi,
};

use crate::{
    db::AppState,
    entities::{
        application::ApplicationStatus,
        opportunity::{OpportunityStatus, OpportunityType},
        user::UserRole,
    },
    errors::ErrorDetail,
    handlers::{applications as app_handlers, auth as auth_handlers, opportunities as opp_handlers},
    schemas::{
        application::{ApplicationCreate, ApplicationResponse, ApplicationStatusUpdate},
        auth::{LoginRequest, Token, UserCreate, UserResponse},
        opportunity::{OpportunityCreate, OpportunityResponse, OpportunityUpdate},
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth_handlers::register,
        auth_handlers::login,
        auth_handlers::verify_email,
        auth_handlers::resend_verification,
        opp_handlers::list_opportunities,
        opp_handlers::get_opportunity,
        opp_handlers::create_opportunity,
        opp_handlers::update_opportunity,
        opp_handlers::delete_opportunity,
        app_handlers::apply,
        app_handlers::list_applications,
        app_handlers::my_applications,
        app_handlers::get_application,
        app_handlers::update_application_status,
        app_handlers::delete_application,
    ),
    components(
        schemas(
            UserCreate,
            UserResponse,
            crate::schemas::auth::VerificationResponse,
            LoginRequest,
            Token,
            UserRole,
            OpportunityCreate,
            OpportunityUpdate,
            crate::schemas::opportunity::OpportunityFullUpdate,
            OpportunityResponse,
            OpportunityType,
            OpportunityStatus,
            ApplicationCreate,
            ApplicationStatusUpdate,
            ApplicationResponse,
            crate::schemas::application::ApplicationDetailResponse,
            ApplicationStatus,
            ErrorDetail,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Authentication and registration endpoints"),
        (name = "Opportunities", description = "Opportunity management endpoints"),
        (name = "Applications", description = "Application tracking and submission endpoints")
    ),
    info(
        title = "Internship Application System",
        version = "1.0.0",
        description = "REST API for managing internship/job opportunities and applications, built with Axum, SeaORM, and PostgreSQL."
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "bearerAuth",
                SecurityScheme::Http(
                    HttpBuilder::new()
                        .scheme(HttpAuthScheme::Bearer)
                        .bearer_format("JWT")
                        .description(Some("JWT access token in Bearer format"))
                        .build(),
                ),
            );
        }
    }
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/opportunities", opportunities::router())
        .nest("/applications", applications::router())
}
