pub mod httpbin;

use std::sync::OnceLock;

use reqwest::Client as ReqwestClient;

/// To be used as singleton for global client
static BASE_CLIENT: OnceLock<Client> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct Client {
    http_client: ReqwestClient,
    api_key: String,
    // config
}

impl Client {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            http_client: ReqwestClient::new(),
            api_key: api_key.into(),
        }
    }
}

/// Global SDK singleton to be initialized before use
pub fn init(api_key: impl Into<String>) {
    BASE_CLIENT
        .set(Client::new(api_key))
        .expect("BaseClient already initialized");
}

/// Base client singleton for use in other clients
pub fn base_client() -> &'static Client {
    BASE_CLIENT
        .get()
        .expect("SDK Client not initialized. Call init() first.")
}
