use serde::{Deserialize, Serialize};

use crate::AccountType;

const STREAMING_IEX_URL: &str = "wss://stream.data.alpaca.markets/v2/iex";
const STREAMING_IEX_SANDBOX_URL: &str = "wss://stream.data.sandbox.alpaca.markets/v2/iex";
const STREAMING_SIP_URL: &str = "wss://stream.data.alpaca.markets/v2/sip";
const STREAMING_SIP_SANDBOX_URL: &str = "wss://stream.data.sandbox.alpaca.markets/v2/sip";
const STREAMING_TEST_URL: &str = "wss://stream.data.alpaca.markets/v2/test";
const STREAMING_CRYPTO_URL: &str = "wss://stream.data.alpaca.markets/v1beta3/crypto";
const STREAMING_CRYPTO_SANDBOX_URL: &str =
    "wss://stream.data.sandbox.alpaca.markets/v1beta3/crypto";
const STREAMING_NEWS_URL: &str = "wss://stream.data.alpaca.markets/v1beta1/news";
const STREAMING_NEWS_SANDBOX_URL: &str = "wss://stream.data.alpaca.markets/v1beta1/news";

/// Supported data feeds
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Feed {
    /// Investors Exchange (IEX) stock data source.
    ///
    /// This feed is available to all accounts
    IEX,
    /// Securities Information Processor (SIP) stock price data source.
    ///
    /// This feed is only usable with the unlimited data plan
    SIP,
    /// Test data feed for simulated stock data.
    ///
    /// This feed is available outside of market hours and is useful for testing
    Test,
    /// Crypto data feed for cryptocurrency data.
    ///
    /// This feed is available to all accounts, and is not subject to market hours
    Crypto,
    /// News data feed for news data.
    News,
}

impl Feed {
    #[must_use]
    pub fn streaming_url(&self, account_type: AccountType) -> &str {
        match account_type {
            AccountType::Paper => self.streaming_url_paper(),
            AccountType::Live => self.streaming_url_live(),
        }
    }

    #[must_use]
    fn streaming_url_paper(&self) -> &str {
        match self {
            Feed::IEX => STREAMING_IEX_SANDBOX_URL,
            Feed::SIP => STREAMING_SIP_SANDBOX_URL,
            Feed::Test => STREAMING_TEST_URL,
            Feed::Crypto => STREAMING_CRYPTO_SANDBOX_URL,
            Feed::News => STREAMING_NEWS_SANDBOX_URL,
        }
    }

    #[must_use]
    fn streaming_url_live(&self) -> &str {
        match self {
            Feed::IEX => STREAMING_IEX_URL,
            Feed::SIP => STREAMING_SIP_URL,
            Feed::Test => STREAMING_TEST_URL,
            Feed::Crypto => STREAMING_CRYPTO_URL,
            Feed::News => STREAMING_NEWS_URL,
        }
    }
}
