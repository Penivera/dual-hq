use axum::{
    routing::{get, patch},
    Router,
};

use crate::{db::AppState, handlers::notifications};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(notifications::list_notifications))
        .route("/{id}/read", patch(notifications::mark_notification_read))
        .route("/read-all", patch(notifications::mark_all_notifications_read))
}
