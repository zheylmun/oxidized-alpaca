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
