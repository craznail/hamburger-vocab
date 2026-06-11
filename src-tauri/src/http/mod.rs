// HTTP client for server communication (reserved).
// Will be implemented when server-side features are needed.

#[allow(dead_code)]
pub struct ApiClient {
    base_url: String,
    client: reqwest::Client,
    token: Option<String>,
}

#[allow(dead_code)]
impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: reqwest::Client::new(),
            token: None,
        }
    }

    pub fn set_token(&mut self, token: String) {
        self.token = Some(token);
    }

    pub fn clear_token(&mut self) {
        self.token = None;
    }
}
