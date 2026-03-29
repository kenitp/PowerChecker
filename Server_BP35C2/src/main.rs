
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::thread;
use std::time::Duration;
use tokio::runtime::Runtime;
// use std::error::Error;

use serde_json::json;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json,
    Router,
};

use crate::switchbot::meter::Meter;

mod config;
mod bp35c2;
mod switchbot;

static POWER_W: AtomicU32 = AtomicU32::new(0);
static POWER_A: AtomicU64 = AtomicU64::new(0);

static HUMIDITY: AtomicU32 = AtomicU32::new(0);
static TEMPERATURE: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let device_path = config::device_path();
    let b_route_id = config::b_route_id();
    let b_route_pass = config::b_route_pass();

    let mut port = bp35c2::ctrl::init_bp35c2(&device_path, &b_route_id, &b_route_pass).unwrap();
    let meter_info = bp35c2::ctrl::scan_meter(&mut port);
    bp35c2::ctrl::connect_meter(&mut port, &meter_info);

    thread::spawn(move || {
        let mut err_count: u32 = 0;
        loop {
            let mut freq_sec = config::GET_FREQ_SEC_POWER;
            let mut err_flg = false;
            match bp35c2::ctrl::read_power_w(&mut port, &meter_info) {
                Ok(v) => POWER_W.store(v, Ordering::Relaxed),
                Err(_) => {
                    freq_sec = 1;
                    err_flg = true;
                }
            }
            match bp35c2::ctrl::read_power_a(&mut port, &meter_info) {
                Ok(v) => POWER_A.store(v.to_bits(), Ordering::Relaxed),
                Err(_) => {
                    freq_sec = 1;
                    err_flg = true;
                }
            }
            println!();
            println!("Power: {} W / {} A",
                POWER_W.load(Ordering::Relaxed),
                f64::from_bits(POWER_A.load(Ordering::Relaxed)));
            match err_flg {
                true => err_count += 1,
                false => err_count = 0,
            }
            if 5 < err_count {
                bp35c2::ctrl::connect_meter(&mut port, &meter_info);
                println!("Maybe once disconnected. Re-connecting...");
            }

            std::thread::sleep(Duration::from_secs(freq_sec));
        }
    });

    let rt = Runtime::new().unwrap();
    let switchbot_devid = config::switchbot_meter_devid();
    let switchbot_token = config::switchbot_token();
    rt.spawn(async move {
        loop {
            let mut freq_sec = config::GET_FREQ_SEC_SB_METER;
            match switchbot::meter::get_meter_status(&switchbot_devid, &switchbot_token).await {
                Ok(v) => {
                    let meter: Meter = *v;
                    HUMIDITY.store(meter.body.humidity, Ordering::Relaxed);
                    TEMPERATURE.store(meter.body.temperature.to_bits(), Ordering::Relaxed);
                }
                Err(_) => freq_sec = 1
            }
            println!();
            println!("Meter: {} ℃ / {} %",
                f64::from_bits(TEMPERATURE.load(Ordering::Relaxed)),
                HUMIDITY.load(Ordering::Relaxed));
            std::thread::sleep(Duration::from_secs(freq_sec));
        }
    });

    let api_url = format!("{}:{}", config::server_ip(), config::server_port());
    let app = Router::new().route(config::API_PATH, get(handler_get_power));
    println!("Start REST API: http://{}{}", api_url, config::API_PATH);
    axum::Server::bind(&api_url.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler_get_power() -> impl IntoResponse {
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
