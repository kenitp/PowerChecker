use std::env;

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

pub const GET_FREQ_SEC_POWER: u64 = 60;
pub const GET_FREQ_SEC_SB_METER: u64 = 30;
pub const RETRY_FREQ_SEC: u64 = 10;

// Home Assistant の Smart Meter B Route 統合が公開するエンティティ
pub const HA_ENTITY_POWER_W: &str = "sensor.smart_meter_power";
pub const HA_ENTITY_CURRENT_R: &str = "sensor.smart_meter_current_r";
pub const HA_ENTITY_CURRENT_T: &str = "sensor.smart_meter_current_t";

pub fn ha_base_url() -> String {
    env::var("HA_BASE_URL").unwrap_or_else(|_| "http://host.docker.internal:8123".to_string())
}

// 機密情報・デバイス識別子は環境変数から取得
pub fn ha_token() -> String {
    env::var("HA_TOKEN").expect("HA_TOKEN must be set")
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
