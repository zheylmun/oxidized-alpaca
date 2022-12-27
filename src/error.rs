use reqwest::Error as ReqwestError;
use snafu::Snafu;
use tokio_tungstenite::tungstenite::Error as TungsteniteError;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub))]
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
    #[snafu(display("Required environment variable not set: {}", "variable_name"))]
    MissingEnvironmentVariable {
        variable_name: String,
        source: std::env::VarError,
    },
    /// Reqwest Send Error
    #[snafu(display("Reqwest send error: {}", "source"))]
    ReqwestSend { source: ReqwestError },
    /// Reqwest Deserialize Error
    #[snafu(display("Reqwest decoding error: {}", "source"))]
    ReqwestDeserialize { source: ReqwestError },

    /// Tungstenite connection error
    #[snafu(display("Tungstenite connection error: {}", "source"))]
    TungsteniteConnection { source: TungsteniteError },

    /// StreamingAuth error
    #[snafu(display("StreamingAuth error"))]
    StreamingAuth {},
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
