use axum::{routing::post, Router, Json, extract::{State, Extension}, middleware};
use serde_json::{json, Value};
use crate::state::AppState;
use crate::errors::AppError;
use crate::middleware::auth::{auth_middleware, RequireBuyer, RequireSeller};
use crate::services::auth::Claims;
use uuid::Uuid;
use chrono::{Utc, Duration};
use rand::{distributions::Alphanumeric, Rng};
use sha2::{Sha256, Digest};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/generate", post(generate_code).route_layer(middleware::from_fn(auth_middleware)))
        .route("/access", post(access_trade).route_layer(middleware::from_fn(auth_middleware)))
}

fn generate_secure_code() -> String {
    let rng = rand::thread_rng();
    let chars: String = rng
        .sample_iter(&Alphanumeric)
        .take(12)
        .map(char::from)
        .collect();
    
    let upper = chars.to_uppercase();
    format!("DT-{}-{}-{}", &upper[0..4], &upper[4..8], &upper[8..12])
}

fn hash_code(code: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(code.as_bytes());
    format!("{:x}", hasher.finalize())
}

async fn generate_code(
    State(state): State<AppState>, 
    Extension(claims): Extension<Claims>,
    _require_seller: RequireSeller,
    Json(payload): Json<Value>
) -> Result<Json<Value>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("Invalid user ID".into()))?;
    
    let shipment_id_str = payload.get("shipment_id").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("shipment_id required".into()))?;
    
    // Verify shipment exists and belongs to this seller
    let user = sqlx::query!("SELECT organization_id FROM users WHERE id = $1", user_id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::NotFound("User not found".into()))?;
        
    let exporter_id = user.organization_id.ok_or_else(|| AppError::Validation("User has no org".into()))?;
    
    let shipment = sqlx::query!("SELECT id FROM shipments WHERE shipment_id = $1 AND exporter_id = $2", shipment_id_str, exporter_id)
        .fetch_optional(&state.db).await?
        .ok_or_else(|| AppError::NotFound("Shipment not found or unauthorized".into()))?;

    let raw_code = generate_secure_code();
    let code_hash = hash_code(&raw_code);
    let expires = Utc::now() + Duration::days(30);

    sqlx::query!(
        "INSERT INTO trade_codes (id, shipment_id, code_hash, created_by, expires_at, created_at) VALUES ($1, $2, $3, $4, $5, $6)",
        Uuid::new_v4(), shipment.id, code_hash, user_id, expires, Utc::now()
    ).execute(&state.db).await?;

    sqlx::query(
        "INSERT INTO audit_logs (id, user_id, action, entity_type, entity_id, created_at) VALUES ($1, $2, 'GENERATE_TRADE_CODE', 'shipment', $3, $4)"
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(shipment.id)
    .bind(Utc::now())
    .execute(&state.db).await?;

    Ok(Json(json!({ "trade_code": raw_code, "expires_at": expires })))
}

async fn access_trade(
    State(state): State<AppState>, 
    Extension(claims): Extension<Claims>,
    _require_buyer: RequireBuyer,
    Json(payload): Json<Value>
) -> Result<Json<Value>, AppError> {
    let user_id = Uuid::parse_str(&claims.sub).map_err(|_| AppError::Auth("Invalid user ID".into()))?;
    
    let trade_code = payload.get("trade_code").and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Validation("trade_code required".into()))?;
        
    let code_hash = hash_code(trade_code);
    
    // Find the code
    let code_rec = sqlx::query!(
        "SELECT shipment_id, expires_at, revoked_at FROM trade_codes WHERE code_hash = $1 ORDER BY created_at DESC LIMIT 1",
        code_hash
    ).fetch_optional(&state.db).await?
    .ok_or_else(|| AppError::NotFound("Trade code is invalid or expired".into()))?;

    if code_rec.revoked_at.is_some() || code_rec.expires_at.map_or(false, |exp| exp < Utc::now()) {
        return Err(AppError::NotFound("Trade code is invalid or expired".into()));
    }

    // Grant access
    // We use ON CONFLICT DO NOTHING to avoid duplicate entries for the same buyer+shipment
    sqlx::query!(
        "INSERT INTO trade_access (id, shipment_id, buyer_id, created_at) VALUES ($1, $2, $3, $4) ON CONFLICT (shipment_id, buyer_id) DO NOTHING",
        Uuid::new_v4(), code_rec.shipment_id, user_id, Utc::now()
    ).execute(&state.db).await?;

    sqlx::query(
        "INSERT INTO audit_logs (id, user_id, action, entity_type, entity_id, created_at) VALUES ($1, $2, 'ACCESS_TRADE', 'shipment', $3, $4)"
    )
    .bind(Uuid::new_v4())
    .bind(user_id)
    .bind(code_rec.shipment_id)
    .bind(Utc::now())
    .execute(&state.db).await?;

    // Return the shipment overview
    let shipment = sqlx::query!(
        r#"
        SELECT s.shipment_id, s.origin_country, s.destination_country, s.current_status::text as status, o.name as exporter_name
        FROM shipments s
        JOIN organizations o ON s.exporter_id = o.id
        WHERE s.id = $1
        "#,
        code_rec.shipment_id
    ).fetch_one(&state.db).await?;

    Ok(Json(json!({ 
        "success": true, 
        "shipment": {
            "id": shipment.shipment_id,
            "origin": shipment.origin_country,
            "destination": shipment.destination_country,
            "status": shipment.status,
            "seller": shipment.exporter_name
        }
    })))
}
