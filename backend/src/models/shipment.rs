use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use rust_decimal::Decimal;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "shipment_status", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ShipmentStatus {
    Draft,
    DocumentsPending,
    UnderReview,
    Approved,
    ReadyToShip,
    InTransit,
    Delivered,
    Closed,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Shipment {
    pub id: Uuid,
    pub shipment_id: String,
    pub exporter_id: Uuid,
    pub buyer_id: Option<Uuid>,
    pub logistics_provider_id: Option<Uuid>,
    pub origin_country: String,
    pub origin_location: String,
    pub destination_country: String,
    pub destination_location: String,
    pub product_category: Option<String>,
    pub quantity: Option<i32>,
    pub total_value: Decimal,
    pub currency: String,
    pub incoterms: String,
    pub departure_date: Option<DateTime<Utc>>,
    pub expected_arrival: Option<DateTime<Utc>>,
    pub current_status: ShipmentStatus,
    pub metadata_hash: Option<String>,
    pub blockchain_transaction: Option<String>,
    pub container_number: Option<String>,
    pub booking_number: Option<String>,
    pub bill_of_lading_number: Option<String>,
    pub vessel_id: Option<Uuid>,
    pub voyage_id: Option<String>,
    pub vessel_name: Option<String>,
    pub mmsi: Option<String>,
    pub imo_number: Option<String>,
    pub carrier: Option<String>,
    pub current_latitude: Option<Decimal>,
    pub current_longitude: Option<Decimal>,
    pub current_speed: Option<Decimal>,
    pub current_course: Option<Decimal>,
    pub current_vessel_status: Option<String>,
    pub last_tracking_update: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ShipmentItem {
    pub id: Uuid,
    pub shipment_id: Uuid,
    pub product_name: String,
    pub product_code: Option<String>,
    pub description: Option<String>,
    pub quantity: i32,
    pub unit: String,
    pub unit_price: Decimal,
    pub total_price: Decimal,
    pub currency: String,
    pub hs_code: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
