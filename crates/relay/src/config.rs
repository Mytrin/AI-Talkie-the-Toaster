use anyhow::Context;

pub struct Config {
    /// HTTP health listener.
    pub http_addr: String,
    /// Raw-TCP listener the board connects to (dumb PCM protocol).
    pub device_addr: String,
    pub gemini_api_key: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            http_addr: std::env::var("HTTP_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".into()),
            device_addr: std::env::var("DEVICE_ADDR").unwrap_or_else(|_| "0.0.0.0:4000".into()),
            gemini_api_key: std::env::var("GEMINI_API_KEY")
                .context("GEMINI_API_KEY must be set")?,
        })
    }
}
