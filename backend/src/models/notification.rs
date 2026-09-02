use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    pub r#type: String, // reserved keyword
    pub title: String,
    pub message: String,
    pub is_read: bool,
    pub related_entity_id: Option<Uuid>,
    pub related_entity_type: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateNotification {
    pub user_id: Uuid,
    pub r#type: String,
    pub title: String,
    pub message: String,
    pub related_entity_id: Option<Uuid>,
    pub related_entity_type: Option<String>,
}
