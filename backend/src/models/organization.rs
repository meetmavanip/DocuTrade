use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, sqlx::Type)]
#[sqlx(type_name = "organization_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrganizationType {
    Exporter,
    Buyer,
    Logistics,
    Inspection,
    Customs,
    Platform,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub legal_name: Option<String>,
    pub organization_type: OrganizationType,
    pub registration_number: Option<String>,
    pub tax_id: Option<String>,
    pub country: String,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub website: Option<String>,
    pub wallet_address: Option<String>,
    pub is_verified: Option<bool>,
    pub verified_at: Option<DateTime<Utc>>,
    pub verified_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
