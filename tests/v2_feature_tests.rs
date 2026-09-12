use askama::Template;
use internship_api::{
    email::{
        RecruiterApprovedTemplate, RecruiterPendingTemplate, UserSuspendedTemplate,
        UserUnsuspendedTemplate,
    },
    entities::user::{UserRole, UserStatus},
    routes::ApiDoc,
    schemas::{
        auth::Token,
        category::{CategoryCreate, CategoryResponse},
        notification::NotificationResponse,
        opportunity::{OpportunityResponse, PaginationQuery},
        profile::{ProfileResponse, ProfileUpdate},
    },
};
use sha2::{Digest, Sha256};
use utoipa::OpenApi;

#[test]
fn test_user_role_and_status_enums() {
    assert_eq!(serde_json::to_string(&UserRole::Recruiter).unwrap(), "\"recruiter\"");
    assert_eq!(serde_json::to_string(&UserRole::Applicant).unwrap(), "\"applicant\"");
    assert_eq!(serde_json::to_string(&UserRole::Admin).unwrap(), "\"admin\"");

    assert_eq!(serde_json::to_string(&UserStatus::Pending).unwrap(), "\"pending\"");
    assert_eq!(serde_json::to_string(&UserStatus::Active).unwrap(), "\"active\"");
    assert_eq!(serde_json::to_string(&UserStatus::Suspended).unwrap(), "\"suspended\"");
}

#[test]
fn test_auth_token_pair_schema() {
    let token = Token::new(
        "sample_access_jwt".to_string(),
        "sample_refresh_raw".to_string(),
        900,
    );
    assert_eq!(token.access_token, "sample_access_jwt");
    assert_eq!(token.refresh_token, "sample_refresh_raw");
    assert_eq!(token.expires_in, 900);
    assert_eq!(token.token_type, "bearer");
}

#[test]
fn test_refresh_token_sha256_hashing() {
    let raw = "test_refresh_token_12345";
    let hashed = format!("{:x}", Sha256::digest(raw.as_bytes()));
    assert_eq!(hashed.len(), 64);
    // Deterministic hash check
    let hashed_again = format!("{:x}", Sha256::digest(raw.as_bytes()));
    assert_eq!(hashed, hashed_again);
}

#[test]
fn test_category_schema_and_slug() {
    let create = CategoryCreate {
        name: "DevOps & Cloud".to_string(),
        slug: None,
    };
    assert_eq!(create.name, "DevOps & Cloud");

    let resp = CategoryResponse {
        id: 10,
        name: "Data Science".to_string(),
        slug: "data-science".to_string(),
        created_at: chrono::Utc::now().fixed_offset(),
    };
    assert_eq!(resp.slug, "data-science");
}

#[test]
fn test_opportunity_enhancement_schemas() {
    let now = chrono::Utc::now().fixed_offset();
    let opp = OpportunityResponse {
        id: 1,
        title: "Rust Systems Engineer Intern".to_string(),
        description: "Develop distributed backend microservices".to_string(),
        company: "RustCorp".to_string(),
        location: "Remote".to_string(),
        type_: internship_api::entities::opportunity::OpportunityType::Internship,
        status: internship_api::entities::opportunity::OpportunityStatus::Open,
        created_by: Some(42),
        category_id: Some(5),
        category_name: Some("Engineering".to_string()),
        deadline: Some(now),
        application_count: Some(15),
        created_at: now,
        updated_at: now,
    };

    let json = serde_json::to_value(&opp).unwrap();
    assert_eq!(json["created_by"], 42);
    assert_eq!(json["category_name"], "Engineering");
    assert_eq!(json["application_count"], 15);

    // When application_count is None, it must be omitted from serialized JSON
    let mut opp_public = opp;
    opp_public.application_count = None;
    let json_public = serde_json::to_value(&opp_public).unwrap();
    assert!(json_public.get("application_count").is_none());
}

