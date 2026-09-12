use axum::{routing::post, Router};

use crate::{db::AppState, handlers::auth};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
        .route("/verify", axum::routing::get(auth::verify_email).post(auth::verify_email))
        .route("/resend-verification", post(auth::resend_verification))
}
