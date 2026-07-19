use serde::{Deserialize, Serialize};

use crate::AccountType;

const STREAMING_IEX_URL: &str = "wss://stream.data.alpaca.markets/v2/iex";
const STREAMING_IEX_SANDBOX_URL: &str = "wss://stream.data.sandbox.alpaca.markets/v2/iex";
const STREAMING_SIP_URL: &str = "wss://stream.data.alpaca.markets/v2/sip";
const STREAMING_SIP_SANDBOX_URL: &str = "wss://stream.data.sandbox.alpaca.markets/v2/sip";
const STREAMING_DELAYED_SIP_URL: &str = "wss://stream.data.alpaca.markets/v2/delayed_sip";
const STREAMING_DELAYED_SIP_SANDBOX_URL: &str =
    "wss://stream.data.sandbox.alpaca.markets/v2/delayed_sip";
const STREAMING_TEST_URL: &str = "wss://stream.data.alpaca.markets/v2/test";
const STREAMING_BOATS_URL: &str = "wss://stream.data.alpaca.markets/v1beta1/boats";
const STREAMING_BOATS_SANDBOX_URL: &str = "wss://stream.data.sandbox.alpaca.markets/v1beta1/boats";
const STREAMING_OVERNIGHT_URL: &str = "wss://stream.data.alpaca.markets/v1beta1/overnight";
const STREAMING_OVERNIGHT_SANDBOX_URL: &str =
    "wss://stream.data.sandbox.alpaca.markets/v1beta1/overnight";

const CRYPTO_US_URL: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto/us";
const CRYPTO_US_KRAKEN_URL: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto/us-1";
const CRYPTO_EU_KRAKEN_URL: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto/eu-1";
const CRYPTO_US2_URL: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto/us-2";
const CRYPTO_BS1_URL: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto/bs-1";

const OPTION_INDICATIVE_LIVE_URL: &str = "wss://stream.data.alpaca.markets/v1beta1/indicative";
const OPTION_INDICATIVE_SANDBOX_URL: &str =
    "wss://stream.data.sandbox.alpaca.markets/v1beta1/indicative";
const OPTION_OPRA_LIVE_URL: &str = "wss://stream.data.alpaca.markets/v1beta1/opra";
const OPTION_OPRA_SANDBOX_URL: &str = "wss://stream.data.sandbox.alpaca.markets/v1beta1/opra";

/// Stock market data feed selector for REST market-data endpoints.
///
/// Serializes to the lowercase `feed=` query value Alpaca expects (e.g.
/// `feed=delayed_sip`).
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum RestFeed {
    /// Investors Exchange. Available to all accounts.
    IEX,
    /// Securities Information Processor (full real-time consolidated tape).
    /// Requires the unlimited data subscription.
    SIP,
    /// SIP with a 15-minute delay. Available to free-tier accounts.
    #[serde(rename = "delayed_sip")]
    DelayedSip,
    /// Over-the-counter exchanges.
    Otc,
    /// Blue Ocean ATS overnight feed.
    Boats,
}

/// Stock streaming feed selector. Each variant maps to a distinct
/// WebSocket endpoint Alpaca exposes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum StreamingFeed {
    /// Investors Exchange. Available to all accounts.
    IEX,
    /// Securities Information Processor. Requires the unlimited data subscription.
    SIP,
    /// SIP with a 15-minute delay.
    #[serde(rename = "delayed_sip")]
    DelayedSip,
    /// Test feed for simulated stock data — available outside market hours.
    Test,
    /// Blue Ocean ATS overnight feed.
    Boats,
    /// Alpaca-derived overnight consolidated feed.
    Overnight,
}

impl StreamingFeed {
    /// Return the WebSocket URL for this feed under the given account type.
    #[must_use]
    pub fn url(self, account_type: AccountType) -> &'static str {
        match account_type {
            AccountType::Live => self.live_url(),
            AccountType::Paper => self.sandbox_url(),
        }
    }

    const fn live_url(self) -> &'static str {
        match self {
            Self::IEX => STREAMING_IEX_URL,
            Self::SIP => STREAMING_SIP_URL,
            Self::DelayedSip => STREAMING_DELAYED_SIP_URL,
            Self::Test => STREAMING_TEST_URL,
            Self::Boats => STREAMING_BOATS_URL,
            Self::Overnight => STREAMING_OVERNIGHT_URL,
        }
    }

    const fn sandbox_url(self) -> &'static str {
        match self {
            Self::IEX => STREAMING_IEX_SANDBOX_URL,
            Self::SIP => STREAMING_SIP_SANDBOX_URL,
            Self::DelayedSip => STREAMING_DELAYED_SIP_SANDBOX_URL,
            Self::Test => STREAMING_TEST_URL,
            Self::Boats => STREAMING_BOATS_SANDBOX_URL,
            Self::Overnight => STREAMING_OVERNIGHT_SANDBOX_URL,
        }
    }
}