#[test]
fn test_composable_pagination_query_params() {
    let query_str = "search=Rust&type=internship&location=Remote&category=backend&page=1&per_page=10";
    let parsed: PaginationQuery = serde_urlencoded::from_str(query_str).unwrap();
    assert_eq!(parsed.search.as_deref(), Some("Rust"));
    assert_eq!(parsed.location.as_deref(), Some("Remote"));
    assert_eq!(parsed.category.as_deref(), Some("backend"));
    assert_eq!(parsed.page, Some(1));
    assert_eq!(parsed.per_page, Some(10));
}

#[test]
fn test_profile_schemas() {
    let update = ProfileUpdate {
        bio: Some("Passionate backend developer with experience in Rust and PostgreSQL.".to_string()),
        cv_url: Some("https://example.com/cv/applicant.pdf".to_string()),
    };
    assert!(update.cv_url.is_some());

    let resp = ProfileResponse {
        id: 1,
        user_id: 10,
        bio: update.bio.clone(),
        cv_url: update.cv_url.clone(),
        updated_at: chrono::Utc::now().fixed_offset(),
    };
    assert_eq!(resp.user_id, 10);
    assert_eq!(resp.cv_url.as_deref(), Some("https://example.com/cv/applicant.pdf"));
}

#[test]
fn test_notification_schema() {
    let notif = NotificationResponse {
        id: 1,
        user_id: 5,
        title: "Application Received".to_string(),
        body: "A new candidate applied to your listing.".to_string(),
        read: false,
        created_at: chrono::Utc::now().fixed_offset(),
    };
    assert!(!notif.read);
    assert_eq!(notif.title, "Application Received");
}

#[test]
fn test_v2_openapi_specification_coverage() {
    let openapi = ApiDoc::openapi();
    let paths: Vec<String> = openapi.paths.paths.keys().cloned().collect();

    // Check v2 endpoints in OpenAPI
    assert!(paths.contains(&"/auth/refresh".to_string()));
    assert!(paths.contains(&"/auth/logout".to_string()));
    assert!(paths.contains(&"/categories".to_string()));
    assert!(paths.contains(&"/categories/{id}".to_string()));
    assert!(paths.contains(&"/profile/me".to_string()));
    assert!(paths.contains(&"/profile/{user_id}".to_string()));
    assert!(paths.contains(&"/notifications".to_string()));
    assert!(paths.contains(&"/notifications/{id}/read".to_string()));
    assert!(paths.contains(&"/notifications/read-all".to_string()));
    assert!(paths.contains(&"/admin/recruiters/pending".to_string()));
    assert!(paths.contains(&"/admin/recruiters/{id}/approve".to_string()));
    assert!(paths.contains(&"/admin/users/{id}/suspend".to_string()));
    assert!(paths.contains(&"/admin/users/{id}/unsuspend".to_string()));
}

#[test]
fn test_askama_v2_templates_rendering() {
    // 1. Recruiter pending
    let t1 = RecruiterPendingTemplate {
        user_name: "Alex Recruiter",
    };
    let html1 = t1.render().expect("recruiter pending template must render");
    assert!(html1.contains("Alex Recruiter"));
    assert!(html1.contains("under administrative review"));

    // 2. Recruiter approved
    let t2 = RecruiterApprovedTemplate {
        user_name: "Alex Recruiter",
        login_url: "https://portal.example.com/login",
    };
    let html2 = t2.render().expect("recruiter approved template must render");
    assert!(html2.contains("Alex Recruiter"));
    assert!(html2.contains("https://portal.example.com/login"));

    // 3. User suspended
    let t3 = UserSuspendedTemplate {
        user_name: "Bad Actor",
        reason: "Spam applications",
    };
    let html3 = t3.render().expect("user suspended template must render");
    assert!(html3.contains("Bad Actor"));
    assert!(html3.contains("Spam applications"));

    // 4. User unsuspended
    let t4 = UserUnsuspendedTemplate {
        user_name: "Good Actor",
        login_url: "https://portal.example.com/login",
    };
    let html4 = t4.render().expect("user unsuspended template must render");
    assert!(html4.contains("Good Actor"));
    assert!(html4.contains("reactivated"));
}
