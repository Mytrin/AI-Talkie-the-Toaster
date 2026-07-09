//! Adapter over the Google Gemini Live API (wss:// + JSON + base64 PCM).
//! Lives on the relay so the board never touches TLS/WebSocket.
//! https://ai.google.dev/gemini-api/docs/live-api

pub struct Client {
    http: reqwest::Client,
    api_key: String,
}

impl Client {
    pub fn new(api_key: String) -> anyhow::Result<Self> {
        Ok(Self {
            http: reqwest::Client::new(),
            api_key,
        })
    }
}