/// Crypto streaming feed selector. Each variant maps to a distinct
/// WebSocket endpoint Alpaca exposes.
///
/// Alpaca does not run a working crypto sandbox — the wss handshake and
/// auth succeed against the sandbox host but every subscribe is rejected.
/// All variants therefore route to the production wss host regardless
/// of [`AccountType`]; the account type still selects which credential
/// pair is used to authenticate.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[non_exhaustive]
pub enum CryptoFeed {
    /// Alpaca-aggregated US crypto feed.
    #[serde(rename = "us")]
    Us,
    /// Kraken-backed US crypto feed.
    #[serde(rename = "us-1")]
    UsKraken,
    /// Kraken-backed EU crypto feed.
    #[serde(rename = "eu-1")]
    EuKraken,
    /// Secondary US crypto feed (`us-2`).
    #[serde(rename = "us-2")]
    Us2,
    /// Bahamas crypto feed (`bs-1`), the reference feed for perpetual
    /// futures pricing.
    #[serde(rename = "bs-1")]
    Bs1,
}

impl CryptoFeed {
    /// Return the WebSocket URL for this feed.
    ///
    /// `account_type` is accepted for interface symmetry with the other
    /// feeds but does not change the URL; see the type-level docs.
    #[must_use]
    pub fn url(self, _account_type: AccountType) -> &'static str {
        match self {
            Self::Us => CRYPTO_US_URL,
            Self::UsKraken => CRYPTO_US_KRAKEN_URL,
            Self::EuKraken => CRYPTO_EU_KRAKEN_URL,
            Self::Us2 => CRYPTO_US2_URL,
            Self::Bs1 => CRYPTO_BS1_URL,
        }
    }
}

/// Options streaming feed selector. Each variant maps to a distinct
/// WebSocket endpoint Alpaca exposes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OptionFeed {
    /// Alpaca-derived NBBO and trade events for accounts without OPRA
    /// access.
    Indicative,
    /// OPRA real-time consolidated tape.
    Opra,
}

