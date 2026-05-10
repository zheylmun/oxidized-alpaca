use socketeer::JsonCodec;

use crate::{
    AccountType, Error,
    streaming::{
        client::{StreamProtocol, StreamProtocolCodec, StreamingClient, sealed},
        messages::{NewsStreamMessage, NewsSubscriptionList},
        wire::{ControlMessage, Request},
    },
};

const NEWS_LIVE_URL: &str = "wss://stream.data.alpaca.markets/v1beta1/news";
const NEWS_SANDBOX_URL: &str = "wss://stream.data.sandbox.alpaca.markets/v1beta1/news";

/// Marker type wiring [`NewsStreamMessage`] / [`NewsSubscriptionList`] into the
/// shared [`StreamingClient`].
#[derive(Debug)]
pub struct NewsProtocol;

impl sealed::Sealed for NewsProtocol {}

impl StreamProtocol for NewsProtocol {
    type Message = NewsStreamMessage;
    type Subscriptions = NewsSubscriptionList;

    fn control(message: &Self::Message) -> Option<&ControlMessage> {
        message.control()
    }

    fn take_subscription_update(
        message: Self::Message,
    ) -> Result<Self::Subscriptions, Self::Message> {
        match message {
            NewsStreamMessage::Subscription(updated) => Ok(updated),
            other => Err(other),
        }
    }
}

impl StreamProtocolCodec for NewsProtocol {
    type Codec = JsonCodec<Vec<NewsStreamMessage>, Request<NewsSubscriptionList>>;
}

/// Client for streaming real-time news articles over a WebSocket connection.
pub type StreamingNewsClient = StreamingClient<NewsProtocol>;

impl StreamingNewsClient {
    /// Connect to Alpaca's news streaming feed.
    pub async fn new(account_type: AccountType) -> Result<Self, Error> {
        let url = match account_type {
            AccountType::Live => NEWS_LIVE_URL,
            AccountType::Paper => NEWS_SANDBOX_URL,
        };
        Self::connect(account_type, url).await
    }
}
