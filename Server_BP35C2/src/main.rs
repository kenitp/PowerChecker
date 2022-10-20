
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
mod command;

static mut POWER_W: u32 = 0;
static mut POWER_A: f64 = 0.0;

#[tokio::main]
async fn main() {
    let mut meter_info = command::MeterInfo {
        channel: String::new(),
        pan_id: String::new(),
        meter_mac_addr: String::new(),
        meter_ip6_addr: String::new(),
        event20: false,
        event22: false,
    };
    let mut port = match command::init_serial_io(config::DEVICE_PATH) {
        Ok(port) => port,
        Err(e) => {
            println!("err value = {}", e);
            return;
        },
    };
    println!("Start Initialize Sequence");
    command::send_command(&mut port, "SKRESET", 1000, true).unwrap();
    let resp = command::send_command(&mut port, "SKVER", 100, true).unwrap();
    match command::wait_resp_command_ok(&resp) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    let resp = command::send_command(&mut port, "SKINFO", 100, true).unwrap();
    match command::wait_resp_command_ok(&resp) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    let resp = command::send_command(&mut port, "SKAPPVER", 100, true).unwrap();
    match command::wait_resp_command_ok(&resp) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    let resp = command::send_command(&mut port, &("SKSETRBID ".to_string() + &config::B_ROUTE_ID.to_string()), 100, true).unwrap();
    match command::wait_resp_command_ok(&resp) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    let resp = command::send_command(&mut port, &("SKSETPWD c ".to_string() + &config::B_ROUTE_PASS.to_string()), 100, true).unwrap();
    match command::wait_resp_command_ok(&resp) {
        Ok(_) => (),
        Err(e) => println!("err value = {}", e),
    }
    loop {
        let resp = command::send_command(&mut port, "SKSCAN 2 FFFFFFFF 6 0", 20000, true).unwrap();
        meter_info = command::wait_resp_event20(&resp, &mut meter_info);
        if meter_info.event20 == true{
            break;
        }
    }
    command::send_command(&mut port, &("SKSREG S2 ".to_string() + &meter_info.channel), 100, true).unwrap();
    command::send_command(&mut port, &("SKSREG S3 ".to_string() + &meter_info.pan_id), 100, true).unwrap();
    let resp = command::send_command(&mut port, &("SKLL64 ".to_string() + & meter_info.meter_mac_addr), 100, true).unwrap();
    meter_info.meter_ip6_addr = resp[1].to_string();
    command::send_command(&mut port, &("SKJOIN ".to_string() + &meter_info.meter_ip6_addr), 300, false).unwrap();
    command::wait_resp_event25(&mut port);

    thread::spawn(move || {
        loop {
            let mut freq_sec = config::GET_FREQ_SEC;
            match command::send_echonet_udp(&mut port, &meter_info.meter_ip6_addr, &command::GET_POWER_W) {
                Ok(_) => {
                    match command::wait_resp_erxudp_w(&mut port, 10000) {
                        Ok(v) => unsafe{POWER_W = v},
                        Err(_) => { freq_sec = 1 }
                    }
                },
                Err(e) => {
                    println!("err value = {}", e);
                    freq_sec = 1;
                }
            }
            match command::send_echonet_udp(&mut port, &meter_info.meter_ip6_addr, &command::GET_POWER_A) {
                Ok(_) => {
                    match command::wait_resp_erxudp_a(&mut port, 10000) {
                        Ok(v) => unsafe {POWER_A = v},
                        Err(_) => { freq_sec = 1 }
                    }
                }
                Err(e) => {
                    println!("err value = {}", e);
                    freq_sec = 1;
                }
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

