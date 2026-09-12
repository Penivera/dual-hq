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
use rand_core::RngCore;
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set};
use serde::Deserialize;

use crate::{
    db::AppState,
    entities::{user, User},
    errors::{AppError, ErrorDetail},
    schemas::auth::{
        Claims, LoginRequest, ResendVerificationRequest, Token, UserCreate, UserResponse,
        VerificationResponse, VerifyEmailQuery,
    },
};

fn get_argon2() -> Argon2<'static> {
    let params = argon2::Params::new(19456, 2, 1, None).unwrap_or_default();
    Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
}

pub fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = get_argon2();
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
    get_argon2()
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

    let mut token_bytes = [0u8; 16];
    rand_core::OsRng.fill_bytes(&mut token_bytes);
    let verification_token: String = token_bytes.iter().map(|b| format!("{b:02x}")).collect();

    let new_user = user::ActiveModel {
        full_name: Set(payload.full_name),
        email: Set(payload.email),
        hashed_password: Set(hashed_password),
        role: Set(user::UserRole::Applicant),
        is_verified: Set(false),
        verification_token: Set(Some(verification_token.clone())),
        created_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };

    let user = new_user.insert(&state.db).await?;

    // Dispatch asynchronous verification email
    crate::email::send_verification_email(
        state.config.clone(),
        user.email.clone(),
        user.full_name.clone(),
        verification_token,
    );

    let response = UserResponse {
        id: user.id,
        full_name: user.full_name,
        email: user.email,
        role: user.role,
        is_verified: user.is_verified,
        created_at: user.created_at,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

#[utoipa::path(
    get,
    path = "/auth/verify",
    tag = "Auth",
    params(
        ("token" = String, Query, description = "Email verification token")
    ),
    responses(
        (status = 200, description = "Email verified successfully", body = VerificationResponse),
        (status = 400, description = "Invalid or expired token", body = ErrorDetail)
    )
)]
pub async fn verify_email(
    State(state): State<AppState>,
    axum::extract::Query(query): axum::extract::Query<VerifyEmailQuery>,
) -> Result<Json<VerificationResponse>, AppError> {
    if query.token.trim().is_empty() {
        return Err(AppError::Validation("Token cannot be empty".to_string()));
    }

    let user = User::find()
        .filter(user::Column::VerificationToken.eq(&query.token))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::Validation("Invalid or expired verification token".to_string()))?;

    let mut active: user::ActiveModel = user.into();
    active.is_verified = Set(true);
    active.verification_token = Set(None);
    let updated = active.update(&state.db).await?;

    // Dispatch asynchronous welcome email to the newly verified user
    crate::email::send_welcome_email(
        state.config.clone(),
        updated.email,
        updated.full_name,
    );

    Ok(Json(crate::schemas::auth::VerificationResponse {
        message: "Email verified successfully! You can now log in and apply for internships.".to_string(),
        is_verified: true,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/resend-verification",
    tag = "Auth",
    request_body = ResendVerificationRequest,
    responses(
        (status = 200, description = "Verification email sent", body = VerificationResponse),
        (status = 404, description = "User not found", body = ErrorDetail)
    )
)]
pub async fn resend_verification(
    State(state): State<AppState>,
    Json(payload): Json<ResendVerificationRequest>,
) -> Result<Json<VerificationResponse>, AppError> {
    let user = User::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User with this email was not found".to_string()))?;

    if user.is_verified {
        return Ok(Json(VerificationResponse {
            message: "Email is already verified.".to_string(),
            is_verified: true,
        }));
    }

    let mut token_bytes = [0u8; 16];
    rand_core::OsRng.fill_bytes(&mut token_bytes);
    let verification_token: String = token_bytes.iter().map(|b| format!("{b:02x}")).collect();

    let mut active: user::ActiveModel = user.clone().into();
    active.verification_token = Set(Some(verification_token.clone()));
    active.update(&state.db).await?;

    crate::email::send_verification_email(
        state.config.clone(),
        user.email,
        user.full_name,
        verification_token,
    );

    Ok(Json(VerificationResponse {
        message: "Verification email has been sent. Please check your inbox.".to_string(),
        is_verified: false,
    }))
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
