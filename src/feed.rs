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
}
