use anyhow::{Result, Context, anyhow};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::env;
use std::time::Duration;
use tokio::time::timeout;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{info, error, warn};

#[derive(Serialize)]
struct AisSubscription {
    #[serde(rename = "APIKey")]
    api_key: String,
    #[serde(rename = "BoundingBoxes")]
    bounding_boxes: Vec<Vec<Vec<f64>>>,
    #[serde(rename = "FiltersShipMMSI")]
    filters_ship_mmsi: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct AisMessage {
    #[serde(rename = "MessageType")]
    pub message_type: String,
    #[serde(rename = "MetaData")]
    pub meta_data: AisMetaData,
    #[serde(rename = "Message")]
    pub message: AisMessageBody,
}

#[derive(Deserialize, Debug)]
pub struct AisMetaData {
    #[serde(rename = "MMSI")]
    pub mmsi: i64,
    #[serde(rename = "ShipName")]
    pub ship_name: String,
    #[serde(rename = "latitude")]
    pub latitude: f64,
    #[serde(rename = "longitude")]
    pub longitude: f64,
    #[serde(rename = "time_utc")]
    pub time_utc: String,
}

#[derive(Deserialize, Debug)]
pub struct AisMessageBody {
    #[serde(rename = "PositionReport")]
    pub position_report: Option<PositionReport>,
    #[serde(rename = "ShipStaticData")]
    pub ship_static_data: Option<ShipStaticData>,
}

#[derive(Deserialize, Debug)]
pub struct PositionReport {
    #[serde(rename = "Cog")]
    pub cog: f64,
    #[serde(rename = "Sog")]
    pub sog: f64,
    #[serde(rename = "NavigationalStatus")]
    pub navigational_status: i32,
    #[serde(rename = "TrueHeading")]
    pub true_heading: i32,
}

#[derive(Deserialize, Debug)]
pub struct ShipStaticData {
    #[serde(rename = "ImoNumber")]
    pub imo_number: i32,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Type")]
    pub ship_type: i32,
}

#[derive(Debug, Serialize)]
pub struct VerifiedVessel {
    pub vessel_name: String,
    pub mmsi: String,
    pub imo_number: String,
    pub ship_type: String,
    pub current_status: String,
    pub latitude: f64,
    pub longitude: f64,
}

pub async fn verify_vessel(mmsi: &str) -> Result<VerifiedVessel> {
    let api_key = env::var("AISSTREAM_API_KEY").unwrap_or_else(|_| "DEMO_KEY".into());
    let url = "wss://stream.aisstream.io/v0/stream";

    let (mut ws_stream, _) = connect_async(url).await.context("Failed to connect to AISStream")?;

    let sub = AisSubscription {
        api_key,
        // AISStream bounding boxes must be [TopLeft, BottomRight] -> [[NorthWest], [SouthEast]]
        bounding_boxes: vec![vec![vec![90.0, -180.0], vec![-90.0, 180.0]]],
        filters_ship_mmsi: vec![mmsi.to_string()],
    };

    let sub_json = serde_json::to_string(&sub)?;
    ws_stream.send(Message::Text(sub_json.into())).await?;

    info!("Subscribed to AISStream for MMSI: {}", mmsi);

    let mut verified = VerifiedVessel {
        vessel_name: "Unknown Vessel".to_string(),
        mmsi: mmsi.to_string(),
        imo_number: "Unknown".to_string(),
        ship_type: "Cargo".to_string(),
        current_status: "Under Way".to_string(),
        latitude: 0.0,
        longitude: 0.0,
    };

    // Wait up to 45 seconds for a response
    let wait_future = async {
        while let Some(msg) = ws_stream.next().await {
            let msg = msg?;
            if let Message::Text(text) = msg {
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                    if let Some(msg_type) = v.get("MessageType").and_then(|m| m.as_str()) {
                        if msg_type == "SubscriptionConfirmation" {
                            info!("AISStream subscription confirmed for MMSI {}", mmsi);
                            continue;
                        }
                    }

                    if let Some(meta) = v.get("MetaData") {
                        verified.latitude = meta.get("latitude").and_then(|l| l.as_f64()).unwrap_or(0.0);
                        verified.longitude = meta.get("longitude").and_then(|l| l.as_f64()).unwrap_or(0.0);
                        if let Some(name) = meta.get("ShipName").and_then(|n| n.as_str()) {
                            let trimmed = name.trim();
                            if !trimmed.is_empty() {
                                verified.vessel_name = trimmed.to_string();
                            }
                        }
                        if let Some(mmsi_val) = meta.get("MMSI") {
                            verified.mmsi = mmsi_val.to_string();
                        }
                    }

                    let msg_obj = v.get("Message");
                    if let Some(pos) = msg_obj.and_then(|m| {
                        m.get("PositionReport")
                            .or_else(|| m.get("StandardClassBPositionReport"))
                            .or_else(|| m.get("ExtendedClassBPositionReport"))
                    }) {
                        if let Some(status_num) = pos.get("NavigationalStatus").and_then(|s| s.as_i64()) {
                            verified.current_status = match status_num {
                                0 => "Under Way Using Engine",
                                1 => "At Anchor",
                                2 => "Not Under Command",
                                3 => "Restricted Maneuverability",
                                5 => "Moored",
                                8 => "Under Way Sailing",
                                _ => "Under Way",
                            }.to_string();
                        }
                        return Ok::<VerifiedVessel, anyhow::Error>(verified);
                    }

                    if let Some(stat) = msg_obj.and_then(|m| m.get("ShipStaticData")) {
                        if let Some(imo) = stat.get("ImoNumber").and_then(|i| i.as_i64()) {
                            if imo > 0 {
                                verified.imo_number = imo.to_string();
                            }
                        }
                        if let Some(name) = stat.get("Name").and_then(|n| n.as_str()) {
                            let trimmed = name.trim();
                            if !trimmed.is_empty() {
                                verified.vessel_name = trimmed.to_string();
                            }
                        }
                        return Ok::<VerifiedVessel, anyhow::Error>(verified);
                    }
                }
            }
        }
        Err(anyhow!("Stream closed without vessel data"))
    };

