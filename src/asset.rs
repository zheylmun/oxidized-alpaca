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
