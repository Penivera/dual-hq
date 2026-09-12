use sea_orm::{ActiveModelTrait, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter, Set};

use crate::{
    config::Config,
    entities::{
        user::{self, UserRole},
        User,
    },
    errors::AppError,
    handlers::auth::hash_password,
};

/// Ensures that an administrator exists in the database.
///
/// 1. If any user with `role = admin` exists, logs and returns it.
/// 2. If the user with `config.admin_email` exists but is not an admin, promotes them.
/// 3. Otherwise, creates a new administrator with the configured credentials.
pub async fn seed_initial_admin(
    db: &DatabaseConnection,
    config: &Config,
) -> Result<user::Model, AppError> {
    ensure_admin(
        db,
        &config.admin_email,
        &config.admin_password,
        &config.admin_name,
    )
    .await
}

/// Ensures an admin with the specified email, password, and name exists.
pub async fn ensure_admin(
    db: &DatabaseConnection,
    email: &str,
    password: &str,
    name: &str,
) -> Result<user::Model, AppError> {
    // 1. Check if ANY admin already exists in the system
    let existing_admin = User::find()
        .filter(user::Column::Role.eq(UserRole::Admin))
        .one(db)
        .await?;

    if let Some(admin) = existing_admin {
        tracing::info!(
            "Admin user '{}' (ID: {}) already exists. System has active administrator.",
            admin.email,
            admin.id
        );
        return Ok(admin);
    }

    // 2. Check if a user with the specified email already exists
    let existing_user = User::find()
        .filter(user::Column::Email.eq(email))
        .one(db)
        .await?;

    if let Some(user) = existing_user {
        tracing::info!(
            "Promoting existing user '{}' (ID: {}) to administrator",
            user.email,
            user.id
        );
        let mut active: user::ActiveModel = user.into();
        active.role = Set(UserRole::Admin);
        active.is_verified = Set(true);
        let updated = active.update(db).await?;
        return Ok(updated);
    }

    // 3. Create brand new administrator
    let hashed_password = hash_password(password)?;
    let new_admin = user::ActiveModel {
        full_name: Set(name.to_string()),
        email: Set(email.to_string()),
        hashed_password: Set(hashed_password),
        role: Set(UserRole::Admin),
        is_verified: Set(true),
        created_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };

    let admin = new_admin.insert(db).await?;
    tracing::info!(
        "Preseeded initial administrator: email='{}', id={}",
        admin.email,
        admin.id
    );

    Ok(admin)
}
