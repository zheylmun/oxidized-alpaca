use reqwest::{Client, Method, RequestBuilder, Url};

use crate::env::Env;

pub struct RestClient {
    env: Env,
    client: Client,
}

impl RestClient {
    #[must_use]
    pub fn new(env: Env) -> RestClient {
        RestClient {
            env,
            client: Client::new(),
        }
    }

    pub(crate) fn request(&self, method: Method, host: &str, path: &str) -> RequestBuilder {
        let url = Url::parse(host).unwrap().join(path).unwrap();
        self.client
            .request(method, url)
            .header("APCA-API-KEY-ID", self.env.key_id.clone())
            .header("APCA-API-SECRET-KEY", self.env.secret_key.clone())
    }
}
