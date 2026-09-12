use sea_orm::{ActiveModelTrait, DatabaseConnection, Set};
use tracing::error;

use crate::entities::notification;

/// Creates an in-app notification in a fire-and-forget background task.
pub fn send_notification(db: DatabaseConnection, user_id: i32, title: String, body: String) {
    tokio::spawn(async move {
        let active_model = notification::ActiveModel {
            user_id: Set(user_id),
            title: Set(title),
            body: Set(body),
            read: Set(false),
            created_at: Set(chrono::Utc::now().into()),
            ..Default::default()
        };

        if let Err(e) = active_model.insert(&db).await {
            error!(error = %e, user_id = user_id, "Failed to insert in-app notification");
        }
    });
}
