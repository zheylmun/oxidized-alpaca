use socketeer::MsgPackCodec;

use crate::{
    AccountType, Error,
    streaming::{
        client::{StreamProtocol, StreamingClient},
        messages::{OptionStreamMessage, OptionSubscriptionList},
        wire::{ControlMessage, Request},
    },
};

const OPTION_INDICATIVE_LIVE_URL: &str = "wss://stream.data.alpaca.markets/v1beta1/indicative";
const OPTION_INDICATIVE_SANDBOX_URL: &str =
    "wss://stream.data.sandbox.alpaca.markets/v1beta1/indicative";
const OPTION_OPRA_LIVE_URL: &str = "wss://stream.data.alpaca.markets/v1beta1/opra";
const OPTION_OPRA_SANDBOX_URL: &str = "wss://stream.data.sandbox.alpaca.markets/v1beta1/opra";

/// Marker type wiring [`OptionStreamMessage`] / [`OptionSubscriptionList`] into
/// the shared [`StreamingClient`].
///
/// Alpaca's options stream is MessagePack-only; JSON is rejected with
/// HTTP 412.
#[derive(Debug)]
pub struct OptionProtocol;

impl StreamProtocol for OptionProtocol {
    type Message = OptionStreamMessage;
    type Subscriptions = OptionSubscriptionList;
    type Codec = MsgPackCodec<Vec<OptionStreamMessage>, Request<OptionSubscriptionList>>;

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

/// Client for streaming real-time options market data over a WebSocket
/// connection.
pub type StreamingOptionClient = StreamingClient<OptionProtocol>;

impl StreamingOptionClient {
    /// Connect to Alpaca's indicative options feed.
    ///
    /// The indicative feed delivers Alpaca-derived NBBO and trade events
    /// for accounts that don't subscribe to the OPRA feed.
    pub async fn new_indicative(account_type: AccountType) -> Result<Self, Error> {
        let url = match account_type {
            AccountType::Live => OPTION_INDICATIVE_LIVE_URL,
            AccountType::Paper => OPTION_INDICATIVE_SANDBOX_URL,
        };
        Self::connect(account_type, url).await
    }

    /// Connect to Alpaca's OPRA options feed (real-time consolidated tape).
    pub async fn new_opra(account_type: AccountType) -> Result<Self, Error> {
        let url = match account_type {
            AccountType::Live => OPTION_OPRA_LIVE_URL,
            AccountType::Paper => OPTION_OPRA_SANDBOX_URL,
        };
        Self::connect(account_type, url).await
    }
}
