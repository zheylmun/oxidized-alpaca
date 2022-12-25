use reqwest::{Client, Method, RequestBuilder, Url};

use crate::{env::Env, error::Result, AccountType};

#[derive(Debug)]
pub struct RestClient {
    env: Env,
    client: Client,
}

impl RestClient {
    /// Create a new [`RestClient`] instance with the given [`AccountType`]
    ///
    /// # Errors
    ///
    /// - This function will return an error if the required environment variables are not set
    pub fn new(account_type: AccountType) -> Result<RestClient> {
        let env = Env::new(account_type)?;
        Ok(RestClient {
            env,
            client: Client::new(),
        })
    }

    pub(crate) fn request(&self, method: Method, host: &str, path: &str) -> RequestBuilder {
        let url = Url::parse(host).unwrap().join(path).unwrap();
        self.client
            .request(method, url)
            .header("APCA-API-KEY-ID", &self.env.key_id)
            .header("APCA-API-SECRET-KEY", &self.env.secret_key)
    }
}
