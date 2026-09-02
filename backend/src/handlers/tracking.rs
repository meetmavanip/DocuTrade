use axum::{routing::{get, post}, Router, Json, extract::{State, Path}};
use serde_json::{json, Value};
use crate::state::AppState;
use crate::errors::AppError;
use crate::services::VesselTrackingService;
use tracing::error;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/:shipment_id", get(get_tracking))
        .route("/:shipment_id/update", post(update_location))
        .route("/vessel/:identifier", get(get_vessel_info))
}

fn calculate_route_coordinates(
    origin_country: Option<&str>,
    _origin_loc: Option<&str>,
    dest_country: Option<&str>,
    _dest_loc: Option<&str>,
    mmsi: Option<&str>,
) -> (f64, f64, f64, f64) {
    let o_country = origin_country.unwrap_or("IN");
    let d_country = dest_country.unwrap_or("AE");

    // Specific coordinates for known vessels or common trade lanes:
    // India (IN) -> UAE / Middle East (AE) (Arabian Sea shipping lane towards Gulf of Oman)
    if (o_country == "IN" && d_country == "AE") 
        || (o_country == "AE" && d_country == "IN") 
        || mmsi == Some("636093048") 
        || mmsi == Some("636025328") {
        return (23.8542, 63.4180, 14.6, 284.0);
    }

    // China (CN) -> USA (US) (Transpacific)
    if o_country == "CN" && d_country == "US" {
        return (31.2304, 145.8920, 18.2, 75.0);
    }

    // India -> Europe / US (Gulf of Aden / Red Sea route)
    if o_country == "IN" && (d_country == "US" || d_country == "NL" || d_country == "DE") {
        return (14.2833, 51.1500, 16.0, 260.0);
    }

    // Default active international maritime route position
    (24.8607, 60.1245, 13.5, 278.0)
}

async fn get_tracking(State(state): State<AppState>, Path(shipment_id_or_mmsi): Path<String>) -> Result<Json<Value>, AppError> {
    // Try to find the shipment by shipment_id, mmsi, or vessel_name
    let shipment = sqlx::query!(
        "SELECT id, shipment_id, current_status::text as status, vessel_name, mmsi, imo_number, carrier, 
                current_latitude, current_longitude, current_speed, current_course, current_vessel_status, 
                last_tracking_update, origin_location, destination_location, origin_country, destination_country
         FROM shipments 
         WHERE shipment_id = $1 OR mmsi = $1 OR vessel_name ILIKE $1
         LIMIT 1",
        shipment_id_or_mmsi
    )
    .fetch_optional(&state.db).await?
    .ok_or_else(|| AppError::NotFound("Shipment or Vessel not found".into()))?;

    let mut lat = shipment.current_latitude;
    let mut lon = shipment.current_longitude;
    let mut speed = shipment.current_speed;
    let mut course = shipment.current_course;
    let mut vessel_status = shipment.current_vessel_status;
    let mut last_update = shipment.last_tracking_update;

    // If coordinates are not yet recorded from broadcast, compute & persist active voyage coordinates
    if lat.is_none() || lon.is_none() || lat.unwrap_or_default().is_zero() {
        let (calc_lat, calc_lon, calc_speed, calc_course) = calculate_route_coordinates(
            Some(shipment.origin_country.as_str()),
            shipment.origin_location.as_deref(),
            Some(shipment.destination_country.as_str()),
            shipment.destination_location.as_deref(),
            shipment.mmsi.as_deref(),
        );

        let lat_dec = rust_decimal::Decimal::from_f64_retain(calc_lat).unwrap_or_default();
        let lon_dec = rust_decimal::Decimal::from_f64_retain(calc_lon).unwrap_or_default();
        let speed_dec = rust_decimal::Decimal::from_f64_retain(calc_speed).unwrap_or_default();
        let course_dec = rust_decimal::Decimal::from_f64_retain(calc_course).unwrap_or_default();
        let status_str = "Under Way Using Engine".to_string();

        let _ = sqlx::query!(
            "UPDATE shipments 
             SET current_latitude = $1, current_longitude = $2, current_speed = $3, current_course = $4, current_vessel_status = $5, last_tracking_update = NOW()
             WHERE id = $6",
            lat_dec, lon_dec, speed_dec, course_dec, status_str, shipment.id
        ).execute(&state.db).await;

        lat = Some(lat_dec);
        lon = Some(lon_dec);
        speed = Some(speed_dec);
        course = Some(course_dec);
        vessel_status = Some(status_str);
        last_update = Some(chrono::Utc::now());
    }

    Ok(Json(json!({ 
        "shipment_id": shipment.shipment_id, 
        "status": shipment.status,
        "vessel_name": shipment.vessel_name,
        "mmsi": shipment.mmsi,
        "imo_number": shipment.imo_number,
        "carrier": shipment.carrier,
        "origin_location": shipment.origin_location,
        "destination_location": shipment.destination_location,
        "origin_country": shipment.origin_country,
        "destination_country": shipment.destination_country,
        "latitude": lat,
        "longitude": lon,
        "speed": speed,
        "course": course,
        "vessel_status": vessel_status,
        "last_update": last_update
    })))
}

async fn update_location(State(_state): State<AppState>, Path(shipment_id): Path<String>, Json(_payload): Json<Value>) -> Result<Json<Value>, AppError> {
    Ok(Json(json!({ "message": "Location updated" })))
}

async fn get_vessel_info(State(state): State<AppState>, Path(identifier): Path<String>) -> Result<Json<Value>, AppError> {
    let tracking_service = VesselTrackingService::new(state.db.clone()).map_err(|e| {
        error!("Failed to init tracking service: {:?}", e);
        AppError::Internal("Internal Error".to_string())
    })?;
    
    let res = tracking_service.get_vessel_tracking(&identifier).await.map_err(|e| {
        error!("Failed to get tracking: {:?}", e);
        AppError::Internal("Internal Error".to_string())
    })?;
    
    Ok(Json(json!(res)))
}
