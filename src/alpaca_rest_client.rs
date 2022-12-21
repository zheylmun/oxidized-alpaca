use crate::alpaca_env::AlpacaEnv;
use reqwest::Client;

struct AlpacaRestClient {
    env: AlpacaEnv,
    client: Client,
}

impl AlpacaRestClient {
    pub fn new(env: AlpacaEnv) -> AlpacaRestClient {
        AlpacaRestClient {
            env,
            client: Client::new(),
        }
    }
}
