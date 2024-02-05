use crate::{
    market_data::{Request, SubscriptionList},
    streaming_client::StreamingClient,
    AccountType,
};
use futures::{future, Stream, StreamExt};
use serde::{Deserialize, Serialize};

pub(crate) const MARKET_DATA_STREAM_HOST: &str =
    "wss://stream.data.alpaca.markets/v1beta3/crypto/us";

/// An enumeration of the different supported data feeds for streaming stock data
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Feed {
    /// Use the Investors Exchange (IEX) as the data source.
    ///
    /// This feed is available to all accounts
    IEX,
    /// This feed is only usable with the unlimited data plan
    SIP,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlMessage {
    Connected,
    Authenticated,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename = "lowercase", tag = "code")]
pub enum Error {
    #[serde(rename = "400")]
    InvalidSyntax,
    #[serde(rename = "401")]
    NotAuthenticated,
    #[serde(rename = "402")]
    AuthFailed,
    #[serde(rename = "403")]
    AlreadyAuthorized,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Quote {
    #[serde(rename = "S")]
    pub symbol: String,
    #[serde(rename = "ax")]
    pub ask_exchange: Option<String>,
    #[serde(rename = "ap")]
    pub ask_price: f64,
    #[serde(rename = "as")]
    pub ask_size: f64,
    #[serde(rename = "bx")]
    pub bid_exchange: Option<String>,
    #[serde(rename = "bp")]
    pub bid_price: f64,
    #[serde(rename = "bs")]
    pub bid_size: f64,
    #[serde(rename = "s")]
    pub trade_size: Option<f64>,
    #[serde(rename = "t")]
    pub timestamp: String,
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Trade {
    #[serde(rename = "S")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub trade_id: i64,
    #[serde(rename = "x")]
    pub exchange: Option<String>,
    #[serde(rename = "p")]
    pub price: f64,
    #[serde(rename = "s")]
    pub size: f64,
    #[serde(rename = "t")]
    pub timestamp: String,
    #[serde(rename = "c")]
    pub conditions: Option<Vec<String>>,
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// The following represent messages we can listen for
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "T")]
pub enum StreamMessage {
    /// Internally consumed stream acknowledging successful completion of requests
    #[serde(rename = "success")]
    Control { msg: ControlMessage },

    #[serde(rename = "error")]
    Error(Error),

    #[serde(rename = "subscription")]
    Subscription(SubscriptionList),

    #[serde(rename = "t")]
    Trade(Trade),
    #[serde(rename = "q")]
    Quote(Quote),
}

pub struct StockPricingSubscription {
    streaming_client: StreamingClient,
}

impl StockPricingSubscription {
    pub fn subscribe(&mut self, subscriptions: SubscriptionList) {
        self.streaming_client
            .send(Request::Subscribe(subscriptions));
    }
    pub fn shutdown(&mut self) {
        self.streaming_client.shutdown();
    }
}

pub struct StockDataClient {}

impl StockDataClient {
    pub async fn connect(
        account_type: AccountType,
        feed: Feed,
    ) -> Result<(StockPricingSubscription, impl Stream<Item = StreamMessage>), crate::error::Error>
    {
        let url = match feed {
            Feed::IEX => MARKET_DATA_STREAM_HOST.to_string(), /*+ "/iex"*/
            Feed::SIP => MARKET_DATA_STREAM_HOST.to_string(), /*+ "/sip"*/
        };
        println!("Connecting to {}", url);
        let mut streaming_client = StreamingClient::new(&account_type, &url).unwrap();
        let stream = streaming_client.connect().await?;
        let auth_request = Request::AuthMessage {
            key: streaming_client.env.key_id.clone(),
            secret: streaming_client.env.secret_key.clone(),
        };
        streaming_client.send(auth_request);

        Ok((
            StockPricingSubscription { streaming_client },
            stream.filter_map(|msg| {
                println!("Received message: {:?}", msg);
                let messages: Vec<StreamMessage> = serde_json::from_str(&msg).unwrap();
                future::ready(Some(messages[0].clone()))
            }),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountType;
    use futures::StreamExt;
    use serial_test::parallel;

    /// Check that we can decode a response containing no bars correctly.
    #[tokio::test]
    #[parallel]
    async fn ensure_connection() {
        let (mut subscription, mut stream) =
            StockDataClient::connect(AccountType::Paper, Feed::SIP)
                .await
                .unwrap();
        assert!(stream.next().await.is_some());

        subscription.shutdown();
    }
}
