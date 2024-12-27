pub enum ClientState<S> {
    Connecting,
    Connected,
    Authenticated(S),
}
