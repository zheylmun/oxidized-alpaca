use reqwest::{Client, Method, RequestBuilder, Url};
use serde::de::DeserializeOwned;

use crate::{AccountType, env::Env, error::Error, error::Result};

const KEY_ID_HEADER: &str = "APCA-API-KEY-ID";
const SECRET_KEY_HEADER: &str = "APCA-API-SECRET-KEY";
const MARKET_DATA_URL: &str = "https://data.alpaca.markets/";

/// Client for the Alpaca Market Data API.
///
/// Handles stock, crypto, and options market data including bars, trades,
/// quotes, snapshots, news, and screener data. All requests go to
/// `data.alpaca.markets` regardless of account type.
///
/// Only create one instance per account type. It can be cloned freely
/// and used across multiple threads.
#[derive(Clone, Debug)]
pub struct MarketDataClient {
    env: Env,
    client: Client,
}

impl MarketDataClient {
    /// Create a new [`MarketDataClient`] with the given [`AccountType`].
    ///
    /// The account type determines which API credentials are loaded from
    /// environment variables, but all requests use the same market data endpoint.
    ///
    /// # Errors
    ///
    /// Returns an error if the required environment variables are not set.
    pub fn new(account_type: AccountType) -> Result<Self> {
        let env = Env::new(&account_type)?;
        Ok(Self {
            env,
            client: Client::new(),
        })
    }

    /// Build a request for the given path, which should include the version prefix
    /// (e.g., `"v2/stocks/AAPL/bars"` or `"v1beta1/news"`).
    pub(crate) fn request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        let url = Url::parse(MARKET_DATA_URL)
            .expect("MARKET_DATA_URL is a valid base URL")
            .join(path)
            .map_err(Error::UrlParse)?;
        Ok(self
            .client
            .request(method, url)
            .header(KEY_ID_HEADER, self.env.key_id())
            .header(SECRET_KEY_HEADER, self.env.secret_key()))
    }

    /// Send a request and deserialize the JSON response, returning an
    /// [`Error::ApiError`] for non-2xx status codes.
    pub(crate) async fn send_and_deserialize<T: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<T> {
        let response = request.send().await.map_err(Error::ReqwestSend)?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::ApiError {
                status: status.as_u16(),
                body,
            });
        }
        response.json().await.map_err(Error::ReqwestDeserialize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[serial_test::parallel]
    async fn test_market_data_client_creation() {
        let client = MarketDataClient::new(AccountType::Paper);
        assert!(client.is_ok());
    }
}
