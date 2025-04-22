mod env;
pub use env::Env;
pub mod error;
pub use error::{Error, Result};
mod feed;
pub use feed::Feed;

#[cfg(feature = "restful")]
pub mod restful;

use serde::{Deserialize, Serialize};
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
