use socketeer::JsonCodec;

use crate::{
    AccountType, Error,
    env::ApiKey,
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
    /// Connect to Alpaca's news streaming feed using credentials loaded from
    /// the environment for `account_type`.
    pub async fn new(account_type: AccountType) -> Result<Self, Error> {
        let api_key = ApiKey::from_env(&account_type)?;
        Self::new_with_credentials(account_type, api_key).await
    }

    /// Connect to Alpaca's news streaming feed using explicitly supplied
    /// credentials. `account_type` still selects the paper/live URL.
    pub async fn new_with_credentials(
        account_type: AccountType,
        api_key: ApiKey,
    ) -> Result<Self, Error> {
        let url = match account_type {
            AccountType::Live => NEWS_LIVE_URL,
            AccountType::Paper => NEWS_SANDBOX_URL,
        };
        Self::connect(api_key, url).await
    }
}
