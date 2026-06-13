//! Asset domain types shared between the REST trading / market-data APIs
//! and the streaming trade-updates feed.

use serde::{Deserialize, Serialize};

/// `AssetClass` represents the category to which the asset belongs to.
/// It serves to identify the nature of the financial instrument.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AssetClass {
    /// US equity securities.
    UsEquity,
    /// US options contracts.
    UsOption,
    /// Cryptocurrency.
    Crypto,
    /// Cryptocurrency perpetual futures.
    CryptoPerp,
}

impl AssetClass {
    /// Wire value as sent to / received from the Alpaca API. Matches the
    /// `snake_case` serde serialization and is the single source of truth
    /// for hand-built query strings (e.g. comma-joined filter lists).
    pub(crate) fn as_wire(self) -> &'static str {
        match self {
            AssetClass::UsEquity => "us_equity",
            AssetClass::UsOption => "us_option",
            AssetClass::Crypto => "crypto",
            AssetClass::CryptoPerp => "crypto_perp",
        }
    }
}
