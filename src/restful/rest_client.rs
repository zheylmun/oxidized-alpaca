use reqwest::{Client, Method, RequestBuilder, Url};

use crate::{env::Env, error::Result, AccountType};

const KEY_ID_HEADER: &str = "APCA-API-KEY-ID";
const SECRET_KEY_HEADER: &str = "APCA-API-SECRET-KEY";
const PAPER_TRADING_URL: &str = "https://paper-api.alpaca.markets/v2/";
const LIVE_TRADING_URL: &str = "https://api.alpaca.markets/v2/";
const STOCK_DATA_URL: &str = "https://api.alpaca.markets/v2/";

pub(crate) enum RequestAPI {
    Trading,
    StockMarketData,
    //CryptoMarketData,
}

#[derive(Clone, Debug)]
pub struct RestClient {
    pub(crate) account_type: AccountType,
    env: Env,
    client: Client,
}

impl RestClient {
    /// Create a new [`RestClient`] instance with the given [`AccountType`]
    /// Only create one instance of this client per account type.
    /// It can be cloned freely and used in multiple threads.
    ///
    /// # Errors
    ///
    /// - This function will return an error if the required environment variables are not set
    pub fn new(account_type: AccountType) -> Result<RestClient> {
        let env = Env::new(&account_type)?;
        Ok(RestClient {
            account_type,
            env,
            client: Client::new(),
        })
    }

    pub(crate) fn request(&self, method: Method, api: RequestAPI, path: &str) -> RequestBuilder {
        let url = Url::parse(self.host(api)).unwrap().join(path).unwrap();
        self.client
            .request(method, url)
            .header(KEY_ID_HEADER, self.env.key_id())
            .header(SECRET_KEY_HEADER, self.env.secret_key())
    }

    const fn host(&self, api: RequestAPI) -> &'static str {
        match api {
            RequestAPI::Trading => match self.account_type {
                AccountType::Paper => PAPER_TRADING_URL,
                AccountType::Live => LIVE_TRADING_URL,
            },
            RequestAPI::StockMarketData => STOCK_DATA_URL,
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = RestClient::new(AccountType::Paper);
        assert!(client.is_ok());
    }
}
