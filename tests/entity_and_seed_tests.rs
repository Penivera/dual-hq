use internship_api::{
    config::Config,
    entities::{Application, Opportunity, User},
};
use sea_orm::{DbBackend, EntityName, Schema};

#[test]
fn test_entities_table_names() {
    assert_eq!(User.table_name(), "users");
    assert_eq!(Opportunity.table_name(), "opportunities");
    assert_eq!(Application.table_name(), "applications");
}

#[test]
fn test_entity_first_schema_generation() {
    let schema = Schema::new(DbBackend::Postgres);

    // Verify entity-first table creation statements can be generated
    let user_table = schema.create_table_from_entity(User);
    let opp_table = schema.create_table_from_entity(Opportunity);
    let app_table = schema.create_table_from_entity(Application);

    let user_sql = DbBackend::Postgres.build(&user_table).to_string();
    let opp_sql = DbBackend::Postgres.build(&opp_table).to_string();
    let app_sql = DbBackend::Postgres.build(&app_table).to_string();

    assert!(user_sql.contains("CREATE TABLE \"users\""));
    assert!(user_sql.contains("\"email\""));
    assert!(opp_sql.contains("CREATE TABLE \"opportunities\""));
    assert!(app_sql.contains("CREATE TABLE \"applications\""));

    // Verify entity-first enum creation
    let user_enums = schema.create_enum_from_entity(User);
    assert!(!user_enums.is_empty());

    // Verify entity-first indexes creation
    let app_indexes = schema.create_index_from_entity(Application);
    assert!(!app_indexes.is_empty());
}

#[test]
fn test_admin_config_defaults() {
    let builder = Config::builder();
    let config: Config = builder.build().unwrap().try_deserialize().unwrap();

    assert_eq!(config.admin_email, "admin@internship.local");
    assert_eq!(config.admin_password, "admin");
    assert_eq!(config.admin_name, "Admin User");
}
