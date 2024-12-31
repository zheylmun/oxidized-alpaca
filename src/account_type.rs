use serde::{Deserialize, Serialize};

/// The type of Alpaca account
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum AccountType {
    /// Paper trading account
    Paper,
    /// Live trading account
    Live,
}
