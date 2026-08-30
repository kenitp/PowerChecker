use std::sync::atomic::Ordering;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::store::{self, HistoryRange, Store};
use crate::{HUMIDITY, POWER_A, POWER_W, TEMPERATURE};

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    pub from: Option<i64>,
    pub to: Option<i64>,
    pub step: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct HistoryPoint {
    pub ts: i64,
    pub power_w: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub power_a: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub from: i64,
    pub to: i64,
    pub step: i64,
    pub points: Vec<HistoryPoint>,
}

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

pub async fn handler_get_history(
    State(store): State<Arc<Store>>,
    Query(query): Query<HistoryQuery>,
) -> impl IntoResponse {
    let range = match store::resolve_range(query.from, query.to, query.step) {
        Ok(range) => range,
        Err(msg) => {
            return (StatusCode::BAD_REQUEST, Json(json!({ "error": msg }))).into_response();
        }
    };

    match tokio::task::spawn_blocking(move || query_history(store, range)).await {
        Ok(Ok(body)) => (StatusCode::OK, Json(body)).into_response(),
        Ok(Err(e)) => {
            eprintln!("ERROR: history query failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "history query failed" })),
            )
                .into_response()
        }
        Err(e) => {
            eprintln!("ERROR: history task join failed: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "history query failed" })),
            )
                .into_response()
        }
    }
}

fn query_history(
    store: Arc<Store>,
    range: HistoryRange,
) -> Result<HistoryResponse, rusqlite::Error> {
    let points = store
        .query(range.clone())?
        .into_iter()
        .map(|sample| HistoryPoint {
            ts: sample.ts,
            power_w: sample.power_w,
            power_a: sample.power_a,
        })
        .collect();

    Ok(HistoryResponse {
        from: range.from,
        to: range.to,
        step: range.step,
        points,
    })
}
