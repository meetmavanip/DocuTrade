use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use sqlx::types::JsonValue;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "tx_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TxStatus {
    Pending,
    Confirmed,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct BlockchainTransaction {
    pub id: Uuid,
    pub shipment_id: Option<Uuid>,
    pub document_id: Option<Uuid>,
    pub transaction_hash: String,
    pub chain_id: i64,
    pub network: String,
    pub contract_address: String,
    pub block_number: Option<i64>,
    pub status: TxStatus,
    pub transaction_type: String,
    pub submitted_at: DateTime<Utc>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct BlockchainEvent {
    pub id: Uuid,
    pub transaction_id: Uuid,
    pub event_name: String,
    pub contract_address: String,
    pub transaction_hash: String,
    pub block_number: i64,
    pub log_index: i32,
    pub event_data: Option<JsonValue>,
    pub occurred_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}
