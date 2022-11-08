#![allow(non_snake_case)]
use reqwest;
use serde::Deserialize;
use serde_json;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Body {
    deviceId: String,
    deviceType: String,
    hubDeviceId: String,
    pub humidity: u32,
    pub temperature: f64
}
#[derive(Deserialize, Debug, PartialEq)]
pub struct Meter {
    statusCode: u32,
    pub body: Body,
    message: String,
}

pub async fn get_meter_status(dev_id: &str, token: &str) -> Result<Box<Meter>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let url: String = String::from("https://api.switch-bot.com/v1.0/devices/") + dev_id + "/status";
    let resp = client.get(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json; charset=utf8")
        .header(reqwest::header::AUTHORIZATION, token)
        .send()
        .await?
        .text()
        .await?;

    let meter;
    match json_to_meter(&resp) {
        Ok(v) => {
            meter = Box::new(v);
        }
        Err(e) => {
            println!("{}", e);
            return Err(Box::new(e));
        }
    }
    Ok(meter)
}




pub fn json_to_meter(json: &str) -> Result<Meter, serde_json::Error> {
    serde_json::from_str(json)
}