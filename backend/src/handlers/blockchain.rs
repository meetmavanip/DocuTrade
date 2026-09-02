use axum::{routing::get, Router, Json, extract::State};
use serde_json::{json, Value};
use crate::state::AppState;
use crate::errors::AppError;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/events", get(list_events))
        .route("/status", get(network_status))
}

async fn list_events(State(_state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!([])))
}

async fn network_status(State(state): State<AppState>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ 
        "network": "Arbitrum Sepolia", 
        "connected": true, 
        "block_number": 71234567,
        "contract_address": state.config.document_verification_contract
    })))
}
