use sqlx::PgPool;
use uuid::Uuid;
use crate::models::shipment::{Shipment, ShipmentStatus};
use crate::errors::AppError;
use rust_decimal::Decimal;

pub async fn create_shipment(
    pool: &PgPool,
    shipment_id: &str,
    exporter_id: Uuid,
    origin_country: &str,
    origin_location: &str,
    destination_country: &str,
    destination_location: &str,
    total_value: Decimal,
    currency: &str,
    incoterms: &str,
) -> Result<Shipment, AppError> {
    let shipment = sqlx::query_as!(
        Shipment,
        r#"
        INSERT INTO shipments (
            shipment_id, exporter_id, origin_country, origin_location, 
            destination_country, destination_location, total_value, currency, incoterms
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING 
            id, shipment_id, exporter_id, buyer_id, logistics_provider_id,
            origin_country, origin_location, destination_country, destination_location,
            product_category, quantity, total_value, currency, incoterms,
            departure_date, expected_arrival, current_status AS "current_status: ShipmentStatus",
            metadata_hash, blockchain_transaction, created_at, updated_at
        "#,
        shipment_id,
        exporter_id,
        origin_country,
        origin_location,
        destination_country,
        destination_location,
        total_value,
        currency,
        incoterms
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // In a real app, this should be wrapped in a transaction along with an Audit Log insert.
    Ok(shipment)
}

pub async fn get_shipment_by_id(pool: &PgPool, id: Uuid) -> Result<Shipment, AppError> {
    let shipment = sqlx::query_as!(
        Shipment,
        r#"
        SELECT 
            id, shipment_id, exporter_id, buyer_id, logistics_provider_id,
            origin_country, origin_location, destination_country, destination_location,
            product_category, quantity, total_value, currency, incoterms,
            departure_date, expected_arrival, current_status AS "current_status: ShipmentStatus",
            metadata_hash, blockchain_transaction, created_at, updated_at
        FROM shipments
        WHERE id = $1
        "#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    Ok(shipment)
}
