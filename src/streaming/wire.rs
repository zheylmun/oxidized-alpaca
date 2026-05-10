use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Server control message indicating connection or authentication success.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum ControlMessage {
    /// Connection to the streaming server was successful.
    Connected,
    /// Authentication was successful.
    Authenticated,
}

/// Error codes returned by the streaming API.
///
/// Marked `#[non_exhaustive]` because Alpaca occasionally introduces new
/// codes; rebuild against a newer crate version when a new one appears.
#[derive(Clone, Copy, Debug, Deserialize_repr, Eq, PartialEq, Serialize_repr)]
#[repr(u16)]
#[non_exhaustive]
pub enum StreamErrorCode {
    /// The request had invalid syntax.
    InvalidSyntax = 400,
    /// The client is not authenticated.
    NotAuthenticated = 401,
    /// Authentication credentials were rejected.
    AuthFailed = 402,
    /// The client is already authorized.
    AlreadyAuthorized = 403,
    /// The client did not authenticate within the server's window.
    AuthTimeout = 404,
    /// Subscription would exceed the account's symbol limit.
    SymbolLimitExceeded = 405,
    /// The account already holds the maximum number of concurrent connections.
    ConnectionLimitExceeded = 406,
    /// The client is consuming messages too slowly and is being disconnected.
    SlowClient = 407,
    /// The account's data plan does not include v2 streaming.
    V2NotEnabled = 408,
    /// The account's subscription level does not include this data.
    InsufficientSubscription = 409,
    /// The requested subscribe action is not valid for this feed.
    InvalidSubscribeAction = 410,
    /// The API key does not have the scope required for this subscription.
    InsufficientScope = 411,
}

/// Error message from the streaming API.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct StreamError {
    /// Error code indicating the type of error.
    pub code: StreamErrorCode,
    /// Human-readable error message.
    #[serde(rename = "msg")]
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::{StreamError, StreamErrorCode};

    /// Round-trip every documented Alpaca streaming error code so a typo in
    /// the `repr(u16)` discriminants would be caught at test time. The list
    /// is exhaustive against `StreamErrorCode` as of this commit; the enum
    /// is `#[non_exhaustive]`, so adding a new variant requires extending
    /// this test alongside it.
    #[test]
    fn stream_error_code_round_trip_all_variants() {
        for (code, expected) in [
            (400u16, StreamErrorCode::InvalidSyntax),
            (401, StreamErrorCode::NotAuthenticated),
            (402, StreamErrorCode::AuthFailed),
            (403, StreamErrorCode::AlreadyAuthorized),
            (404, StreamErrorCode::AuthTimeout),
            (405, StreamErrorCode::SymbolLimitExceeded),
            (406, StreamErrorCode::ConnectionLimitExceeded),
            (407, StreamErrorCode::SlowClient),
            (408, StreamErrorCode::V2NotEnabled),
            (409, StreamErrorCode::InsufficientSubscription),
            (410, StreamErrorCode::InvalidSubscribeAction),
            (411, StreamErrorCode::InsufficientScope),
        ] {
            let json = code.to_string();
            let decoded: StreamErrorCode = serde_json::from_str(&json)
                .unwrap_or_else(|e| panic!("failed to decode {code}: {e}"));
            assert_eq!(decoded, expected, "code {code} decoded to wrong variant");
            let reencoded = serde_json::to_string(&decoded).unwrap();
            assert_eq!(reencoded, json, "code {code} re-encoded to wrong number");
        }
    }

    #[test]
    fn stream_error_decodes_from_wire_payload() {
        // Mirrors what the per-feed message envelopes deliver after the
        // outer `{"T":"error", ...}` tag is consumed by serde.
        let json = r#"{"code":410,"msg":"invalid subscribe action"}"#;
        let err: StreamError = serde_json::from_str(json).unwrap();
        assert_eq!(err.code, StreamErrorCode::InvalidSubscribeAction);
        assert_eq!(err.message, "invalid subscribe action");
    }
}

/// Outgoing wire-protocol message used internally by the streaming
/// clients to talk to Alpaca. Crate-private; callers reach the same
/// behaviour through the client's `add_subscriptions` /
/// `remove_subscriptions` methods.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action")]
pub(crate) enum Request<T> {
    /// Authenticate with API key and secret.
    #[serde(rename = "auth")]
    AuthMessage {
        /// API key ID.
        key: String,
        /// API secret key.
        secret: String,
    },
    /// Subscribe to streaming channels.
    #[serde(rename = "subscribe")]
    Subscribe(T),
    /// Unsubscribe from streaming channels.
    #[serde(rename = "unsubscribe")]
    Unsubscribe(T),
}
