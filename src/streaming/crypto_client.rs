use socketeer::JsonCodec;

use crate::{
    AccountType, Error,
    streaming::{
        client::{StreamProtocol, StreamingClient},
        messages::{CryptoStreamMessage, CryptoSubscriptionList},
        wire::{ControlMessage, Request},
    },
};

// Alpaca does not run a working crypto sandbox: the wss handshake and auth
// succeed against `stream.data.sandbox.alpaca.markets`, but every subscribe
// is rejected. Both Paper and Live accounts must use the production hosts;
// per-account credentials are still selected by `Env::new` from
// `AccountType`.
const CRYPTO_US_URL: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto/us";
const CRYPTO_US_KRAKEN_URL: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto/us-1";
const CRYPTO_EU_KRAKEN_URL: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto/eu-1";

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
    ///
    /// Both `AccountType::Paper` and `AccountType::Live` route to the same
    /// production wss host because Alpaca does not run a crypto sandbox;
    /// the account type still selects the credential pair used to authenticate.
    pub async fn new_us(account_type: AccountType) -> Result<Self, Error> {
        Self::connect(account_type, CRYPTO_US_URL).await
    }

    /// Connect to Alpaca's Kraken-backed US crypto streaming feed.
    ///
    /// Both `AccountType::Paper` and `AccountType::Live` route to the same
    /// production wss host; see [`new_us`][Self::new_us].
    pub async fn new_us_kraken(account_type: AccountType) -> Result<Self, Error> {
        Self::connect(account_type, CRYPTO_US_KRAKEN_URL).await
    }

    /// Connect to Alpaca's Kraken-backed EU crypto streaming feed.
    ///
    /// Both `AccountType::Paper` and `AccountType::Live` route to the same
    /// production wss host; see [`new_us`][Self::new_us].
    pub async fn new_eu_kraken(account_type: AccountType) -> Result<Self, Error> {
        Self::connect(account_type, CRYPTO_EU_KRAKEN_URL).await
    }
}
