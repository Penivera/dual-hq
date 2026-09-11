use std::time::{SystemTime, UNIX_EPOCH};

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use axum::{
    body::Bytes,
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::{
    db::AppState,
    entities::{user, User},
    errors::{AppError, ErrorDetail},
    schemas::auth::{Claims, LoginRequest, Token, UserCreate, UserResponse},
};

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| AppError::Internal(format!("Failed to hash password: {e}")))?
        .to_string();
    Ok(password_hash)
}

pub fn verify_password(password: &str, hashed_password: &str) -> bool {
    let parsed_hash = match PasswordHash::new(hashed_password) {
        Ok(h) => h,
        Err(_) => return false,
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}

pub fn create_jwt_token(
    user_id: i32,
    role: &str,
    secret: &str,
    expiry_hours: i64,
) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .as_secs() as usize;

    let expiration = now + (expiry_hours as usize * 3600);

    let claims = Claims {
        sub: user_id.to_string(),
        role: role.to_string(),
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|e| AppError::Internal(format!("Failed to create token: {e}")))
}

#[utoipa::path(
    post,
    path = "/auth/register",
    tag = "Auth",
    request_body = UserCreate,
    responses(
        (status = 201, description = "User registered successfully", body = UserResponse),
        (status = 409, description = "Email already registered", body = ErrorDetail),
        (status = 422, description = "Validation error", body = ErrorDetail)
    )
)]
pub async fn register(
    State(state): State<AppState>,
    Json(payload): Json<UserCreate>,
) -> Result<impl IntoResponse, AppError> {
    if payload.email.trim().is_empty() || payload.password.trim().is_empty() {
        return Err(AppError::Validation(
            "Email and password cannot be empty".to_string(),
        ));
    }

    // Check if email already registered
    let existing_user = User::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&state.db)
        .await?;

    if existing_user.is_some() {
        return Err(AppError::Conflict("Email already registered".to_string()));
    }

    let hashed_password = hash_password(&payload.password)?;

    let new_user = user::ActiveModel {
        full_name: Set(payload.full_name),
        email: Set(payload.email),
        hashed_password: Set(hashed_password),
        role: Set(user::UserRole::Applicant),
        created_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };

    let user = new_user.insert(&state.db).await?;

    let response = UserResponse {
        id: user.id,
        full_name: user.full_name,
        email: user.email,
        role: user.role,
        created_at: user.created_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful, returns JWT access token", body = Token),
        (status = 401, description = "Invalid email or password", body = ErrorDetail)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Token>, AppError> {
    // Determine content type: JSON or x-www-form-urlencoded (OAuth2 standard)
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/json");

    let login_data: LoginRequest = if content_type.contains("application/x-www-form-urlencoded") {
        #[derive(Deserialize)]
        struct FormLogin {
            username: Option<String>,
            email: Option<String>,
            password: String,
        }
        let form: FormLogin = serde_urlencoded::from_bytes(&body).map_err(|e| {
            AppError::Validation(format!("Invalid form credentials: {e}"))
        })?;
        LoginRequest {
            email: form.email,
            username: form.username,
            password: form.password,
        }
    } else {
        serde_json::from_slice(&body).map_err(|e| {
            AppError::Validation(format!("Invalid JSON credentials: {e}"))
        })?
    };

    let email = login_data.get_email();
    if email.is_empty() || login_data.password.is_empty() {
        return Err(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
    }

    let user = User::find()
        .filter(user::Column::Email.eq(email))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid email or password".to_string()))?;

    if !verify_password(&login_data.password, &user.hashed_password) {
        return Err(AppError::Unauthorized(
            "Invalid email or password".to_string(),
        ));
    }

    let role_str = match user.role {
        user::UserRole::Admin => "admin",
        user::UserRole::Applicant => "applicant",
    };

    let token = create_jwt_token(
        user.id,
        role_str,
        &state.config.jwt_secret,
        state.config.jwt_expiry_hours,
    )?;

    Ok(Json(Token::new(token)))
}
