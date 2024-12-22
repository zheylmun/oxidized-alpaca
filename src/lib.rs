mod env;
mod error;
pub mod market_data;
mod trader_api;
pub mod utilities;

pub use {
    error::{Error, Result},
    trader_api::TraderApi,
};

use lazy_init::Lazy;
use utilities::RestClient;



pub struct Alpaca {
    rest_client: RestClient,
    trader_api: Lazy<TraderApi>,
}

impl Alpaca {
    /// Create a new Alpaca instance with the given [`AccountType`]
    ///
    /// # Errors
    ///
    /// - This function will return an error if the required environment variables are not set
    pub fn new(account_type: AccountType) -> Result<Self> {
        Ok(Self {
            rest_client: RestClient::new(account_type)?,
            trader_api: Lazy::new(),
        })
    }

    pub fn trader_api(&self) -> &TraderApi {
        self.trader_api
            .get_or_create(|| TraderApi::new(self.rest_client.clone()))
    }
}
