use serde::Deserialize;

use super::base_client;

#[derive(Debug, Clone)]
pub struct HttpbinClient {
    base_url: String,
}

#[derive(Deserialize)]
struct UuidResponse {
    uuid: String,
}

impl HttpbinClient {
    pub fn new() -> Self {
        Self {
            base_url: "https://httpbin.org".to_string(),
        }
    }

    /// Gets a uuid from httpbin
    pub async fn get_uuid(&self) -> Result<String, Box<dyn std::error::Error>> {
        let uuid: UuidResponse = base_client()
            .http_client
            .get(self.base_url.clone() + "/uuid")
            .send()
            .await?
            .json()
            .await?;

        Ok(uuid.uuid)
    }
}

impl Default for HttpbinClient {
    fn default() -> Self {
        Self::new()
    }
}
