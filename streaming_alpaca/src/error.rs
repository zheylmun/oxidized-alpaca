use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Alpaca error: {0}")]
    CommonAlpaca(#[from] common_alpaca::Error),
    #[error("Websocket error: {0}")]
    Tungstenite(#[from] tokio_tungstenite::tungstenite::Error),
}
