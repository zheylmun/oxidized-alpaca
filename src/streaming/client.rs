use serde::{Serialize, de::DeserializeOwned};
use socketeer::Socketeer;
use std::collections::VecDeque;

use crate::{
    AccountType, Error,
    env::ApiKey,
    streaming::wire::{ControlMessage, Request},
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
    /// Connect to `url` and complete the connect/auth handshake using the
    /// credentials for `account`.
    pub(crate) async fn connect(account: AccountType, url: &str) -> Result<Self, Error> {
        let api_key = ApiKey::from_env(&account)?;
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
        }
        Ok(client)
    }

    /// Receive the next message from the feed, transparently consuming
    /// subscription-confirmation envelopes.
    pub async fn next_message(&mut self) -> Result<P::Message, Error> {
        loop {
            let incoming = self.next_message_internal().await?;
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
