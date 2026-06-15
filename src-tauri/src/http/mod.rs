use reqwest::Client;

/// Shared HTTP client for external API requests.
/// Created once at startup and reused across all TTS synthesis calls.
pub struct HttpClientState {
    pub client: Client,
}

impl Default for HttpClientState {
    fn default() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("failed to create HTTP client"),
        }
    }
}
