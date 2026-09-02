use crate::services::kpler_client::{KplerClient, KplerTrade, KplerPortCall};
use anyhow::Result;
use sqlx::PgPool;
use tracing::{info, error};
use serde::{Deserialize, Serialize};

pub struct VesselTrackingService {
    kpler_client: KplerClient,
    pool: PgPool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TrackingResponse {
    pub vessel_name: Option<String>,
    pub imo: Option<i32>,
    pub current_status: String,
    pub origin: Option<String>,
    pub destination: Option<String>,
    pub eta: Option<String>,
    pub port_calls: Vec<KplerPortCall>,
    // Fallback/Simulated coordinates based on origin/dest if live AIS not present
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub is_live: bool,
}

impl VesselTrackingService {
    pub fn new(pool: PgPool) -> Result<Self> {
        Ok(Self {
            kpler_client: KplerClient::new()?,
            pool,
        })
    }
    
    pub async fn get_vessel_tracking(&self, _vessel_identifier: &str) -> Result<TrackingResponse> {
        // Completely fake API data as requested, bypassing Kpler
        let mut response = TrackingResponse {
            vessel_name: Some("MSC DEMO".to_string()),
            imo: Some(9440621),
            current_status: "IN TRANSIT".to_string(),
            origin: Some("Mundra, India".to_string()),
            destination: Some("Jebel Ali, UAE".to_string()),
            eta: Some("2026-09-05T12:00:00Z".to_string()),
            port_calls: vec![],
            latitude: None,
            longitude: None,
            is_live: true,
        };
        
        // FAKE AIS SIMULATION LOOP (Mundra -> Dubai over 10 minutes)
        let now = chrono::Utc::now().timestamp();
        let loop_duration = 600.0; // 10 minutes
        let progress = ((now % 600) as f64) / loop_duration;
        
        let start_lat = 22.73;
        let start_lng = 69.73;
        let end_lat = 25.0;
        let end_lng = 55.0;
        
        response.latitude = Some(start_lat + (end_lat - start_lat) * progress);
        response.longitude = Some(start_lng + (end_lng - start_lng) * progress);
        
        Ok(response)
    }
}
