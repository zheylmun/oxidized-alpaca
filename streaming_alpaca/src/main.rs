//! A simple example of hooking up stdin/stdout to a WebSocket stream.
//!
//! This example will connect to a server specified in the argument list and
//! then forward all data read on stdin to the server, printing out all data
//! received on stdout.
//!
//! Note that this is not currently optimized for performance, especially around
//! buffer management. Rather it's intended to show an example of working with a
//! client.
//!
//! You can use this example together with the `server` example.

use std::time::Duration;

use common_alpaca::AccountType;
use streaming_alpaca::{stock_data, StreamingMarketDataClient};
use tracing_subscriber::fmt::Subscriber;

#[tokio::main]
async fn main() {
    let subscriber = Subscriber::builder()
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let mut client = StreamingMarketDataClient::new_test_client(AccountType::Paper)
        .await
        .unwrap();

    let subscriptions = stock_data::SubscriptionList::new()
        .add_minute_bars("FAKEPACA")
        .add_daily_bars("FAKEPACA")
        .add_updated_bars("FAKEPACA")
        .add_quotes("FAKEPACA")
        .add_trades("FAKEPACA");
    client.add_subscriptions(&subscriptions).await.unwrap();

    let mut count = 0u32;
    loop {
        let message = client.next_message().await.unwrap();
        println!("{:?}", message);
        if count % 100 == 0 {
            client.remove_subscriptions(&subscriptions).await.unwrap();
            tokio::time::sleep(Duration::from_secs(30)).await;
            client.add_subscriptions(&subscriptions).await.unwrap();
        }
        count = count.wrapping_add(1);
    }
}
