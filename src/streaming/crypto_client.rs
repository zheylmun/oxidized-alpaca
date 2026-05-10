use socketeer::JsonCodec;

use crate::{
    AccountType, CryptoFeed, Error,
    streaming::{
        client::{StreamProtocol, StreamingClient},
        messages::{CryptoStreamMessage, CryptoSubscriptionList},
        wire::{ControlMessage, Request},
    },
};

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
    /// Connect to the chosen [`CryptoFeed`] using the credentials for
    /// `account_type`.
    ///
    /// Alpaca does not run a working crypto sandbox, so every feed
    /// routes to the production wss host regardless of account type;
    /// the account type still selects which credential pair is used
    /// to authenticate.
    pub async fn new(account_type: AccountType, feed: CryptoFeed) -> Result<Self, Error> {
        Self::connect(account_type, feed.url(account_type)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Pins the regression fix: every crypto feed routes to the production
    /// wss host. The sandbox host accepts auth but rejects every subscribe,
    /// so even Paper accounts must connect to production.
    #[test]
    fn crypto_urls_target_production_host() {
        for feed in [CryptoFeed::Us, CryptoFeed::UsKraken, CryptoFeed::EuKraken] {
            for account in [AccountType::Live, AccountType::Paper] {
                let url = feed.url(account);
                assert!(
                    url.starts_with("wss://stream.data.alpaca.markets/"),
                    "{url} should target the production wss host",
                );
                assert!(
                    !url.contains("sandbox"),
                    "{url} must not point at the (broken) sandbox host",
                );
            }
        }
    }

    #[test]
    fn crypto_urls_use_distinct_v1beta3_paths() {
        assert_eq!(
            CryptoFeed::Us.url(AccountType::Live),
            "wss://stream.data.alpaca.markets/v1beta3/crypto/us",
        );
        assert_eq!(
            CryptoFeed::UsKraken.url(AccountType::Live),
            "wss://stream.data.alpaca.markets/v1beta3/crypto/us-1",
        );
        assert_eq!(
            CryptoFeed::EuKraken.url(AccountType::Live),
            "wss://stream.data.alpaca.markets/v1beta3/crypto/eu-1",
        );
    }
}
