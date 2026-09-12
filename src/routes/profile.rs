use axum::{
    routing::get,
    Router,
};

use crate::{db::AppState, handlers::profile};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/me", get(profile::get_my_profile).put(profile::update_my_profile))
        .route("/{user_id}", get(profile::get_profile_by_user_id))
}
