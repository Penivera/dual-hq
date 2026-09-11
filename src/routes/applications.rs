use axum::{
    routing::{get, patch},
    Router,
};

use crate::{db::AppState, handlers::applications};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(applications::list_applications).post(applications::apply),
        )
        .route("/me", get(applications::my_applications))
        .route(
            "/{id}",
            get(applications::get_application).delete(applications::delete_application),
        )
        .route("/{id}/status", patch(applications::update_application_status))
}
