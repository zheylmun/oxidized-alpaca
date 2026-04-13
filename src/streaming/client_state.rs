/// Connection state of a streaming client.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ClientState<S> {
    /// WebSocket connection is being established.
    Connecting,
    /// Connected but not yet authenticated.
    Connected,
    /// Authenticated and ready, carrying session state.
    Authenticated(S),
}
