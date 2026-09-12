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
        status: internship_api::entities::user::UserStatus::Active,
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

#[test]
fn test_askama_verification_template_rendering() {
    use askama::Template;
    use internship_api::email::VerificationTemplate;

    let tmpl = VerificationTemplate {
        user_name: "Jane Doe",
        verify_url: "https://example.com/api/auth/verify?token=tok123",
        verification_token: "tok123",
    };
    let html = tmpl.render().expect("Should render verification template");
    assert!(html.contains("Jane Doe"));
    assert!(html.contains("https://example.com/api/auth/verify?token=tok123"));
    assert!(html.contains("tok123"));
    assert!(html.contains("Internship Portal"));
}

#[test]
fn test_askama_welcome_template_rendering() {
    use askama::Template;
    use internship_api::email::WelcomeTemplate;

    let tmpl = WelcomeTemplate {
        user_name: "John Smith",
        dashboard_url: "https://example.com/dashboard",
    };
    let html = tmpl.render().expect("Should render welcome template");
    assert!(html.contains("John Smith"));
    assert!(html.contains("Your Account is Verified"));
    assert!(html.contains("https://example.com/dashboard"));
}

#[test]
fn test_askama_new_opportunity_template_rendering() {
    use askama::Template;
    use internship_api::email::NewOpportunityTemplate;

    let tmpl = NewOpportunityTemplate {
        student_name: "Alex",
        title: "Rust Backend Engineer",
        company_name: "Dual HQ",
        location: "Remote",
        opp_type: "Internship",
        stipend: "$3000/mo",
        view_url: "https://example.com/opportunities/42",
    };
    let html = tmpl.render().expect("Should render new opportunity template");
    assert!(html.contains("Alex"));
    assert!(html.contains("Rust Backend Engineer"));
    assert!(html.contains("Dual HQ"));
    assert!(html.contains("Remote"));
    assert!(html.contains("$3000/mo"));
    assert!(html.contains("https://example.com/opportunities/42"));
}

#[test]
fn test_askama_application_accepted_template_rendering() {
    use askama::Template;
    use internship_api::email::ApplicationAcceptedTemplate;

    let tmpl = ApplicationAcceptedTemplate {
        applicant_name: "Sarah Connor",
        opportunity_title: "Systems Architect Intern",
        company_name: "Tech Corp",
        next_steps: "Complete onboarding forms by Friday.",
        dashboard_url: "https://example.com/dashboard/applications",
    };
    let html = tmpl.render().expect("Should render application accepted template");
    assert!(html.contains("Sarah Connor"));
    assert!(html.contains("Systems Architect Intern"));
    assert!(html.contains("Tech Corp"));
    assert!(html.contains("Complete onboarding forms by Friday."));
    assert!(html.contains("ACCEPTED"));
}
