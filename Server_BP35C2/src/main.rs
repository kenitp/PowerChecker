use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::Duration;

use axum::{routing::get, Router};

use crate::switchbot::meter::Meter;

mod api;
mod config;
mod homeassistant;
mod switchbot;

static POWER_W: AtomicU32 = AtomicU32::new(0);
static POWER_A: AtomicU64 = AtomicU64::new(0);

static HUMIDITY: AtomicU32 = AtomicU32::new(0);
static TEMPERATURE: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let ha = homeassistant::Client::new(config::ha_base_url(), config::ha_token());
    tokio::spawn(async move {
        loop {
            let freq_sec = match read_power(&ha).await {
                Ok((power_w, power_a)) => {
                    POWER_W.store(power_w, Ordering::Relaxed);
                    POWER_A.store(power_a.to_bits(), Ordering::Relaxed);
                    println!();
                    println!("Power: {} W / {} A", power_w, power_a);
                    config::GET_FREQ_SEC_POWER
                }
                Err(e) => {
                    eprintln!("ERROR: failed to read smart meter: {e}");
                    config::RETRY_FREQ_SEC
                }
            };
            tokio::time::sleep(Duration::from_secs(freq_sec)).await;
        }
    });

    let switchbot_meter_devid = config::switchbot_meter_devid();
    let switchbot_token = config::switchbot_token();
    let switchbot_secret = config::switchbot_secret();
    tokio::spawn(async move {
        loop {
            let mut freq_sec = config::GET_FREQ_SEC_SB_METER;
            match switchbot::meter::get_meter_status(
                &switchbot_meter_devid,
                &switchbot_token,
                &switchbot_secret,
            )
            .await
            {
                Ok(v) => {
                    let meter: Meter = *v;
                    HUMIDITY.store(meter.body.humidity, Ordering::Relaxed);
                    TEMPERATURE.store(meter.body.temperature.to_bits(), Ordering::Relaxed);
                }
                Err(_) => freq_sec = config::RETRY_FREQ_SEC,
            }
            println!();
            println!(
                "Meter: {} ℃ / {} %",
                f64::from_bits(TEMPERATURE.load(Ordering::Relaxed)),
                HUMIDITY.load(Ordering::Relaxed)
            );
            tokio::time::sleep(Duration::from_secs(freq_sec)).await;
        }
    });

    let api_url = format!("{}:{}", config::server_ip(), config::server_port());
    let app = Router::new().route(config::API_PATH, get(api::handler_get_power));
    println!("Start REST API: http://{}{}", api_url, config::API_PATH);

    let listener = tokio::net::TcpListener::bind(&api_url).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

/// 瞬時電力(W)と R/T 相の平均電流(A)を取得する
async fn read_power(ha: &homeassistant::Client) -> Result<(u32, f64), homeassistant::Error> {
    let power_w = ha.state(config::HA_ENTITY_POWER_W).await?;
    let current_r = ha.state(config::HA_ENTITY_CURRENT_R).await?;
    let current_t = ha.state(config::HA_ENTITY_CURRENT_T).await?;

    Ok((power_w.round() as u32, (current_r + current_t) / 2.0))
}
