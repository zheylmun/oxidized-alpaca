//! Oxidized Alpaca - A Rust client library for the Alpaca trading API.
//!
//! # Numeric types
//!
//! Numeric fields on response types mirror Alpaca's wire format rather than
//! being normalized: fields Alpaca encodes as string-quoted decimals (most
//! Trading API monetary and quantity fields) deserialize to
//! [`rust_decimal::Decimal`], and fields Alpaca encodes as bare JSON numbers
//! (market data prices and sizes, streaming market data, portfolio history
//! equity and P/L, screener / forex / fixed-income prices) deserialize to
//! `f64` and have already been rounded by the JSON number representation by
//! the time they leave the API.
//!
//! The crate does not coerce one encoding into the other, so the mapping
//! from Alpaca's docs onto the Rust types is direct and any precision loss
//! is the API's rather than ours. For real calculations — backtesting,
//! P/L attribution, order sizing — most callers will want to remap these
//! into a representation appropriate to their domain (integer minor units,
//! fixed-point, a `Money` newtype, etc.) rather than computing on the wire
//! types directly.
#![warn(missing_docs)]
/// Asset domain types shared between the REST and streaming APIs.
pub mod asset;
pub use asset::AssetClass;
mod env;
/// Error types for the crate.
pub mod error;
#[cfg(feature = "restful")]
pub use error::RestError;
pub use error::UrlError;
#[cfg(feature = "streaming")]
pub use error::WebsocketError;
pub use error::{Error, Result};
/// Data feed types for streaming and market data sources.
mod feed;
pub use feed::{CryptoFeed, OptionFeed, RestFeed, StreamingFeed};
/// Strongly-typed identifier newtypes for Alpaca-issued IDs.
pub mod ids;
pub use ids::{
    AccountId, ActivityId, AssetId, ClientOrderId, ExecutionId, OptionContractId, OrderId,
    WatchlistId,
};
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
