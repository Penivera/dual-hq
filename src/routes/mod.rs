pub mod applications;
pub mod auth;
pub mod categories;
pub mod notifications;
pub mod opportunities;
pub mod profile;

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
        user::{UserRole, UserStatus},
    },
    errors::ErrorDetail,
    handlers::{
        admin as admin_handlers, applications as app_handlers, auth as auth_handlers,
        categories as cat_handlers, notifications as notif_handlers,
        opportunities as opp_handlers, profile as profile_handlers,
    },
    schemas::{
        application::{
            ApplicationCreate, ApplicationDetailResponse, ApplicationResponse,
            ApplicationStatusUpdate,
        },
        auth::{
            LoginRequest, RefreshTokenRequest, ResendVerificationRequest, Token, UserCreate,
            UserResponse, VerificationResponse, VerifyEmailQuery,
        },
        category::{CategoryCreate, CategoryResponse},
        notification::NotificationResponse,
        opportunity::{
            OpportunityCreate, OpportunityFullUpdate, OpportunityResponse, OpportunityUpdate,
        },
        profile::{ProfileResponse, ProfileUpdate},
    },
};

#[derive(OpenApi)]
#[openapi(
    paths(
        auth_handlers::register,
        auth_handlers::login,
        auth_handlers::refresh_token,
        auth_handlers::logout,
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
        cat_handlers::list_categories,
        cat_handlers::create_category,
        cat_handlers::delete_category,
        profile_handlers::get_my_profile,
        profile_handlers::update_my_profile,
        profile_handlers::get_profile_by_user_id,
        notif_handlers::list_notifications,
        notif_handlers::mark_notification_read,
        notif_handlers::mark_all_notifications_read,
        admin_handlers::list_pending_recruiters,
        admin_handlers::approve_recruiter,
        admin_handlers::suspend_user,
        admin_handlers::unsuspend_user,
    ),
    components(
        schemas(
            UserCreate,
            UserResponse,
            VerificationResponse,
            VerifyEmailQuery,
            ResendVerificationRequest,
            LoginRequest,
            RefreshTokenRequest,
            Token,
            UserRole,
            UserStatus,
            OpportunityCreate,
            OpportunityUpdate,
            OpportunityFullUpdate,
            OpportunityResponse,
            OpportunityType,
            OpportunityStatus,
            CategoryCreate,
            CategoryResponse,
            ProfileUpdate,
            ProfileResponse,
            NotificationResponse,
            admin_handlers::SuspendUserRequest,
            ApplicationCreate,
            ApplicationStatusUpdate,
            ApplicationResponse,
            ApplicationDetailResponse,
            ApplicationStatus,
            ErrorDetail,
        )
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "Auth", description = "Authentication, registration, token refresh and session lifecycle"),
        (name = "Opportunities", description = "Opportunity listings, discovery, search and lifecycle"),
        (name = "Applications", description = "Application tracking, submission and status management"),
        (name = "Categories", description = "Opportunity categorization endpoints"),
        (name = "Profile", description = "Applicant profile management"),
        (name = "Notifications", description = "User in-app notifications"),
        (name = "Admin", description = "Administrative user lifecycle and recruiter approval")
    ),
    info(
        title = "Internship Application System",
        version = "1.0.0",
        description = "REST API for managing internship opportunities, applications, user profiles, and hiring workflows."
    )
)]
pub struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            let scheme = SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("JWT")
                    .description(Some("JWT access token in Bearer format"))
                    .build(),
            );
            components.add_security_scheme("bearer_auth", scheme.clone());
            components.add_security_scheme("bearerAuth", scheme);
        }
    }
}

pub fn create_router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/opportunities", opportunities::router())
        .nest("/applications", applications::router())
        .nest("/categories", categories::router())
        .nest("/profile", profile::router())
        .nest("/notifications", notifications::router())
}
