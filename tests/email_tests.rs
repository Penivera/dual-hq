use internship_api::{
    config::Config,
    email::send_email_smtp,
    routes::ApiDoc,
    schemas::auth::{UserResponse, VerificationResponse, VerifyEmailQuery},
};
use utoipa::OpenApi;

#[tokio::test]
async fn test_send_email_smtp_disabled_noop() {
    let mut config = Config::builder()
        .build()
        .unwrap()
        .try_deserialize::<Config>()
        .unwrap();
    config.smtp_enabled = false;

    let res = send_email_smtp(
        &config,
        "test@example.com",
        Some("Test User"),
        "Test Subject",
        "<p>Hello world</p>",
    )
    .await;

    assert!(res.is_ok(), "When SMTP is disabled, send_email_smtp should succeed without sending");
}

#[test]
fn test_verification_schemas() {
    let query = VerifyEmailQuery {
        token: "abc123token".to_string(),
    };
    assert_eq!(query.token, "abc123token");

    let response = VerificationResponse {
        message: "Email verified successfully".to_string(),
        is_verified: true,
    };
    assert!(response.is_verified);
    assert_eq!(response.message, "Email verified successfully");

    let user_resp = UserResponse {
        id: 1,
        full_name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        role: internship_api::entities::user::UserRole::Applicant,
        is_verified: false,
        created_at: chrono::Utc::now().into(),
    };
    assert!(!user_resp.is_verified);
}

#[test]
fn test_openapi_includes_email_verification_endpoints() {
    let openapi = ApiDoc::openapi();
    let paths: Vec<String> = openapi.paths.paths.keys().cloned().collect();

    assert!(paths.contains(&"/auth/verify".to_string()));
    assert!(paths.contains(&"/auth/resend-verification".to_string()));
}
