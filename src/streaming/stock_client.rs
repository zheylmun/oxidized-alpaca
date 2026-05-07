use crate::{
    AccountType, Error, Feed,
    env::Env,
    streaming::{
        messages::{StockStreamMessage, StockSubscriptionList},
        wire::{ControlMessage, Request},
    },
};
use socketeer::Socketeer;
use std::collections::VecDeque;

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

type StockSocket = Socketeer<Vec<StockStreamMessage>, Request<StockSubscriptionList>>;

/// Client for streaming real-time stock market data over a WebSocket connection.
#[derive(Debug)]
pub struct StreamingStockClient {
    websocket: StockSocket,
    messages: VecDeque<StockStreamMessage>,
    subscriptions: StockSubscriptionList,
}

impl StreamingStockClient {
    /// Create a new streaming client connected to the test feed.
    pub async fn new_test_client(account_type: AccountType) -> Result<Self, Error> {
        let env = Env::new(&account_type)?;
        let websocket = StockSocket::connect(Feed::Test.streaming_url(account_type)).await?;
        Self::initialize_with_websocket(env, websocket).await
    }

    /// Create a new streaming client connected to the IEX feed.
    pub async fn new_iex_client(account_type: AccountType) -> Result<Self, Error> {
        let env = Env::new(&account_type)?;
        let websocket = StockSocket::connect(Feed::IEX.streaming_url(account_type)).await?;
        Self::initialize_with_websocket(env, websocket).await
    }

    /// Create a new streaming client connected to the SIP feed.
    pub async fn new_sip_client(account_type: AccountType) -> Result<Self, Error> {
        let env = Env::new(&account_type)?;
        let websocket = StockSocket::connect(Feed::SIP.streaming_url(account_type)).await?;
        Self::initialize_with_websocket(env, websocket).await
    }

    async fn initialize_with_websocket(env: Env, websocket: StockSocket) -> Result<Self, Error> {
        let mut client = Self {
            websocket,
            messages: VecDeque::new(),
            subscriptions: StockSubscriptionList::new(),
        };
        let connection_confirmation = client.next_message_internal().await?;
        if let Some(ControlMessage::Connected) = connection_confirmation.control() {
            info!("Connected to Alpaca Streaming API");
        } else {
            return Err(Error::UnexpectedConnectionMessage(format!(
                "{connection_confirmation:?}",
            )));
        }

        client
            .websocket
            .send(Request::AuthMessage {
                key: env.key_id().to_string(),
                secret: env.secret_key().to_string(),
            })
            .await?;
        let auth_response = client.next_message_internal().await?;
        if let Some(ControlMessage::Authenticated) = auth_response.control() {
            info!("Authenticated with Alpaca Streaming API");
        }
        Ok(client)
    }

    /// Receive the next market data message, filtering out control messages.
    pub async fn next_message(&mut self) -> Result<StockStreamMessage, Error> {
        loop {
            let incoming = self.next_message_internal().await?;
            if let Some(message) = self.handle_subscription_update(incoming) {
                return Ok(message);
            }
        }
    }

    /// Subscribe to additional market data streams, returning the updated subscription list.
    pub async fn add_subscriptions(
        &mut self,
        subscriptions: &StockSubscriptionList,
    ) -> Result<StockSubscriptionList, Error> {
        self.websocket
            .send(Request::Subscribe(subscriptions.clone()))
            .await?;
        self.await_subscription_update_message().await?;
        Ok(self.subscriptions.clone())
    }

    /// Unsubscribe from market data streams, returning the updated subscription list.
    pub async fn remove_subscriptions(
        &mut self,
        subscriptions: &StockSubscriptionList,
    ) -> Result<StockSubscriptionList, Error> {
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
                    return Err(Error::WebsocketError(e));
                }
            }
        }
        Ok(())
    }

    async fn next_message_internal(&mut self) -> Result<StockStreamMessage, Error> {
        while self.messages.is_empty() {
            match self.websocket.next_message().await {
                Ok(messages) => self.messages.extend(messages),
                Err(e) => {
                    error!("Error retrieving next message: {e:?}");
                    return Err(Error::WebsocketError(e));
                }
            }
        }
        Ok(self
            .messages
            .pop_front()
            .expect("loop above guarantees the queue is non-empty"))
    }

    fn handle_subscription_update(
        &mut self,
        message: StockStreamMessage,
    ) -> Option<StockStreamMessage> {
        match message {
            StockStreamMessage::Subscription(updated) => {
                self.subscriptions = updated;
                None
            }
            _ => Some(message),
        }
    }
}
