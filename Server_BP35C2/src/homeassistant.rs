use serde::Deserialize;

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
