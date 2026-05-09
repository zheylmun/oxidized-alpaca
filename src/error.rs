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

    #[test]
    fn missing_environment_variable_display_names_var() {
        let source = std::env::var("__definitely_missing_for_test__").unwrap_err();
        let err = Error::MissingEnvironmentVariable {
            variable_name: "ALPACA_PAPER_API_KEY_ID".to_string(),
            source,
        };
        let rendered = err.to_string();
        assert!(
            rendered.contains("ALPACA_PAPER_API_KEY_ID"),
            "expected `{rendered}` to name the missing variable",
        );
    }

    #[test]
    fn api_error_display_includes_status_and_body() {
        let err = Error::ApiError {
            status: 422,
            body: "symbol not found".to_string(),
        };
        let rendered = err.to_string();
        assert!(
            rendered.contains("422") && rendered.contains("symbol not found"),
            "expected `{rendered}` to surface both status and body",
        );
    }

    #[test]
    fn unexpected_connection_message_display_passes_through_payload() {
        let err = Error::UnexpectedConnectionMessage("Subscription { trades: [] }".to_string());
        let rendered = err.to_string();
        assert!(
            rendered.contains("Subscription { trades: [] }"),
            "expected `{rendered}` to include the wrapped payload",
        );
    }

    #[test]
    fn streaming_auth_display_is_human_readable() {
        let rendered = Error::StreamingAuth.to_string();
        assert!(
            !rendered.is_empty() && rendered.to_lowercase().contains("auth"),
            "expected `{rendered}` to mention auth",
        );
    }
}
