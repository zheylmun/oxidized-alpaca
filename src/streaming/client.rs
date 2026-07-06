use serde::{Serialize, de::DeserializeOwned};
use socketeer::Socketeer;
use std::collections::VecDeque;

use crate::{
    Error,
    env::ApiKey,
    streaming::wire::{ControlMessage, Request, StreamError},
};

macro_rules! info {
    ($($arg:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::info!($($arg)*);
    };
}
macro_rules! error {
    ($($arg:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::error!($($arg)*);
    };
}

pub(crate) mod sealed {
    /// Sealing supertrait for [`super::StreamProtocol`]. Users cannot
    /// implement this trait, so they cannot implement
    /// [`super::StreamProtocol`] either; only the per-feed marker types
    /// shipped by this crate are valid protocols.
    pub trait Sealed {}
}

/// Per-feed protocol description for [`StreamingClient`].
///
/// Each Alpaca streaming feed (stock, crypto, news, options, …) uses
/// the same WebSocket envelope but has its own message enum and
/// subscription-list shape. The per-feed marker types shipped by this
/// crate (`StockProtocol`, `CryptoProtocol`, `NewsProtocol`,
/// `OptionProtocol`) implement `StreamProtocol` to wire those up;
/// callers reach the corresponding clients through the
/// `StreamingStockClient` / `StreamingCryptoClient` / … type aliases.
///
/// This trait is sealed and cannot be implemented outside the crate.
pub trait StreamProtocol: sealed::Sealed + 'static {
    /// The full message enum delivered by this feed (including control,
    /// error, and subscription variants).
    type Message: DeserializeOwned + Send + Clone + std::fmt::Debug + 'static;
    /// The per-feed subscription list (e.g. `StockSubscriptionList`).
    type Subscriptions: Serialize
        + DeserializeOwned
        + Default
        + Clone
        + Send
        + std::fmt::Debug
        + 'static;

    /// Return the embedded [`ControlMessage`] when `message` is a
    /// `success` envelope (Connected / Authenticated). Used to drive
    /// the auth handshake.
    fn control(message: &Self::Message) -> Option<&ControlMessage>;

    /// Return the embedded [`StreamError`] when `message` is an `error`
    /// envelope. Used to surface a rejected subscribe/unsubscribe instead
    /// of blocking forever waiting for a confirmation that never arrives.
    fn stream_error(message: &Self::Message) -> Option<&StreamError>;

    /// If `message` is a subscription confirmation, return the updated
    /// subscription list. Otherwise return the message unchanged so the
    /// caller can surface it.
    fn take_subscription_update(
        message: Self::Message,
    ) -> Result<Self::Subscriptions, Self::Message>;
}

/// Crate-private extension of [`StreamProtocol`] that names the
/// `socketeer` codec for each feed. Kept off the public trait so the
/// `socketeer` dependency stays an implementation detail — callers
/// using the streaming clients never see it in any public signature.
pub(crate) trait StreamProtocolCodec: StreamProtocol {
    type Codec: socketeer::Codec<Tx = Request<Self::Subscriptions>, Rx = Vec<Self::Message>>
        + Default;
}

type StreamSocket<P> = Socketeer<<P as StreamProtocolCodec>::Codec>;

/// Generic streaming-feed client shared by every Alpaca WebSocket data feed.
///
/// Construct one of the per-feed type aliases (`StreamingStockClient`,
/// `StreamingCryptoClient`, `StreamingNewsClient`, `StreamingOptionClient`)
/// rather than instantiating this directly.
#[derive(Debug)]
#[allow(
    private_bounds,
    reason = "StreamProtocolCodec is intentionally sealed — only this crate's marker types can satisfy it, which keeps the socketeer dependency an implementation detail."
)]
pub struct StreamingClient<P: StreamProtocol + StreamProtocolCodec> {
    websocket: StreamSocket<P>,
    messages: VecDeque<P::Message>,
    subscriptions: P::Subscriptions,
}

#[allow(
    private_bounds,
    reason = "See StreamingClient — bound is sealed on purpose."
)]
impl<P: StreamProtocol + StreamProtocolCodec> StreamingClient<P> {
    /// Connect to `url` and complete the connect/auth handshake using
    /// `api_key`.
    pub(crate) async fn connect(api_key: ApiKey, url: &str) -> Result<Self, Error> {
        let websocket = StreamSocket::<P>::connect(url).await?;
        let mut client = Self {
            websocket,
            messages: VecDeque::new(),
            subscriptions: P::Subscriptions::default(),
        };

        let connection_confirmation = client.next_message_internal().await?;
        if let Some(ControlMessage::Connected) = P::control(&connection_confirmation) {
            info!("Connected to Alpaca Streaming API");
        } else {
            return Err(Error::UnexpectedConnectionMessage(format!(
                "{connection_confirmation:?}",
            )));
        }

        client
            .websocket
            .send(Request::AuthMessage {
                key: api_key.key_id().to_string(),
                secret: api_key.secret_key().to_string(),
            })
            .await?;
        let auth_response = client.next_message_internal().await?;
        if let Some(ControlMessage::Authenticated) = P::control(&auth_response) {
            info!("Authenticated with Alpaca Streaming API");
        } else {
            error!("Alpaca rejected the streaming credentials: {auth_response:?}");
            return Err(Error::StreamingAuth);
        }
        Ok(client)
    }

