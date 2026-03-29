pub const API_PATH: &str = "/api/power";
pub const GET_FREQ_SEC_POWER: u64 = 60;
pub const GET_FREQ_SEC_SB_METER: u64 = 30;

pub fn device_path() -> String {
    std::env::var("DEVICE_PATH").expect("DEVICE_PATH must be set in environment")
}

pub fn server_ip() -> String {
    std::env::var("SERVER_IP").expect("SERVER_IP must be set in environment")
}

pub fn server_port() -> u16 {
    std::env::var("SERVER_PORT")
        .expect("SERVER_PORT must be set in environment")
        .parse::<u16>()
        .expect("SERVER_PORT must be a valid port number")
}

pub fn b_route_id() -> String {
    std::env::var("B_ROUTE_ID").expect("B_ROUTE_ID must be set in environment")
}

pub fn b_route_pass() -> String {
    std::env::var("B_ROUTE_PASS").expect("B_ROUTE_PASS must be set in environment")
}

pub fn switchbot_token() -> String {
    std::env::var("SWITCHBOT_TOKEN").expect("SWITCHBOT_TOKEN must be set in environment")
}

pub fn switchbot_meter_devid() -> String {
    std::env::var("SWITCHBOT_METER_DEVID").expect("SWITCHBOT_METER_DEVID must be set in environment")
}
