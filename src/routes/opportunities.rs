use axum::{
    routing::get,
    Router,
};

use crate::{db::AppState, handlers::opportunities};

pub fn router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(opportunities::list_opportunities).post(opportunities::create_opportunity),
        )
        .route(
            "/{id}",
            get(opportunities::get_opportunity)
                .put(opportunities::update_opportunity)
                .delete(opportunities::delete_opportunity),
        )
}
