use crate::{error::Result, streaming_client::StreamingClient, AccountType};
use futures::Stream;
use serde::Serialize;

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

pub struct StockDataClient {
    streaming_client: StreamingClient,
}

impl StockDataClient {
    pub fn new(account_type: &AccountType, feed: &Feed) -> Result<StockDataClient> {
        let url = match feed {
            Feed::IEX => MARKET_DATA_STREAM_HOST.to_string() + "/iex",
            Feed::SIP => MARKET_DATA_STREAM_HOST.to_string() + "/sip",
        };
        let streaming_client = StreamingClient::new(account_type, &url)?;
        Ok(StockDataClient { streaming_client })
    }

    pub async fn connect(&mut self) -> impl Stream<Item = String> {
        self.streaming_client.connect().await
    }

    pub fn shutdown(&mut self) {
        self.streaming_client.shutdown();
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
        let mut result = client.connect().await;
        assert!(result.next().await.is_some());
        client.shutdown();
    }
}
