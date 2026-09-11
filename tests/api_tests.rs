use internship_api::{
    admin::load_admin_config,
    errors::AppError,
    handlers::auth::{create_jwt_token, hash_password, verify_password},
    routes::ApiDoc,
    schemas::auth::Claims,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use utoipa::OpenApi;

#[test]
fn test_password_hashing_and_verification() {
    let password = "SecretPassword123!";
    let hash = hash_password(password).expect("hashing should succeed");

    assert!(verify_password(password, &hash));
    assert!(!verify_password("wrong_password", &hash));
}

#[test]
fn test_jwt_token_generation_and_validation() {
    let secret = "test-secret-key-12345678901234567890";
    let token = create_jwt_token(42, "admin", secret, 24).expect("token creation should succeed");

    let mut validation = Validation::default();
    validation.validate_exp = true;

    let decoded = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .expect("token should be valid");

    assert_eq!(decoded.claims.sub, "42");
    assert_eq!(decoded.claims.role, "admin");
}

#[test]
fn test_openapi_spec_generation() {
    let openapi = ApiDoc::openapi();
    assert_eq!(openapi.info.title, "Internship Application System");

    // Check all required paths exist in generated OpenAPI
    let paths: Vec<String> = openapi.paths.paths.keys().cloned().collect();
    assert!(paths.contains(&"/auth/register".to_string()));
    assert!(paths.contains(&"/auth/login".to_string()));
    assert!(paths.contains(&"/opportunities".to_string()));
    assert!(paths.contains(&"/opportunities/{id}".to_string()));
    assert!(paths.contains(&"/applications".to_string()));
    assert!(paths.contains(&"/applications/me".to_string()));
    assert!(paths.contains(&"/applications/{id}".to_string()));
    assert!(paths.contains(&"/applications/{id}/status".to_string()));

    // Check bearerAuth security scheme is configured
    assert!(openapi.components.is_some());
    let components = openapi.components.unwrap();
    assert!(components.security_schemes.contains_key("bearerAuth"));
}

#[test]
fn test_sea_orm_pro_admin_config() {
    let parser_res = sea_orm_pro::ConfigParser::new().load_config("pro_admin");
    println!("ConfigParser result: {:?}", parser_res);
    let config = load_admin_config();
    assert!(config.site.theme.title.contains("SeaORM Pro"));
    assert!(config.raw_tables.contains_key("users"));
    assert!(config.raw_tables.contains_key("opportunities"));
    assert!(config.raw_tables.contains_key("applications"));
}

#[test]
fn test_error_shapes() {
    let err = AppError::NotFound("Opportunity not found".to_string());
    assert_eq!(err.to_string(), "Opportunity not found");

    let err2 = AppError::Unauthorized("Could not validate credentials".to_string());
    assert_eq!(err2.to_string(), "Could not validate credentials");
}

#[test]
fn test_routes_router_creation() {
    // Verifies that Axum route definitions and nesting do not panic
    let router = internship_api::routes::create_router();
    let _admin_router = internship_api::admin::create_admin_router();
    drop(router);
}

#[tokio::test]
async fn test_admin_graphql_schema_metadata_and_introspection() {
    let schema = internship_api::graphql::get_admin_schema();

    // Test _sea_orm_entity_metadata query
    let res = schema
        .execute("query { _sea_orm_entity_metadata(table_name: \"users\") { primary_key columns { name nullable } } }")
        .await;
    assert!(res.errors.is_empty(), "GraphQL errors: {:?}", res.errors);
    let json_data = serde_json::to_value(&res.data).unwrap();
    let meta = &json_data["_sea_orm_entity_metadata"];
    assert_eq!(meta["primary_key"], serde_json::json!(["id"]));
    let cols = meta["columns"].as_array().unwrap();
    assert!(cols.iter().any(|c| c["name"] == "id"));
    assert!(cols.iter().any(|c| c["name"] == "email"));
    assert!(cols.iter().any(|c| c["name"] == "role"));

    // Test introspection query schema has queryType and users/opportunities/applications fields
    let res = schema
        .execute("query { __schema { queryType { fields { name } } } }")
        .await;
    assert!(res.errors.is_empty(), "Introspection errors: {:?}", res.errors);
    let json_data = serde_json::to_value(&res.data).unwrap();
    let fields = json_data["__schema"]["queryType"]["fields"].as_array().unwrap();
    assert!(fields.iter().any(|f| f["name"] == "users"));
    assert!(fields.iter().any(|f| f["name"] == "opportunities"));
    assert!(fields.iter().any(|f| f["name"] == "applications"));
}

