#[cfg(feature = "restful")]
use reqwest::Error as ReqwestError;
use thiserror::Error;

/// Opaque error returned by the streaming WebSocket transport.
///
/// The crate uses [`socketeer`] internally, but its concrete error type
/// is not exposed so the underlying dependency can be swapped out
/// without a breaking release. Use [`std::error::Error::source`] to
/// inspect the chain when diagnosing failures.
#[cfg(feature = "streaming")]
#[derive(Debug)]
pub struct WebsocketError(socketeer::Error);

#[cfg(feature = "streaming")]
impl std::fmt::Display for WebsocketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(feature = "streaming")]
impl std::error::Error for WebsocketError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

#[cfg(feature = "streaming")]
impl From<socketeer::Error> for WebsocketError {
    fn from(value: socketeer::Error) -> Self {
        Self(value)
    }
}

#[cfg(feature = "streaming")]
impl From<socketeer::Error> for Error {
    fn from(value: socketeer::Error) -> Self {
        Self::Websocket(WebsocketError(value))
    }
}

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

    /// Streaming WebSocket transport error.
    #[cfg(feature = "streaming")]
    #[error("websocket error: {0}")]
    Websocket(#[from] WebsocketError),

    /// Url Parse Error
    #[error("Url parse error: {0}")]
    UrlParse(#[source] url::ParseError),
    /// Unexpected connection message
    #[error("Unexpected connection message: {0}")]
    UnexpectedConnectionMessage(String),
    /// StreamingAuth error
    #[error("Streaming Auth error")]
    StreamingAuth,
    /// A time-frame multiplier was outside the documented valid range.
    #[cfg(feature = "restful")]
    #[error("invalid timeframe: {amount}{unit} is outside the valid range {valid_range}")]
    InvalidTimeFrame {
        /// Unit of the rejected time-frame.
        unit: crate::restful::market_data::TimeFrameUnit,
        /// Multiplier that was rejected.
        amount: u16,
        /// Human-readable description of the valid range.
        valid_range: &'static str,
    },
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

    #[cfg(feature = "streaming")]
    #[test]
    fn websocket_error_display_includes_inner_cause() {
        let inner = socketeer::Error::WebsocketClosed;
        let inner_text = inner.to_string();
        let err: Error = inner.into();
        let rendered = err.to_string();
        assert!(
            rendered.starts_with("websocket error:"),
            "expected `{rendered}` to be tagged with the websocket error prefix",
        );
        assert!(
            rendered.contains(&inner_text),
            "expected `{rendered}` to surface the inner cause `{inner_text}`",
        );
    }

    #[cfg(feature = "streaming")]
    #[test]
    fn websocket_error_preserves_source_chain_through_opaque_wrapper() {
        use std::error::Error as _;

        let url_err = url::Url::parse("not a url").unwrap_err();
        let url_err_text = url_err.to_string();
        let inner = socketeer::Error::UrlParse {
            url: "not a url".to_string(),
            source: url_err,
        };
        let err: Error = inner.into();

        // Top-level source is the opaque WebsocketError facade; the
        // `socketeer::Error` layer is intentionally skipped in the chain
        // so the private dependency stays out of the public API.
        let wrapper = err
            .source()
            .expect("Error::Websocket should expose its WebsocketError as a source");
        let parse_err = wrapper
            .source()
            .expect("WebsocketError forwards source past socketeer::Error");
        let parse_err = parse_err
            .downcast_ref::<url::ParseError>()
            .expect("the deepest source should be the original url::ParseError");
        assert_eq!(parse_err.to_string(), url_err_text);
    }
}
