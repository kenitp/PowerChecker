use std::sync::atomic::Ordering;

use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;

use crate::{HUMIDITY, POWER_A, POWER_W, TEMPERATURE};

pub async fn handler_get_power() -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(json!({
            "power_w": POWER_W.load(Ordering::Relaxed).to_string(),
            "power_a": format!("{:.*}", 1, f64::from_bits(POWER_A.load(Ordering::Relaxed))),
            "temperature": format!("{:.*}", 1, f64::from_bits(TEMPERATURE.load(Ordering::Relaxed))),
            "humidity": HUMIDITY.load(Ordering::Relaxed).to_string()
        })),
    )
}
