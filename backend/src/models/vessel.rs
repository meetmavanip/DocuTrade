use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Vessel {
    pub id: Uuid,
    pub vessel_name: String,
    pub imo: Option<String>,
    pub mmsi: Option<String>,
    pub vessel_type: Option<String>,
    pub capacity: Option<Decimal>,
    pub deadweight_tonnage: Option<Decimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct VesselVoyage {
    pub id: Uuid,
    pub vessel_id: Uuid,
    pub voyage_id: String,
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub departure_time: Option<DateTime<Utc>>,
    pub eta: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct PortCall {
    pub id: Uuid,
    pub vessel_id: Uuid,
    pub voyage_id: Option<String>,
    pub port_call_id: Option<String>,
    pub port_name: Option<String>,
    pub location: Option<String>,
    pub eta: Option<DateTime<Utc>>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ShipmentVesselLink {
    pub id: Uuid,
    pub shipment_id: Uuid,
    pub vessel_id: Uuid,
    pub voyage_id: Option<String>,
    pub confidence_score: Option<Decimal>,
    pub source: Option<String>,
    pub created_at: DateTime<Utc>,
}
