use socketeer::JsonCodec;

use crate::{
    AccountType, CryptoFeed, Error,
    env::ApiKey,
    streaming::{
        client::{StreamProtocol, StreamProtocolCodec, StreamingClient, sealed},
        messages::{CryptoStreamMessage, CryptoSubscriptionList},
        wire::{ControlMessage, Request, StreamError},
    },
};

/// Marker type wiring [`CryptoStreamMessage`] / [`CryptoSubscriptionList`] into
/// the shared [`StreamingClient`].
#[derive(Debug)]
pub struct CryptoProtocol;

impl sealed::Sealed for CryptoProtocol {}

impl StreamProtocol for CryptoProtocol {
    type Message = CryptoStreamMessage;
    type Subscriptions = CryptoSubscriptionList;

    fn control(message: &Self::Message) -> Option<&ControlMessage> {
        message.control()
    }

    fn stream_error(message: &Self::Message) -> Option<&StreamError> {
        message.stream_error()
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

impl StreamProtocolCodec for CryptoProtocol {
    type Codec = JsonCodec<Vec<CryptoStreamMessage>, Request<CryptoSubscriptionList>>;
}

/// Client for streaming real-time crypto market data over a WebSocket connection.
pub type StreamingCryptoClient = StreamingClient<CryptoProtocol>;

impl StreamingCryptoClient {
    /// Connect to the chosen [`CryptoFeed`] using credentials loaded from the
    /// environment for `account_type`.
    ///
    /// Alpaca does not run a working crypto sandbox, so every feed
    /// routes to the production wss host regardless of account type;
    /// the account type still selects which credential pair is used
    /// to authenticate.
    pub async fn new(account_type: AccountType, feed: CryptoFeed) -> Result<Self, Error> {
        let api_key = ApiKey::from_env(&account_type)?;
        Self::new_with_credentials(account_type, feed, api_key).await
    }

    /// Connect to the chosen [`CryptoFeed`] using explicitly supplied
    /// credentials.
    pub async fn new_with_credentials(
        account_type: AccountType,
        feed: CryptoFeed,
        api_key: ApiKey,
    ) -> Result<Self, Error> {
        Self::connect(api_key, feed.url(account_type)).await
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
        for feed in [
            CryptoFeed::Us,
            CryptoFeed::UsKraken,
            CryptoFeed::EuKraken,
            CryptoFeed::Us2,
            CryptoFeed::Bs1,
        ] {
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
            CryptoFeed::Us2.url(AccountType::Live),
            "wss://stream.data.alpaca.markets/v1beta3/crypto/us-2",
        );
        assert_eq!(
            CryptoFeed::Bs1.url(AccountType::Live),
            "wss://stream.data.alpaca.markets/v1beta3/crypto/bs-1",
        );
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
