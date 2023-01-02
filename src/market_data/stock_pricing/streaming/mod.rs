use crate::{
    error::Result,
    market_data::{Request, SubscriptionList},
    streaming_client::StreamingClient,
    AccountType,
};
use futures::{future, Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;

pub(crate) const MARKET_DATA_STREAM_HOST: &str = "wss://stream.data.alpaca.markets/v2";

/// An enumeration of the different supported data feeds for streaming stock data
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Feed {
    /// Use the Investors Exchange (IEX) as the data source.
    ///
    /// This feed is available to all accounts
    IEX,
    /// This feed is only usable with the unlimited data plan
    SIP,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "lowercase")]

pub enum ControlMessage {
    Connected,
    Authenticated,
}

#[derive(Debug, Deserialize)]
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

/// The following represent messages we can listen for
#[derive(Debug, Deserialize)]
#[serde(tag = "T")]
pub enum StreamMessage {
    /// Internally consumed stream acknowledging successful completion of requests
    #[serde(rename = "success")]
    Control {},

    #[serde(rename = "error")]
    Error(Error),

    #[serde(rename = "subscription")]
    Subscription { subscriptions: SubscriptionList },
}

pub struct StockDataClient {
    streaming_client: StreamingClient,
    active_subscriptions: SubscriptionList,
}

impl StockDataClient {
    pub fn new(account_type: &AccountType, feed: &Feed) -> Result<StockDataClient> {
        let url = match feed {
            Feed::IEX => MARKET_DATA_STREAM_HOST.to_string() + "/iex",
            Feed::SIP => MARKET_DATA_STREAM_HOST.to_string() + "/sip",
        };
        let streaming_client = StreamingClient::new(account_type, &url)?;
        Ok(StockDataClient {
            streaming_client,
            active_subscriptions: SubscriptionList::new(),
        })
    }

    pub async fn connect(&mut self) -> impl Stream<Item = StreamMessage> + '_ {
        let stream = self.streaming_client.connect().await;
        let auth_request = Request::AuthMessage {
            key: self.streaming_client.env.key_id.clone(),
            secret: self.streaming_client.env.secret_key.clone(),
        };
        let auth_message = Message::Text(serde_json::to_string(&auth_request).unwrap());
        self.streaming_client.send(auth_message);
        stream.filter_map(|msg| {
            println!("Message: {:?}", &msg);
            match serde_json::from_str(&msg).unwrap() {
                StreamMessage::Control {} => future::ready(None),
                StreamMessage::Error(error) => future::ready(Some(StreamMessage::Error(error))),
                StreamMessage::Subscription { subscriptions } => {
                    self.active_subscriptions = subscriptions;
                    future::ready(None)
                }
            }
        })
    }

    pub fn shutdown(&mut self) {
        self.streaming_client.shutdown();
    }

    pub fn subscribe_to_bars(&mut self, symbol: &str) {
        let sub_request = SubscriptionList::new().add_bars(symbol);
        let message = Message::Text(serde_json::to_string(&sub_request).unwrap());
        self.streaming_client.send(message);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AccountType;
    use futures::StreamExt;

    /// Check that we can decode a response containing no bars correctly.
    #[tokio::test]
    async fn ensure_connection() {
        let mut client = StockDataClient::new(&AccountType::Paper, &Feed::IEX).unwrap();
        {
            let mut result = client.connect().await;
            assert!(result.next().await.is_some());
        }
        client.shutdown();
    }
}
