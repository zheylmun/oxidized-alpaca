mod accounts;

use accounts::Accounts;
use lazy_init::Lazy;

use crate::utilities::RestClient;

pub struct TraderApi {
    client: RestClient,
    accounts: Lazy<Accounts>,
}

impl TraderApi {
    pub fn new(client: RestClient) -> Self {
        Self {
            client,
            accounts: Lazy::new(),
        }
    }

    pub fn accounts(&self) -> &Accounts {
        self.accounts
            .get_or_create(|| Accounts::new(self.client.clone()))
    }
}
