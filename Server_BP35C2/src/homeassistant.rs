use serde::Deserialize;
use serde_json::json;

pub type Error = Box<dyn std::error::Error + Send + Sync>;

#[derive(Deserialize)]
struct StateResponse {
    state: String,
}

pub struct Client {
    http: reqwest::Client,
    base_url: String,
    token: String,
}

impl Client {
    pub fn new(base_url: String, token: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url,
            token,
        }
    }

    /// 対象エンティティの再取得を要求し、完了まで待機する
    pub async fn refresh(&self, entity_ids: &[&str]) -> Result<(), Error> {
        self.http
            .post(format!(
                "{}/api/services/homeassistant/update_entity",
                self.base_url
            ))
            .bearer_auth(&self.token)
            .json(&json!({ "entity_id": entity_ids }))
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }

    pub async fn state(&self, entity_id: &str) -> Result<f64, Error> {
        let response: StateResponse = self
            .http
            .get(format!("{}/api/states/{}", self.base_url, entity_id))
            .bearer_auth(&self.token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        response.state.parse().map_err(|_| {
            format!("{entity_id} is not numeric: \"{}\"", response.state).into()
        })
    }
}
