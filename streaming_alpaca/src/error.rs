use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Alpaca error: {0}")]
    CommonAlpaca(#[from] common_alpaca::Error),
    #[error("Socketeer error: {0}")]
    Socketeer(#[from] socketeer::Error),
    #[error("Unexpected Response: {0}")]
    UnexpectedResponse(String),
}
