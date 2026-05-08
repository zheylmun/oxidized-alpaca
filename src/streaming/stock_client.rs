use socketeer::JsonCodec;

use crate::{
    AccountType, Error, StreamingFeed,
    streaming::{
        client::{StreamProtocol, StreamingClient},
        messages::{StockStreamMessage, StockSubscriptionList},
        wire::{ControlMessage, Request},
    },
};

/// Marker type wiring [`StockStreamMessage`] / [`StockSubscriptionList`] into the
/// shared [`StreamingClient`].
#[derive(Debug)]
pub struct StockProtocol;

impl StreamProtocol for StockProtocol {
    type Message = StockStreamMessage;
    type Subscriptions = StockSubscriptionList;
    type Codec = JsonCodec<Vec<StockStreamMessage>, Request<StockSubscriptionList>>;

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

/// Client for streaming real-time stock market data over a WebSocket connection.
pub type StreamingStockClient = StreamingClient<StockProtocol>;

impl StreamingStockClient {
    /// Create a new streaming client connected to the test feed.
    pub async fn new_test_client(account_type: AccountType) -> Result<Self, Error> {
        Self::connect(account_type, StreamingFeed::Test.url(account_type)).await
    }

    /// Create a new streaming client connected to the IEX feed.
    pub async fn new_iex_client(account_type: AccountType) -> Result<Self, Error> {
        Self::connect(account_type, StreamingFeed::IEX.url(account_type)).await
    }

    /// Create a new streaming client connected to the SIP feed.
    pub async fn new_sip_client(account_type: AccountType) -> Result<Self, Error> {
        Self::connect(account_type, StreamingFeed::SIP.url(account_type)).await
    }

    /// Create a new streaming client connected to the 15-minute delayed SIP feed.
    pub async fn new_delayed_sip_client(account_type: AccountType) -> Result<Self, Error> {
        Self::connect(account_type, StreamingFeed::DelayedSip.url(account_type)).await
    }
}
