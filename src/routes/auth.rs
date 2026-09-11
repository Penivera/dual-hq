use axum::{routing::post, Router};

use crate::{db::AppState, handlers::auth};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/register", post(auth::register))
        .route("/login", post(auth::login))
}