impl OptionFeed {
    /// Return the WebSocket URL for this feed under the given account type.
    #[must_use]
    pub fn url(self, account_type: AccountType) -> &'static str {
        match (self, account_type) {
            (Self::Indicative, AccountType::Live) => OPTION_INDICATIVE_LIVE_URL,
            (Self::Indicative, AccountType::Paper) => OPTION_INDICATIVE_SANDBOX_URL,
            (Self::Opra, AccountType::Live) => OPTION_OPRA_LIVE_URL,
            (Self::Opra, AccountType::Paper) => OPTION_OPRA_SANDBOX_URL,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streaming_urls_split_live_and_sandbox() {
        assert_eq!(
            StreamingFeed::IEX.url(AccountType::Live),
            "wss://stream.data.alpaca.markets/v2/iex"
        );
        assert_eq!(
            StreamingFeed::IEX.url(AccountType::Paper),
            "wss://stream.data.sandbox.alpaca.markets/v2/iex"
        );
    }

    #[test]
    fn delayed_sip_routes_to_its_own_endpoint() {
        assert_eq!(
            StreamingFeed::DelayedSip.url(AccountType::Live),
            "wss://stream.data.alpaca.markets/v2/delayed_sip"
        );
    }

    #[test]
    fn boats_and_overnight_have_dedicated_urls() {
        assert_eq!(
            StreamingFeed::Boats.url(AccountType::Live),
            "wss://stream.data.alpaca.markets/v1beta1/boats"
        );
        assert_eq!(
            StreamingFeed::Overnight.url(AccountType::Live),
            "wss://stream.data.alpaca.markets/v1beta1/overnight"
        );
    }

    #[test]
    fn test_feed_uses_a_single_url_for_both_accounts() {
        assert_eq!(
            StreamingFeed::Test.url(AccountType::Live),
            StreamingFeed::Test.url(AccountType::Paper)
        );
    }

    #[test]
    fn rest_feed_serializes_lowercase_with_renames() {
        assert_eq!(serde_json::to_string(&RestFeed::IEX).unwrap(), "\"iex\"");
        assert_eq!(serde_json::to_string(&RestFeed::SIP).unwrap(), "\"sip\"");
        assert_eq!(
            serde_json::to_string(&RestFeed::DelayedSip).unwrap(),
            "\"delayed_sip\""
        );
        assert_eq!(serde_json::to_string(&RestFeed::Otc).unwrap(), "\"otc\"");
        assert_eq!(
            serde_json::to_string(&RestFeed::Boats).unwrap(),
            "\"boats\""
        );
    }

    #[test]
    fn rest_feed_round_trips_through_serde() {
        for variant in [
            RestFeed::IEX,
            RestFeed::SIP,
            RestFeed::DelayedSip,
            RestFeed::Otc,
            RestFeed::Boats,
        ] {
            let encoded = serde_json::to_string(&variant).unwrap();
            let decoded: RestFeed = serde_json::from_str(&encoded).unwrap();
            assert_eq!(decoded, variant, "{variant:?} did not round-trip");
        }
    }

    #[test]
    fn option_url_matrix_pins_all_four_endpoints() {
        assert_eq!(
            OptionFeed::Indicative.url(AccountType::Live),
            "wss://stream.data.alpaca.markets/v1beta1/indicative",
        );
        assert_eq!(
            OptionFeed::Indicative.url(AccountType::Paper),
            "wss://stream.data.sandbox.alpaca.markets/v1beta1/indicative",
        );
        assert_eq!(
            OptionFeed::Opra.url(AccountType::Live),
            "wss://stream.data.alpaca.markets/v1beta1/opra",
        );
        assert_eq!(
            OptionFeed::Opra.url(AccountType::Paper),
            "wss://stream.data.sandbox.alpaca.markets/v1beta1/opra",
        );
    }

    #[test]
    fn option_urls_split_live_and_sandbox_hosts() {
        for feed in [OptionFeed::Indicative, OptionFeed::Opra] {
            let live = feed.url(AccountType::Live);
            let paper = feed.url(AccountType::Paper);
            assert_ne!(
                live, paper,
                "{feed:?} should route paper accounts to the sandbox host"
            );
            assert!(
                live.starts_with("wss://stream.data.alpaca.markets/"),
                "{live} should target the production host",
            );
            assert!(
                paper.starts_with("wss://stream.data.sandbox.alpaca.markets/"),
                "{paper} should target the sandbox host",
            );
        }
    }

    #[test]
    fn option_indicative_and_opra_route_to_distinct_paths() {
        for account in [AccountType::Live, AccountType::Paper] {
            assert_ne!(
                OptionFeed::Indicative.url(account),
                OptionFeed::Opra.url(account),
                "indicative and OPRA must not collide for {account:?}",
            );
        }
    }

    #[test]
    fn crypto_feed_serializes_as_alpaca_path_segments() {
        // Wire format mirrors the path segment Alpaca uses in the WSS URL
        // (`/v1beta3/crypto/{us,us-1,eu-1}`), so the same identifier is
        // valid in both contexts.
        assert_eq!(serde_json::to_string(&CryptoFeed::Us).unwrap(), "\"us\"");
        assert_eq!(
            serde_json::to_string(&CryptoFeed::UsKraken).unwrap(),
            "\"us-1\""
        );
        assert_eq!(
            serde_json::to_string(&CryptoFeed::EuKraken).unwrap(),
            "\"eu-1\""
        );
        assert_eq!(serde_json::to_string(&CryptoFeed::Us2).unwrap(), "\"us-2\"");
        assert_eq!(serde_json::to_string(&CryptoFeed::Bs1).unwrap(), "\"bs-1\"");
        for variant in [
            CryptoFeed::Us,
            CryptoFeed::UsKraken,
            CryptoFeed::EuKraken,
            CryptoFeed::Us2,
            CryptoFeed::Bs1,
        ] {
            let encoded = serde_json::to_string(&variant).unwrap();
            let decoded: CryptoFeed = serde_json::from_str(&encoded).unwrap();
            assert_eq!(decoded, variant, "{variant:?} did not round-trip");
        }
    }

    #[test]
    fn option_feed_serializes_as_alpaca_path_segments() {
        assert_eq!(
            serde_json::to_string(&OptionFeed::Indicative).unwrap(),
            "\"indicative\""
        );
        assert_eq!(
            serde_json::to_string(&OptionFeed::Opra).unwrap(),
            "\"opra\""
        );
        for variant in [OptionFeed::Indicative, OptionFeed::Opra] {
            let encoded = serde_json::to_string(&variant).unwrap();
            let decoded: OptionFeed = serde_json::from_str(&encoded).unwrap();
            assert_eq!(decoded, variant, "{variant:?} did not round-trip");
        }
    }

    #[test]
    fn crypto_url_ignores_account_type() {
        // Documented invariant: every crypto feed routes to the production
        // wss host regardless of `AccountType`. Pinning it here so a
        // future refactor that wires the account type through can't
        // quietly send paper accounts back to the (broken) sandbox host.
        for feed in [
            CryptoFeed::Us,
            CryptoFeed::UsKraken,
            CryptoFeed::EuKraken,
            CryptoFeed::Us2,
            CryptoFeed::Bs1,
        ] {
            assert_eq!(
                feed.url(AccountType::Live),
                feed.url(AccountType::Paper),
                "{feed:?} must yield the same URL for live and paper accounts",
            );
        }
    }
}
