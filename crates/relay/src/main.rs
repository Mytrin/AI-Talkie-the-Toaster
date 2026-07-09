mod config;
mod device;
mod gemini;

use std::sync::Arc;

use axum::{Router, routing::get};
use tracing_subscriber::EnvFilter;

use crate::config::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env()?;
    let gemini = Arc::new(gemini::Client::new(config.gemini_api_key.clone())?);

    let http = Router::new().route("/health", get(|| async { "ok" }));
    let http_listener = tokio::net::TcpListener::bind(&config.http_addr).await?;
    tracing::info!(addr = %config.http_addr, "http listening");

    let device_listener = tokio::net::TcpListener::bind(&config.device_addr).await?;
    tracing::info!(addr = %config.device_addr, "device (raw tcp) listening");

    let devices = tokio::spawn(accept_devices(device_listener, gemini));
    axum::serve(http_listener, http).await?;
    devices.await??;
    Ok(())
}

async fn accept_devices(
    listener: tokio::net::TcpListener,
    gemini: Arc<gemini::Client>,
) -> anyhow::Result<()> {
    loop {
        let (stream, _) = listener.accept().await?;
        let gemini = gemini.clone();
        tokio::spawn(async move {
            if let Err(e) = device::handle(stream, &gemini).await {
                tracing::error!(error = %e, "device connection ended");
            }
        });
    }
}
