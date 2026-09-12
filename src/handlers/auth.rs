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
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand_core::RngCore;
use sea_orm::{
    sea_query::Expr, ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, Set,
};
use serde::Deserialize;
use sha2::{Digest, Sha256};

use crate::{
    db::AppState,
    entities::{
        refresh_token,
        user::{self, UserRole, UserStatus},
        RefreshToken, User,
    },
    errors::{AppError, ErrorDetail},
    schemas::auth::{
        Claims, LoginRequest, RefreshTokenRequest, ResendVerificationRequest, Token, UserCreate,
        UserResponse, VerificationResponse, VerifyEmailQuery,
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

pub fn create_jwt_token_with_secs(
    user_id: i32,
    role: &str,
    secret: &str,
    expiry_secs: u64,
) -> Result<String, AppError> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| AppError::Internal(e.to_string()))?
        .as_secs();

    let expiration = (now + expiry_secs) as usize;

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

pub fn create_jwt_token(
    user_id: i32,
    role: &str,
    secret: &str,
    expiry_hours: i64,
) -> Result<String, AppError> {
    create_jwt_token_with_secs(user_id, role, secret, (expiry_hours * 3600) as u64)
}

fn generate_random_token_string() -> String {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

fn hash_token(raw_token: &str) -> String {
    format!("{:x}", Sha256::digest(raw_token.as_bytes()))
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
    let verification_token = generate_random_token_string();

    let role = payload.role.unwrap_or(UserRole::Applicant);
    let status = match role {
        UserRole::Recruiter => UserStatus::Pending,
        _ => UserStatus::Active,
    };

    let new_user = user::ActiveModel {
        full_name: Set(payload.full_name),
        email: Set(payload.email),
        hashed_password: Set(hashed_password),
        role: Set(role.clone()),
        status: Set(status),
        is_verified: Set(false),
        verification_token: Set(Some(verification_token.clone())),
        created_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };

    let user = new_user.insert(&state.db).await?;

    if role == UserRole::Recruiter {
        crate::email::send_recruiter_pending_email(
            state.config.clone(),
            user.email.clone(),
            user.full_name.clone(),
        );
    } else {
        crate::email::send_verification_email(
            state.config.clone(),
            user.email.clone(),
            user.full_name.clone(),
            verification_token,
        );
    }

    let response = UserResponse {
        id: user.id,
        full_name: user.full_name,
        email: user.email,
        role: user.role,
        status: user.status,
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
        return Err(AppError::Validation("Token is required".to_string()));
    }

    let user = User::find()
        .filter(user::Column::VerificationToken.eq(&query.token))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::Validation("Invalid or expired verification token".to_string()))?;

    let mut user_active: user::ActiveModel = user.clone().into();
    user_active.is_verified = Set(true);
    user_active.verification_token = Set(None);
    user_active.update(&state.db).await?;

    crate::email::send_welcome_email(
        state.config.clone(),
        user.email.clone(),
        user.full_name.clone(),
    );

    Ok(Json(VerificationResponse {
        message: "Email verified successfully".to_string(),
        is_verified: true,
    }))
}

#[utoipa::path(
    post,
    path = "/auth/resend-verification",
    tag = "Auth",
    request_body = ResendVerificationRequest,
    responses(
        (status = 200, description = "Verification email resent", body = VerificationResponse),
        (status = 400, description = "Email already verified or not found", body = ErrorDetail)
    )
)]
pub async fn resend_verification(
    State(state): State<AppState>,
    Json(payload): Json<ResendVerificationRequest>,
) -> Result<Json<VerificationResponse>, AppError> {
    if payload.email.trim().is_empty() {
        return Err(AppError::Validation("Email is required".to_string()));
    }

    let user = User::find()
        .filter(user::Column::Email.eq(&payload.email))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    if user.is_verified {
        return Err(AppError::Validation(
            "Account is already verified".to_string(),
        ));
    }

    let new_token = generate_random_token_string();
    let mut user_active: user::ActiveModel = user.clone().into();
    user_active.verification_token = Set(Some(new_token.clone()));
    user_active.update(&state.db).await?;

    crate::email::send_verification_email(
        state.config.clone(),
        user.email.clone(),
        user.full_name.clone(),
        new_token,
    );

    Ok(Json(VerificationResponse {
        message: "Verification email resent successfully".to_string(),
        is_verified: false,
    }))
}

#[derive(Deserialize)]
struct FormLogin {
    username: Option<String>,
    email: Option<String>,
    password: String,
}

