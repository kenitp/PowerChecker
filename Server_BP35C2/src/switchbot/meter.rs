#![allow(non_snake_case)]
use base64::{engine::general_purpose::STANDARD, Engine};
use hmac::{Hmac, Mac};
use reqwest;
use serde::Deserialize;
use serde_json;
use sha2::Sha256;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize, Debug, PartialEq)]
pub struct Body {
    deviceId: String,
    deviceType: String,
    hubDeviceId: String,
    pub humidity: u32,
    pub temperature: f64,
}

#[derive(Deserialize, Debug, PartialEq)]
pub struct Meter {
    statusCode: u32,
    pub body: Body,
    message: String,
}

/// SwitchBot API v1.1用の認証ヘッダーを生成
fn generate_auth_headers(token: &str, secret: &str) -> (String, String, String) {
    // タイムスタンプ（ミリ秒）
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string();

    // nonce（UUID v4）
    let nonce = Uuid::new_v4().to_string();

    // 署名の生成: sign = Base64(HMAC-SHA256(secret, token + t + nonce))
    let string_to_sign = format!("{}{}{}", token, t, nonce);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(string_to_sign.as_bytes());
    let sign = STANDARD.encode(mac.finalize().into_bytes());

    (t, nonce, sign)
}

pub async fn get_meter_status(
    dev_id: &str,
    token: &str,
    secret: &str,
) -> Result<Box<Meter>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    // v1.1 エンドポイントを使用
    let url: String =
        String::from("https://api.switch-bot.com/v1.1/devices/") + dev_id + "/status";

    // 認証ヘッダーを生成
    let (t, nonce, sign) = generate_auth_headers(token, secret);

    let resp = client
        .get(&url)
        .header(reqwest::header::CONTENT_TYPE, "application/json; charset=utf8")
        .header(reqwest::header::AUTHORIZATION, token)
        .header("sign", sign)
        .header("t", t)
        .header("nonce", nonce)
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
            println!("JSON parse error: {}", e);
            println!("Response: {}", resp);
            return Err(Box::new(e));
        }
    }
    Ok(meter)
}

pub fn json_to_meter(json: &str) -> Result<Meter, serde_json::Error> {
    serde_json::from_str(json)
}
