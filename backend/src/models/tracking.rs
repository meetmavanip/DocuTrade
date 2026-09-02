use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use rust_decimal::Decimal;
use super::shipment::ShipmentStatus;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct TrackingEvent {
    pub id: Uuid,
    pub shipment_id: Uuid,
    pub event_type: String,
    pub status: ShipmentStatus,
    pub description: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub speed: Option<Decimal>,
    pub heading: Option<Decimal>,
    pub timestamp: DateTime<Utc>,
    pub source: Option<String>,
    pub accuracy: Option<Decimal>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ShipmentLocation {
    pub id: Uuid,
    pub shipment_id: Uuid,
    pub latitude: Decimal,
    pub longitude: Decimal,
    pub speed: Option<Decimal>,
    pub heading: Option<Decimal>,
    pub timestamp: DateTime<Utc>,
    pub source: Option<String>,
    pub accuracy: Option<Decimal>,
    pub created_at: DateTime<Utc>,
}
