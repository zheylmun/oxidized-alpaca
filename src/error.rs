use reqwest::Error as ReqwestError;
use thiserror::Error;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;

#[derive(Debug, Error)]
pub enum Error {
    /// Oxidized Alpaca requires the following environment variables to be set:
    ///
    /// ### Paper Trading:
    /// - `ALPACA_PAPER_API_KEY_ID`
    /// - `ALPACA_PAPER_API_SECRET_KEY`
    ///
    /// ### Live Trading:
    /// - `ALPACA_LIVE_API_KEY_ID`
    /// - `ALPACA_LIVE_API_SECRET_KEY`
    #[error("Required environment variable not set: {}", variable_name)]
    MissingEnvironmentVariable {
        variable_name: String,
        #[source]
        source: std::env::VarError,
    },
    /// Reqwest Send Error
    #[error("Reqwest send error: {}", "source")]
    ReqwestSend(#[source] ReqwestError),
    /// Reqwest Deserialize Error
    #[error("Reqwest decoding error: {}", 0)]
    ReqwestDeserialize(#[source] ReqwestError),

    /// Tungstenite connection error
    #[error("Tungstenite connection error: {}", 0)]
    TungsteniteConnection(#[source] TungsteniteError),

    /// Url Parse Error
    #[error("Url parse error: {}", 0)]
    UrlParse(#[source] url::ParseError),

    /// StreamingAuth error
    #[error("Streaming Auth error")]
    StreamingAuth {},
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
