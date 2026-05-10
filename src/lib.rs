//! Oxidized Alpaca - A Rust client library for the Alpaca trading API.
#![warn(missing_docs)]
/// Asset domain types shared between the REST and streaming APIs.
pub mod asset;
pub use asset::AssetClass;
mod env;
/// Error types for the crate.
pub mod error;
pub use error::{Error, Result};
/// Data feed types for streaming and market data sources.
mod feed;
pub use feed::{RestFeed, StreamingFeed};
/// Order domain types shared between the REST trading API and the streaming
/// trade-updates feed.
pub mod orders;
mod serde_helpers;

/// RESTful API client and endpoint types.
#[cfg(feature = "restful")]
pub mod restful;
#[cfg(feature = "restful")]
pub use restful::{MarketDataClient, TradingClient};

use serde::{Deserialize, Serialize};
/// Streaming WebSocket API client.
#[cfg(feature = "streaming")]
pub mod streaming;

/// The type of Alpaca account
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum AccountType {
    /// Paper trading account
    Paper,
    /// Live trading account
    Live,
}
