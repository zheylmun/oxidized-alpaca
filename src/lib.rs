pub mod env;
pub mod market_data;
pub mod rest_client;
mod utils;

/// The type of Alpaca account
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AccountType {
    /// Paper trading account
    Paper,
    /// Live trading account
    Live,
}
