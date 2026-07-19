use reqwest::{Client, Method, RequestBuilder, Url};
use serde::de::DeserializeOwned;

use crate::{AccountType, env::ApiKey, error::Error, error::Result};

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
    api_key: ApiKey,
    client: Client,
    base_url: Url,
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
        let api_key = ApiKey::from_env(&account_type)?;
        Self::new_with_credentials(account_type, api_key)
    }

    /// Create a new [`MarketDataClient`] with explicitly supplied credentials.
    ///
    /// `account_type` is accepted for symmetry with the trading client and
    /// forward compatibility, but is currently unused: all market-data
    /// requests use the same endpoint regardless of account type.
    pub fn new_with_credentials(_account_type: AccountType, api_key: ApiKey) -> Result<Self> {
        Ok(Self {
            api_key,
            client: Client::new(),
            base_url: Url::parse(MARKET_DATA_URL).expect("MARKET_DATA_URL is a valid base URL"),
        })
    }

    /// Point this client at an arbitrary base URL so tests can drive the
    /// paginating endpoints against a local mock server. Crate-internal and
    /// test-only: the public constructors always target Alpaca.
    #[cfg(test)]
    pub(crate) fn with_base_url(mut self, base_url: &str) -> Self {
        self.base_url = Url::parse(base_url).expect("test base URL is valid");
        self
    }

    /// Build a request for the given path, which should include the version prefix
    /// (e.g., `"v2/stocks/AAPL/bars"` or `"v1beta1/news"`).
    pub(crate) fn request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        let url = self.base_url.join(path)?;
        Ok(self
            .client
            .request(method, url)
            .header(KEY_ID_HEADER, self.api_key.key_id())
            .header(SECRET_KEY_HEADER, self.api_key.secret_key()))
    }

    /// Send a request and deserialize the JSON response, returning an
    /// [`Error::ApiError`] for non-2xx status codes.
    pub(crate) async fn send_and_deserialize<T: DeserializeOwned>(
        &self,
        request: RequestBuilder,
    ) -> Result<T> {
        let response = request
            .send()
            .await
            .map_err(|e| Error::ReqwestSend(e.into()))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::ApiError {
                status: status.as_u16(),
                body,
            });
        }
        response
            .json()
            .await
            .map_err(|e| Error::ReqwestDeserialize(e.into()))
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

    #[tokio::test]
    #[serial_test::parallel]
    async fn new_with_credentials_builds_client() {
        let api_key = ApiKey::new("test_key_id", "test_secret_key");
        let client = MarketDataClient::new_with_credentials(AccountType::Paper, api_key);
        assert!(client.is_ok());
    }
}
