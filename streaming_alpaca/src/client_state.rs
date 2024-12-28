#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ClientState<S> {
    Connecting,
    Connected,
    Authenticated(S),
}
