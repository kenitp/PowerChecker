use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use axum::{routing::get, Router};

use crate::switchbot::meter::Meter;

mod api;
mod bp35c2;
mod config;
mod store;
mod switchbot;

static POWER_W: AtomicU32 = AtomicU32::new(0);
static POWER_A: AtomicU64 = AtomicU64::new(0);

static HUMIDITY: AtomicU32 = AtomicU32::new(0);
static TEMPERATURE: AtomicU64 = AtomicU64::new(0);

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let store = Arc::new(
        store::Store::open(config::db_path()).expect("failed to open power history database"),
    );
    let store_writer = Arc::clone(&store);

    let b_route_id = config::b_route_id();
    let b_route_pass = config::b_route_pass();

    let mut port =
        bp35c2::ctrl::init_bp35c2(config::DEVICE_PATH, &b_route_id, &b_route_pass).unwrap();
    let meter_info = bp35c2::ctrl::scan_meter(&mut port);
    bp35c2::ctrl::connect_meter(&mut port, &meter_info);

    thread::spawn(move || {
        let mut err_count: u32 = 0;
        loop {
            let mut freq_sec = config::GET_FREQ_SEC_POWER;
            let mut err_flg = false;
            let mut power_w_ok = None;
            let mut power_a_ok = None;

            match bp35c2::ctrl::read_power_w(&mut port, &meter_info) {
                Ok(v) => {
                    POWER_W.store(v, Ordering::Relaxed);
                    power_w_ok = Some(v);
                }
                Err(_) => {
                    freq_sec = 1;
                    err_flg = true;
                }
            }

            thread::sleep(Duration::from_millis(1000));

            match bp35c2::ctrl::read_power_a(&mut port, &meter_info) {
                Ok(v) => {
                    POWER_A.store(v.to_bits(), Ordering::Relaxed);
                    power_a_ok = Some(v);
                }
                Err(_) => {
                    freq_sec = 1;
                    err_flg = true;
                }
            }

            println!();
            println!(
                "Power: {} W / {} A",
                POWER_W.load(Ordering::Relaxed),
                f64::from_bits(POWER_A.load(Ordering::Relaxed))
            );

            if let Some(power_w) = power_w_ok {
                if let Err(e) = store_writer.insert(power_w, power_a_ok) {
                    println!("ERROR: failed to store sample: {e}");
                }
            }

            match err_flg {
                true => {
                    err_count += 1;
                    println!("Error occurred, incrementing error count to: {}", err_count);
                }
                false => {
                    if err_count > 0 {
                        println!(
                            "Successful reading, resetting error count from: {}",
                            err_count
                        );
                    }
                    err_count = 0;
                }
            }

            if 5 < err_count {
                println!(
                    "ERROR: Too many consecutive errors ({}), attempting to reconnect...",
                    err_count
                );
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    bp35c2::ctrl::connect_meter(&mut port, &meter_info);
                })) {
                    Ok(_) => println!("Reconnection completed"),
                    Err(_) => println!("ERROR: Reconnection failed with panic"),
                }
            }

            thread::sleep(Duration::from_secs(freq_sec));
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
                Err(_) => freq_sec = 1,
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
    let app = Router::new()
        .route(config::API_PATH, get(api::handler_get_power))
        .route(config::API_HISTORY_PATH, get(api::handler_get_history))
        .with_state(store);
    println!("Start REST API: http://{}{}", api_url, config::API_PATH);
    println!(
        "Start REST API: http://{}{}",
        api_url,
        config::API_HISTORY_PATH
    );

    let listener = tokio::net::TcpListener::bind(&api_url).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
