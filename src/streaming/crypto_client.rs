use socketeer::JsonCodec;

use crate::{
    AccountType, Error,
    streaming::{
        client::{StreamProtocol, StreamingClient},
        messages::{CryptoStreamMessage, CryptoSubscriptionList},
        wire::{ControlMessage, Request},
    },
};

const CRYPTO_US_LIVE_URL: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto/us";
const CRYPTO_US_SANDBOX_URL: &str = "wss://stream.data.sandbox.alpaca.markets/v1beta3/crypto/us";
const CRYPTO_US_KRAKEN_LIVE_URL: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto/us-1";
const CRYPTO_US_KRAKEN_SANDBOX_URL: &str =
    "wss://stream.data.sandbox.alpaca.markets/v1beta3/crypto/us-1";
const CRYPTO_EU_KRAKEN_LIVE_URL: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto/eu-1";
const CRYPTO_EU_KRAKEN_SANDBOX_URL: &str =
    "wss://stream.data.sandbox.alpaca.markets/v1beta3/crypto/eu-1";

/// Marker type wiring [`CryptoStreamMessage`] / [`CryptoSubscriptionList`] into
/// the shared [`StreamingClient`].
#[derive(Debug)]
pub struct CryptoProtocol;

impl StreamProtocol for CryptoProtocol {
    type Message = CryptoStreamMessage;
    type Subscriptions = CryptoSubscriptionList;
    type Codec = JsonCodec<Vec<CryptoStreamMessage>, Request<CryptoSubscriptionList>>;

    fn control(message: &Self::Message) -> Option<&ControlMessage> {
        message.control()
    }

    fn take_subscription_update(
        message: Self::Message,
    ) -> Result<Self::Subscriptions, Self::Message> {
        match message {
            CryptoStreamMessage::Subscription(updated) => Ok(updated),
            other => Err(other),
        }
    }
}

/// Client for streaming real-time crypto market data over a WebSocket connection.
pub type StreamingCryptoClient = StreamingClient<CryptoProtocol>;

impl StreamingCryptoClient {
    /// Connect to Alpaca's US crypto streaming feed.
    pub async fn new_us(account_type: AccountType) -> Result<Self, Error> {
        let url = match account_type {
            AccountType::Live => CRYPTO_US_LIVE_URL,
            AccountType::Paper => CRYPTO_US_SANDBOX_URL,
        };
        Self::connect(account_type, url).await
    }

    /// Connect to Alpaca's Kraken-backed US crypto streaming feed.
    pub async fn new_us_kraken(account_type: AccountType) -> Result<Self, Error> {
        let url = match account_type {
            AccountType::Live => CRYPTO_US_KRAKEN_LIVE_URL,
            AccountType::Paper => CRYPTO_US_KRAKEN_SANDBOX_URL,
        };
        Self::connect(account_type, url).await
    }

    /// Connect to Alpaca's Kraken-backed EU crypto streaming feed.
    pub async fn new_eu_kraken(account_type: AccountType) -> Result<Self, Error> {
        let url = match account_type {
            AccountType::Live => CRYPTO_EU_KRAKEN_LIVE_URL,
            AccountType::Paper => CRYPTO_EU_KRAKEN_SANDBOX_URL,
        };
        Self::connect(account_type, url).await
    }
}
