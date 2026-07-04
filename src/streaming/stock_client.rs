use socketeer::JsonCodec;

use crate::{
    AccountType, Error, StreamingFeed,
    env::ApiKey,
    streaming::{
        client::{StreamProtocol, StreamProtocolCodec, StreamingClient, sealed},
        messages::{StockStreamMessage, StockSubscriptionList},
        wire::{ControlMessage, Request},
    },
};

/// Marker type wiring [`StockStreamMessage`] / [`StockSubscriptionList`] into the
/// shared [`StreamingClient`].
#[derive(Debug)]
pub struct StockProtocol;

impl sealed::Sealed for StockProtocol {}

impl StreamProtocol for StockProtocol {
    type Message = StockStreamMessage;
    type Subscriptions = StockSubscriptionList;

    fn control(message: &Self::Message) -> Option<&ControlMessage> {
        message.control()
    }

    fn take_subscription_update(
        message: Self::Message,
    ) -> Result<Self::Subscriptions, Self::Message> {
        match message {
            StockStreamMessage::Subscription(updated) => Ok(updated),
            other => Err(other),
        }
    }
}

impl StreamProtocolCodec for StockProtocol {
    type Codec = JsonCodec<Vec<StockStreamMessage>, Request<StockSubscriptionList>>;
}

/// Client for streaming real-time stock market data over a WebSocket connection.
pub type StreamingStockClient = StreamingClient<StockProtocol>;

impl StreamingStockClient {
    /// Connect to the chosen [`StreamingFeed`] using credentials loaded from
    /// the environment for `account_type`.
    pub async fn new(account_type: AccountType, feed: StreamingFeed) -> Result<Self, Error> {
        let api_key = ApiKey::from_env(&account_type)?;
        Self::new_with_credentials(account_type, feed, api_key).await
    }

    /// Connect to the chosen [`StreamingFeed`] using explicitly supplied
    /// credentials. `account_type` still selects the paper/live URL.
    pub async fn new_with_credentials(
        account_type: AccountType,
        feed: StreamingFeed,
        api_key: ApiKey,
    ) -> Result<Self, Error> {
        Self::connect(api_key, feed.url(account_type)).await
    }
}
