use std::env;
use std::path::PathBuf;

pub const DEVICE_PATH: &str = "/dev/ttyUSB_power";
pub fn server_ip() -> String {
    env::var("SERVER_IP").unwrap_or_else(|_| "192.168.1.110".to_string())
}
pub fn server_port() -> i32 {
    env::var("SERVER_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(3000)
}
pub const API_PATH: &str = "/api/power";
pub const API_HISTORY_PATH: &str = "/api/power/history";

pub const GET_FREQ_SEC_POWER: u64 = 60;
pub const GET_FREQ_SEC_SB_METER: u64 = 30;

pub fn db_path() -> PathBuf {
    env::var("DB_PATH")
        .unwrap_or_else(|_| "data/power.db".to_string())
        .into()
}

// 機密情報・デバイス識別子は環境変数から取得
pub fn b_route_id() -> String {
    env::var("B_ROUTE_ID").expect("B_ROUTE_ID must be set")
}

pub fn b_route_pass() -> String {
    env::var("B_ROUTE_PASS").expect("B_ROUTE_PASS must be set")
}

pub fn switchbot_token() -> String {
    env::var("SWITCHBOT_TOKEN").expect("SWITCHBOT_TOKEN must be set")
}

pub fn switchbot_secret() -> String {
    env::var("SWITCHBOT_SECRET").expect("SWITCHBOT_SECRET must be set")
}

pub fn switchbot_meter_devid() -> String {
    env::var("SWITCHBOT_METER_DEVID").expect("SWITCHBOT_METER_DEVID must be set")
}
