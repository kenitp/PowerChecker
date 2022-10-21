
use std::thread;
use std::time::Duration;

// use std::error::Error;

use serde_json::json;

use axum::{
    http::StatusCode,
    response::IntoResponse, 
    routing::get,
    Json,
    Router,
};

mod config;
mod bp35c2;

static mut POWER_W: u32 = 0;
static mut POWER_A: f64 = 0.0;

#[tokio::main]
async fn main() {

    let mut port = bp35c2::ctrl::init_bp35c2(config::DEVICE_PATH, config::B_ROUTE_ID, config::B_ROUTE_PASS).unwrap();
    let meter_info = bp35c2::ctrl::scan_meter(&mut port);
    bp35c2::ctrl::connect_meter(&mut port, &meter_info);

    thread::spawn(move || {
        loop {
            let mut freq_sec = config::GET_FREQ_SEC;
            match bp35c2::ctrl::read_power_w(&mut port, &meter_info) {
                Ok(v) => unsafe{ POWER_W = v },
                Err(_) => freq_sec = 1
            }
            match bp35c2::ctrl::read_power_a(&mut port, &meter_info) {
                Ok(v) => unsafe { POWER_A = v },
                Err(_) => freq_sec = 1
            }
            println!();
            unsafe {
                println!("Power: {} W / {} A", POWER_W, POWER_A);
            }
            std::thread::sleep(Duration::from_secs(freq_sec));
        }
    });

    let api_url: &str = &(config::SERVER_IP.to_string() + &":".to_string() + &config::SERVER_PORT.to_string());
    let app = Router::new().route(config::API_PATH, get(handler_get_power));
    println!("Start REST API: http://{}{}", api_url, config::API_PATH);
    axum::Server::bind(&api_url.parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler_get_power() -> impl IntoResponse {
    unsafe {
        (
            StatusCode::OK,
            Json(json!({
                "power_w": POWER_W.to_string(),
                "power_a": format!("{:.*}", 1, POWER_A)
            })),
        )
    }    
}