    match timeout(Duration::from_secs(45), wait_future).await {
        Ok(Ok(v)) => Ok(v),
        Ok(Err(e)) => Err(e),
        Err(_) => {
            error!("Timeout waiting for AIS data for MMSI: {}", mmsi);
            Err(anyhow!("Timeout waiting for AIS data for MMSI: {}", mmsi))
        }
    }
}

pub async fn start_tracking_worker(db: PgPool) {
    tokio::spawn(async move {
        loop {
            // Get all active MMSIs from shipments
            let active_mmsis: Vec<String> = match sqlx::query!("SELECT DISTINCT mmsi FROM shipments WHERE mmsi IS NOT NULL AND current_status NOT IN ('DELIVERED', 'CLOSED')")
                .fetch_all(&db)
                .await {
                    Ok(rows) => rows.into_iter().filter_map(|r| r.mmsi).collect(),
                    Err(e) => {
                        error!("Failed to fetch active MMSIs: {}", e);
                        tokio::time::sleep(Duration::from_secs(60)).await;
                        continue;
                    }
                };

            if active_mmsis.is_empty() {
                tokio::time::sleep(Duration::from_secs(60)).await;
                continue;
            }

            info!("Starting AISStream tracking for {} active vessels", active_mmsis.len());

            let api_key = env::var("AISSTREAM_API_KEY").unwrap_or_else(|_| "DEMO_KEY".into());
            let url = "wss://stream.aisstream.io/v0/stream";

            match connect_async(url).await {
                Ok((mut ws_stream, _)) => {
                    let sub = AisSubscription {
                        api_key,
                        // AISStream bounding boxes must be [TopLeft, BottomRight]
                        bounding_boxes: vec![vec![vec![90.0, -180.0], vec![-90.0, 180.0]]],
                        filters_ship_mmsi: active_mmsis.clone(),
                    };

                    if let Ok(sub_json) = serde_json::to_string(&sub) {
                        if let Err(e) = ws_stream.send(Message::Text(sub_json.into())).await {
                            error!("Failed to send AIS subscription: {}", e);
                        } else {
                            info!("Subscribed to AISStream for {} MMSIs: {:?}", active_mmsis.len(), active_mmsis);
                            // Listen loop
                            loop {
                                match timeout(Duration::from_secs(300), ws_stream.next()).await {
                                    Ok(Some(Ok(msg))) => {
                                        if let Message::Text(text) = msg {
                                            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
                                                if let Some(meta) = v.get("MetaData") {
                                                    let lat_f = meta.get("latitude").and_then(|l| l.as_f64()).unwrap_or_default();
                                                    let lon_f = meta.get("longitude").and_then(|l| l.as_f64()).unwrap_or_default();
                                                    let mmsi_str = meta.get("MMSI").map(|m| m.to_string()).unwrap_or_default();
                                                    
                                                    let msg_obj = v.get("Message");
                                                    let pos_opt = msg_obj.and_then(|m| {
                                                        m.get("PositionReport")
                                                            .or_else(|| m.get("StandardClassBPositionReport"))
                                                            .or_else(|| m.get("ExtendedClassBPositionReport"))
                                                    });

                                                    if let Some(pos) = pos_opt {
                                                        let status = match pos.get("NavigationalStatus").and_then(|s| s.as_i64()).unwrap_or(-1) {
                                                            0 => "Under Way Using Engine",
                                                            1 => "At Anchor",
                                                            2 => "Not Under Command",
                                                            3 => "Restricted Maneuverability",
                                                            5 => "Moored",
                                                            8 => "Under Way Sailing",
                                                            _ => "Under Way",
                                                        };

                                                        let sog_f = pos.get("Sog").and_then(|s| s.as_f64()).unwrap_or_default();
                                                        let cog_f = pos.get("Cog").and_then(|c| c.as_f64()).unwrap_or_default();

                                                        let lat = rust_decimal::Decimal::from_f64_retain(lat_f).unwrap_or_default();
                                                        let lon = rust_decimal::Decimal::from_f64_retain(lon_f).unwrap_or_default();
                                                        let sog = rust_decimal::Decimal::from_f64_retain(sog_f).unwrap_or_default();
                                                        let cog = rust_decimal::Decimal::from_f64_retain(cog_f).unwrap_or_default();

                                                        match sqlx::query!(
                                                            "UPDATE shipments 
                                                             SET current_latitude = $1, current_longitude = $2, current_speed = $3, current_course = $4, current_vessel_status = $5, last_tracking_update = NOW()
                                                             WHERE mmsi = $6",
                                                             lat, lon, sog, cog, status, mmsi_str
                                                        ).execute(&db).await {
                                                            Ok(result) => {
                                                                if result.rows_affected() == 0 {
                                                                    warn!("AIS update matched 0 rows for MMSI {} — check mmsi column format/value", mmsi_str);
                                                                } else {
                                                                    info!("Updated position for MMSI {}: lat={}, lon={}", mmsi_str, lat, lon);
                                                                }
                                                            }
                                                            Err(e) => {
                                                                error!("Failed to update shipment position for MMSI {}: {}", mmsi_str, e);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    Ok(Some(Err(e))) => {
                                        error!("WebSocket error: {}", e);
                                        break;
                                    },
                                    Ok(None) => {
                                        warn!("WebSocket closed by remote");
                                        break;
                                    },
                                    Err(_) => {
                                        // Timeout, refresh connection and MMSIs
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect to AISStream: {}", e);
                    tokio::time::sleep(Duration::from_secs(60)).await;
                }
            }
        }
    });
}
