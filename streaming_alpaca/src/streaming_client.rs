use common_alpaca::{AccountType, Feed};
use serde::{Deserialize, Serialize};
use socketeer::Socketeer;
use std::fmt;

struct StreamingMarketDataClient<RxMessage, TxMessage> {
    websocket: Socketeer<RxMessage, TxMessage>,
}

impl<RxMessage, TxMessage> StreamingMarketDataClient<RxMessage, TxMessage> {
    pub fn new_test_client(account_type: AccountType) -> Self {
        let websocket = Socketeer::connect(Feed::Test.streaming_url(account_type));
        Self { websocket }
    }
}