    /// Receive the next message from the feed, transparently consuming
    /// subscription-confirmation envelopes.
    ///
    /// If the server sends an error envelope, this returns
    /// [`Error::StreamingError`] rather than delivering it as a message —
    /// so a server-side error can't be silently dropped by a caller that
    /// only matches on data variants. The WebSocket is left open.
    pub async fn next_message(&mut self) -> Result<P::Message, Error> {
        loop {
            let incoming = self.next_message_internal().await?;
            if let Some(stream_error) = P::stream_error(&incoming) {
                error!("Alpaca streaming error: {stream_error:?}");
                return Err(Error::StreamingError(stream_error.clone()));
            }
            if let Some(message) = self.handle_subscription_update(incoming) {
                return Ok(message);
            }
        }
    }

    /// Subscribe to additional channels, returning the server-confirmed
    /// subscription list.
    pub async fn add_subscriptions(
        &mut self,
        subscriptions: &P::Subscriptions,
    ) -> Result<P::Subscriptions, Error> {
        self.websocket
            .send(Request::Subscribe(subscriptions.clone()))
            .await?;
        self.await_subscription_update_message().await?;
        Ok(self.subscriptions.clone())
    }

    /// Unsubscribe from channels, returning the server-confirmed
    /// subscription list.
    pub async fn remove_subscriptions(
        &mut self,
        subscriptions: &P::Subscriptions,
    ) -> Result<P::Subscriptions, Error> {
        self.websocket
            .send(Request::Unsubscribe(subscriptions.clone()))
            .await?;
        self.await_subscription_update_message().await?;
        Ok(self.subscriptions.clone())
    }

    /// Close the WebSocket connection and shut down the client.
    pub async fn shut_down(self) -> Result<(), Error> {
        self.websocket.close_connection().await?;
        Ok(())
    }

