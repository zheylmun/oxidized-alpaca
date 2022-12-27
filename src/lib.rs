pub mod env;
pub mod error;
pub mod market_data;
pub mod rest_client;
pub(crate) mod streaming_client;

mod utils;

/// The type of Alpaca account
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccountType {
    /// Paper trading account
    Paper,
    /// Live trading account
    Live,
}