#[utoipa::path(
    post,
    path = "/auth/login",
    tag = "Auth",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = Token),
        (status = 401, description = "Invalid credentials", body = ErrorDetail),
        (status = 403, description = "Account pending or suspended", body = ErrorDetail),
        (status = 422, description = "Validation error", body = ErrorDetail)
    )
)]
pub async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<Token>, AppError> {
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    let login_data: LoginRequest = if content_type.contains("application/x-www-form-urlencoded") {
        if body.is_empty() {
            return Err(AppError::Validation("Request body cannot be empty".to_string()));
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

    match user.status {
        UserStatus::Pending => {
            return Err(AppError::Forbidden(
                "Your account is pending review by an administrator".to_string(),
            ));
        }
        UserStatus::Suspended => {
            return Err(AppError::Forbidden(
                "Your account has been suspended".to_string(),
            ));
        }
        UserStatus::Active => {}
    }

    let role_str = match user.role {
        UserRole::Admin => "admin",
        UserRole::Recruiter => "recruiter",
        UserRole::Applicant => "applicant",
    };

    // 15-minute access token (900 seconds)
    let access_token = create_jwt_token_with_secs(
        user.id,
        role_str,
        &state.config.jwt_secret,
        900,
    )?;

    // 7-day refresh token
    let raw_refresh_token = generate_random_token_string();
    let hashed_refresh_token = hash_token(&raw_refresh_token);
    let refresh_expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    let refresh_entry = refresh_token::ActiveModel {
        user_id: Set(user.id),
        token: Set(hashed_refresh_token),
        expires_at: Set(refresh_expires_at.into()),
        revoked: Set(false),
        created_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };
    refresh_entry.insert(&state.db).await?;

    Ok(Json(Token::new(access_token, raw_refresh_token, 900)))
}

#[utoipa::path(
    post,
    path = "/auth/refresh",
    tag = "Auth",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = Token),
        (status = 401, description = "Invalid or expired refresh token", body = ErrorDetail),
        (status = 403, description = "Account pending or suspended", body = ErrorDetail)
    )
)]
pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Result<Json<Token>, AppError> {
    if payload.refresh_token.trim().is_empty() {
        return Err(AppError::Unauthorized("Refresh token is required".to_string()));
    }

    let hashed_token = hash_token(&payload.refresh_token);

    let token_record = RefreshToken::find()
        .filter(refresh_token::Column::Token.eq(&hashed_token))
        .filter(refresh_token::Column::Revoked.eq(false))
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("Invalid or revoked refresh token".to_string()))?;

    if token_record.expires_at < chrono::Utc::now().fixed_offset() {
        return Err(AppError::Unauthorized("Refresh token has expired".to_string()));
    }

    let user = User::find_by_id(token_record.user_id)
        .one(&state.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("User not found".to_string()))?;

    match user.status {
        UserStatus::Pending => {
            return Err(AppError::Forbidden(
                "Your account is pending review by an administrator".to_string(),
            ));
        }
        UserStatus::Suspended => {
            return Err(AppError::Forbidden(
                "Your account has been suspended".to_string(),
            ));
        }
        UserStatus::Active => {}
    }

    // Revoke old refresh token (rotation)
    let mut token_active: refresh_token::ActiveModel = token_record.into();
    token_active.revoked = Set(true);
    token_active.update(&state.db).await?;

    let role_str = match user.role {
        UserRole::Admin => "admin",
        UserRole::Recruiter => "recruiter",
        UserRole::Applicant => "applicant",
    };

    let access_token = create_jwt_token_with_secs(
        user.id,
        role_str,
        &state.config.jwt_secret,
        900,
    )?;

    let new_raw_token = generate_random_token_string();
    let new_hashed = hash_token(&new_raw_token);
    let refresh_expires_at = chrono::Utc::now() + chrono::Duration::days(7);

    let new_entry = refresh_token::ActiveModel {
        user_id: Set(user.id),
        token: Set(new_hashed),
        expires_at: Set(refresh_expires_at.into()),
        revoked: Set(false),
        created_at: Set(chrono::Utc::now().into()),
        ..Default::default()
    };
    new_entry.insert(&state.db).await?;

    Ok(Json(Token::new(access_token, new_raw_token, 900)))
}

#[utoipa::path(
    post,
    path = "/auth/logout",
    tag = "Auth",
    request_body(content = Option<RefreshTokenRequest>, description = "Optional refresh token to revoke"),
    responses(
        (status = 200, description = "Logged out successfully")
    )
)]
pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Option<Json<RefreshTokenRequest>>,
) -> Result<StatusCode, AppError> {
    if let Some(Json(req)) = body {
        if !req.refresh_token.trim().is_empty() {
            let hashed = hash_token(&req.refresh_token);
            let _ = RefreshToken::update_many()
                .filter(refresh_token::Column::Token.eq(hashed))
                .col_expr(refresh_token::Column::Revoked, Expr::value(true))
                .exec(&state.db)
                .await;
        }
    }

    // If Bearer token present in header, revoke all refresh tokens for that user as well
    if let Some(auth_val) = headers.get(header::AUTHORIZATION).and_then(|h| h.to_str().ok()) {
        if let Some(token) = auth_val.strip_prefix("Bearer ").or_else(|| auth_val.strip_prefix("bearer ")) {
            let mut validation = Validation::default();
            validation.validate_exp = true;
            if let Ok(token_data) = decode::<Claims>(
                token,
                &DecodingKey::from_secret(state.config.jwt_secret.as_bytes()),
                &validation,
            ) {
                if let Ok(user_id) = token_data.claims.sub.parse::<i32>() {
                    let _ = RefreshToken::update_many()
                        .filter(refresh_token::Column::UserId.eq(user_id))
                        .col_expr(refresh_token::Column::Revoked, Expr::value(true))
                        .exec(&state.db)
                        .await;
                }
            }
        }
    }

    Ok(StatusCode::OK)
}
