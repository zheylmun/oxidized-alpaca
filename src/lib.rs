pub mod env;
pub mod error;
pub mod market_data;

pub mod utilities;

/// The type of Alpaca account
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AccountType {
    /// Paper trading account
    Paper,
    /// Live trading account
    Live,
}
