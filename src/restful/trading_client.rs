use reqwest::{Client, Method, RequestBuilder, Url};
use serde::de::DeserializeOwned;

use crate::{AccountType, env::Env, error::Error, error::Result};

const KEY_ID_HEADER: &str = "APCA-API-KEY-ID";
const SECRET_KEY_HEADER: &str = "APCA-API-SECRET-KEY";
const PAPER_TRADING_URL: &str = "https://paper-api.alpaca.markets/";
const LIVE_TRADING_URL: &str = "https://api.alpaca.markets/";

/// Client for the Alpaca Trading API.
///
/// Handles account management, orders, positions, watchlists, and other
/// trading operations. Uses `paper-api.alpaca.markets` for paper trading
/// and `api.alpaca.markets` for live trading.
///
/// Only create one instance per account type. It can be cloned freely
/// and used across multiple threads.
#[derive(Clone, Debug)]
pub struct TradingClient {
    account_type: AccountType,
    env: Env,
    client: Client,
}

impl TradingClient {
    /// Create a new [`TradingClient`] with the given [`AccountType`].
    ///
    /// # Errors
    ///
    /// Returns an error if the required environment variables are not set.
    pub fn new(account_type: AccountType) -> Result<Self> {
        let env = Env::new(&account_type)?;
        Ok(Self {
            account_type,
            env,
            client: Client::new(),
        })
    }

    /// Build a request for the given path, which should include the
    /// version prefix (e.g. `"v2/orders"` or `"v2/account/activities"`).
    pub(crate) fn request(&self, method: Method, path: &str) -> Result<RequestBuilder> {
        let url = Url::parse(self.base_url())
            .expect("base URL constants are valid")
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

    /// Send a request and discard the body, returning an
    /// [`Error::ApiError`] for non-2xx status codes.
    pub(crate) async fn send_no_body(&self, request: RequestBuilder) -> Result<()> {
        let response = request.send().await.map_err(Error::ReqwestSend)?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(Error::ApiError {
                status: status.as_u16(),
                body,
            });
        }
        Ok(())
    }

    const fn base_url(&self) -> &'static str {
        match self.account_type {
            AccountType::Paper => PAPER_TRADING_URL,
            AccountType::Live => LIVE_TRADING_URL,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[serial_test::parallel]
    async fn test_trading_client_creation() {
        let client = TradingClient::new(AccountType::Paper);
        assert!(client.is_ok());
    }
}
