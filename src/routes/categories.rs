use axum::{
    routing::{delete, get},
    Router,
};

use crate::{db::AppState, handlers::categories};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(categories::list_categories).post(categories::create_category))
        .route("/{id}", delete(categories::delete_category))
}
