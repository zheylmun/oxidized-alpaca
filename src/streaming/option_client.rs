use socketeer::MsgPackCodec;

use crate::{
    AccountType, Error, OptionFeed,
    env::ApiKey,
    streaming::{
        client::{StreamProtocol, StreamProtocolCodec, StreamingClient, sealed},
        messages::{OptionStreamMessage, OptionSubscriptionList},
        wire::{ControlMessage, Request},
    },
};

/// Marker type wiring [`OptionStreamMessage`] / [`OptionSubscriptionList`] into
/// the shared [`StreamingClient`].
///
/// Alpaca's options stream is MessagePack-only; JSON is rejected with
/// HTTP 412.
#[derive(Debug)]
pub struct OptionProtocol;

impl sealed::Sealed for OptionProtocol {}

impl StreamProtocol for OptionProtocol {
    type Message = OptionStreamMessage;
    type Subscriptions = OptionSubscriptionList;

    fn control(message: &Self::Message) -> Option<&ControlMessage> {
        message.control()
    }

    fn take_subscription_update(
        message: Self::Message,
    ) -> Result<Self::Subscriptions, Self::Message> {
        match message {
            OptionStreamMessage::Subscription(updated) => Ok(updated),
            other => Err(other),
        }
    }
}

impl StreamProtocolCodec for OptionProtocol {
    type Codec = MsgPackCodec<Vec<OptionStreamMessage>, Request<OptionSubscriptionList>>;
}

/// Client for streaming real-time options market data over a WebSocket
/// connection.
pub type StreamingOptionClient = StreamingClient<OptionProtocol>;

impl StreamingOptionClient {
    /// Connect to the chosen [`OptionFeed`] using credentials loaded from the
    /// environment for `account_type`.
    pub async fn new(account_type: AccountType, feed: OptionFeed) -> Result<Self, Error> {
        let api_key = ApiKey::from_env(&account_type)?;
        Self::new_with_credentials(account_type, feed, api_key).await
    }

    /// Connect to the chosen [`OptionFeed`] using explicitly supplied
    /// credentials.
    pub async fn new_with_credentials(
        account_type: AccountType,
        feed: OptionFeed,
        api_key: ApiKey,
    ) -> Result<Self, Error> {
        Self::connect(api_key, feed.url(account_type)).await
    }
}