    async fn await_subscription_update_message(&mut self) -> Result<(), Error> {
        let mut received = false;
        while !received {
            match self.websocket.next_message().await {
                Ok(messages) => {
                    for message in messages {
                        if let Some(stream_error) = P::stream_error(&message) {
                            error!("Alpaca rejected the subscription request: {stream_error:?}");
                            return Err(Error::StreamingSubscribe(stream_error.clone()));
                        }
                        match self.handle_subscription_update(message) {
                            None => {
                                received = true;
                            }
                            Some(message) => {
                                self.messages.push_back(message);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error retrieving next message: {e:?}");
                    return Err(Error::from(e));
                }
            }
        }
        Ok(())
    }

    async fn next_message_internal(&mut self) -> Result<P::Message, Error> {
        while self.messages.is_empty() {
            match self.websocket.next_message().await {
                Ok(messages) => self.messages.extend(messages),
                Err(e) => {
                    error!("Error retrieving next message: {e:?}");
                    return Err(Error::from(e));
                }
            }
        }
        Ok(self
            .messages
            .pop_front()
            .expect("loop above guarantees the queue is non-empty"))
    }

    fn handle_subscription_update(&mut self, message: P::Message) -> Option<P::Message> {
        match P::take_subscription_update(message) {
            Ok(updated) => {
                self.subscriptions = updated;
                None
            }
            Err(other) => Some(other),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::StreamingClient;
    use crate::{Error, env::ApiKey, streaming::StockProtocol};
    use futures::{SinkExt, StreamExt};
    use socketeer::{Message, WebSocketStreamType, get_mock_address, tungstenite};

    // The `success` envelope the real feed sends immediately on connect.
    const CONNECTED: &str = r#"[{"T":"success","msg":"connected"}]"#;

    /// Script the server side of the connect/auth handshake: send the
    /// connection confirmation, wait for the client's auth request, then reply
    /// with `auth_response` (a JSON array of stock stream messages). The socket
    /// is held open until the client hangs up so the final frame flushes.
    async fn scripted_handshake(
        mut ws: WebSocketStreamType,
        auth_response: &'static str,
    ) -> Result<bool, tungstenite::Error> {
        ws.send(Message::text(CONNECTED)).await?;
        let _auth_request = ws.next().await;
        ws.send(Message::text(auth_response)).await?;
        while let Some(Ok(message)) = ws.next().await {
            if message.is_close() {
                break;
            }
        }
        Ok(true)
    }

    /// A rejected key pair must surface as `Error::StreamingAuth`, not a
    /// "successful" connect. Regression test for the silent auth-rejection bug.
    #[tokio::test]
    async fn connect_rejects_failed_auth() {
        let address = get_mock_address(|ws| {
            scripted_handshake(ws, r#"[{"T":"error","code":402,"msg":"auth failed"}]"#)
        })
        .await;
        let url = format!("ws://{address}");

        let result =
            StreamingClient::<StockProtocol>::connect(ApiKey::new("bad", "creds"), &url).await;

        assert!(
            matches!(result, Err(Error::StreamingAuth)),
            "expected Err(StreamingAuth) on rejected auth, got {result:?}"
        );
    }

    /// A successful handshake still returns a connected client.
    #[tokio::test]
    async fn connect_accepts_successful_auth() {
        let address = get_mock_address(|ws| {
            scripted_handshake(ws, r#"[{"T":"success","msg":"authenticated"}]"#)
        })
        .await;
        let url = format!("ws://{address}");

        let result =
            StreamingClient::<StockProtocol>::connect(ApiKey::new("good", "creds"), &url).await;

        assert!(result.is_ok(), "expected Ok(client), got {result:?}");
    }

    /// A subscribe the server rejects (error envelope, socket kept open) must
    /// return `Err(StreamingSubscribe)` rather than blocking forever waiting
    /// for a confirmation that never arrives. Regression test for the silent
    /// subscribe-rejection hang.
    #[tokio::test]
    async fn add_subscriptions_surfaces_rejection() {
        use crate::streaming::{StockSubscriptionList, StreamErrorCode};

        // Complete the auth handshake, then reject the subscribe request with
        // an error envelope while keeping the socket open — mirroring Alpaca's
        // behaviour for e.g. an insufficient-subscription plan.
        async fn reject_subscribe(mut ws: WebSocketStreamType) -> Result<bool, tungstenite::Error> {
            ws.send(Message::text(CONNECTED)).await?;
            let _auth_request = ws.next().await;
            ws.send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#))
                .await?;
            let _subscribe_request = ws.next().await;
            ws.send(Message::text(
                r#"[{"T":"error","code":409,"msg":"insufficient subscription"}]"#,
            ))
            .await?;
            while let Some(Ok(message)) = ws.next().await {
                if message.is_close() {
                    break;
                }
            }
            Ok(true)
        }

        let address = get_mock_address(reject_subscribe).await;
        let url = format!("ws://{address}");
        let mut client = StreamingClient::<StockProtocol>::connect(ApiKey::new("k", "s"), &url)
            .await
            .expect("handshake succeeds");

        let subscriptions = StockSubscriptionList::new().add_trades("AAPL");
        // The timeout guards against a regression re-introducing the hang: on
        // the fixed code the call returns promptly with an error.
        let outcome = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.add_subscriptions(&subscriptions),
        )
        .await
        .expect("add_subscriptions must not hang on a rejected subscribe");

        match outcome {
            Err(Error::StreamingSubscribe(err)) => {
                assert_eq!(err.code, StreamErrorCode::InsufficientSubscription);
            }
            other => panic!("expected Err(StreamingSubscribe), got {other:?}"),
        }
    }

    /// A mid-stream error envelope must surface as `Err(StreamingError)` from
    /// `next_message`, not be handed back as `Ok(Message::Error(..))` where a
    /// caller matching only data variants would silently drop it. Regression
    /// test for the mid-stream-error footgun.
    #[tokio::test]
    async fn next_message_surfaces_stream_error() {
        use crate::streaming::StreamErrorCode;

        // Complete the handshake, then push a mid-stream error envelope and
        // keep the socket open.
        async fn stream_error_after_auth(
            mut ws: WebSocketStreamType,
        ) -> Result<bool, tungstenite::Error> {
            ws.send(Message::text(CONNECTED)).await?;
            let _auth_request = ws.next().await;
            ws.send(Message::text(r#"[{"T":"success","msg":"authenticated"}]"#))
                .await?;
            ws.send(Message::text(
                r#"[{"T":"error","code":407,"msg":"slow client"}]"#,
            ))
            .await?;
            while let Some(Ok(message)) = ws.next().await {
                if message.is_close() {
                    break;
                }
            }
            Ok(true)
        }

        let address = get_mock_address(stream_error_after_auth).await;
        let url = format!("ws://{address}");
        let mut client = StreamingClient::<StockProtocol>::connect(ApiKey::new("k", "s"), &url)
            .await
            .expect("handshake succeeds");

        let outcome =
            tokio::time::timeout(std::time::Duration::from_secs(5), client.next_message())
                .await
                .expect("next_message must not hang on an error envelope");

        match outcome {
            Err(Error::StreamingError(err)) => {
                assert_eq!(err.code, StreamErrorCode::SlowClient);
            }
            other => panic!("expected Err(StreamingError), got {other:?}"),
        }
    }
}
