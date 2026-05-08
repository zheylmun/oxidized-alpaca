use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Server control message indicating connection or authentication success.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlMessage {
    /// Connection to the streaming server was successful.
    Connected,
    /// Authentication was successful.
    Authenticated,
}

/// Error codes returned by the streaming API.
#[derive(Clone, Debug, Deserialize_repr, Serialize_repr)]
#[repr(u16)]
pub enum StreamErrorCode {
    /// The request had invalid syntax.
    InvalidSyntax = 400,
    /// The client is not authenticated.
    NotAuthenticated = 401,
    /// Authentication credentials were rejected.
    AuthFailed = 402,
    /// The client is already authorized.
    AlreadyAuthorized = 403,
}

/// Error message from the streaming API.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StreamError {
    /// Error code indicating the type of error.
    pub code: StreamErrorCode,
    /// Human-readable error message.
    #[serde(rename = "msg")]
    pub message: String,
}

/// Outgoing wire-protocol message used by streaming clients to talk to
/// Alpaca. Generic over the per-feed subscription list type.
///
/// Public only so that it can appear in the [`StreamProtocol::Codec`]
/// associated type. Callers should use the client's
/// `add_subscriptions` / `remove_subscriptions` methods instead of
/// constructing these by hand.
///
/// [`StreamProtocol::Codec`]: crate::streaming::StreamProtocol::Codec
#[doc(hidden)]
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action")]
pub enum Request<T> {
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
