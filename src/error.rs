#[cfg(feature = "restful")]
use reqwest::Error as ReqwestError;
use thiserror::Error;

/// Errors that can occur when using the Alpaca API client.
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
        /// Name of the missing environment variable.
        variable_name: String,
        /// The underlying `VarError`.
        #[source]
        source: std::env::VarError,
    },
    /// Reqwest Send Error
    #[cfg(feature = "restful")]
    #[error("Reqwest send error: {0}")]
    ReqwestSend(#[source] ReqwestError),
    /// Reqwest Deserialize Error
    #[cfg(feature = "restful")]
    #[error("Reqwest decoding error: {0}")]
    ReqwestDeserialize(#[source] ReqwestError),

    /// API returned a non-2xx status code
    #[error("API error (HTTP {}): {}", status, body)]
    ApiError {
        /// HTTP status code.
        status: u16,
        /// Response body text.
        body: String,
    },

    /// Socketeer connection error
    #[cfg(feature = "streaming")]
    #[error("Socketeer websocket error: {0}")]
    WebsocketError(#[from] socketeer::Error),

    /// Url Parse Error
    #[error("Url parse error: {0}")]
    UrlParse(#[source] url::ParseError),
    /// Unexpected connection message
    #[error("Unexpected connection message: {0}")]
    UnexpectedConnectionMessage(String),
    /// StreamingAuth error
    #[error("Streaming Auth error")]
    StreamingAuth,
}

/// A `Result` type alias using [`Error`] as the default error type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn url_parse_display_includes_inner_cause() {
        let inner = url::Url::parse("not a url").unwrap_err();
        let inner_text = inner.to_string();
        let err = Error::UrlParse(inner);
        let rendered = err.to_string();
        assert!(
            rendered.contains(&inner_text),
            "expected `{rendered}` to include the inner cause `{inner_text}`",
        );
    }
}
