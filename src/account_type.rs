use serde::{Deserialize, Serialize};

/// The type of Alpaca account
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub enum AccountType {
    /// Paper trading account
    Paper,
    /// Live trading account
    Live,
}

const PAPER_URL: &str = "https://paper-api.alpaca.markets/v2/";
const LIVE_URL: &str = "https://api.alpaca.markets/v2/";

impl AccountType {
    /// Get the Alpaca endpoint for the given account type
    pub fn endpoint(&self) -> &'static str {
        match self {
            AccountType::Paper => PAPER_URL,
            AccountType::Live => LIVE_URL,
        }
    }
}
