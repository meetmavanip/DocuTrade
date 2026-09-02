use axum::{
    routing::{get, post},
    Router, Json, extract::{State, Path, Extension},
};
use serde_json::{json, Value};
use uuid::Uuid;
use crate::state::AppState;
use crate::errors::AppError;
use crate::services::auth::Claims;
use crate::middleware::auth::auth_middleware;
use axum::middleware as axum_middleware;
use serde::{Deserialize, Serialize};
use crate::models::notification::Notification;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(get_notifications))
        .route("/:id/read", post(mark_as_read))
        .route("/read-all", post(mark_all_read))
        .route_layer(axum_middleware::from_fn(auth_middleware))
}

async fn get_notifications(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Vec<Notification>>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("Invalid user ID in token".into()))?;

    let notifications = sqlx::query_as!(
        Notification,
        r#"
        SELECT id, user_id, type, title, message, is_read, related_entity_id, related_entity_type, created_at
        FROM notifications
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 50
        "#,
        user_id
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch notifications: {}", e);
        AppError::Internal("Database error".into())
    })?;

    Ok(Json(notifications))
}

async fn mark_as_read(
    State(state): State<AppState>,
    Path(notification_id): Path<Uuid>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("Invalid user ID in token".into()))?;

    let result = sqlx::query!(
        r#"
        UPDATE notifications
        SET is_read = true
        WHERE id = $1 AND user_id = $2
        "#,
        notification_id,
        user_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to mark notification as read: {}", e);
        AppError::Internal("Database error".into())
    })?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Notification not found".into()));
    }

    Ok(Json(json!({ "status": "success", "message": "Notification marked as read" })))
}

async fn mark_all_read(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<Value>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub)
        .map_err(|_| AppError::Auth("Invalid user ID in token".into()))?;

    sqlx::query!(
        r#"
        UPDATE notifications
        SET is_read = true
        WHERE user_id = $1 AND is_read = false
        "#,
        user_id
    )
    .execute(&state.db)
    .await
    .map_err(|e| {
        tracing::error!("Failed to mark all notifications as read: {}", e);
        AppError::Internal("Database error".into())
    })?;

    Ok(Json(json!({ "status": "success", "message": "All notifications marked as read" })))
}
