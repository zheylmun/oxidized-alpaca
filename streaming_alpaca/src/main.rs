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

use common_alpaca::AccountType;
use streaming_alpaca::{Feed, StockPricingClient, SubscriptionList};
use tracing_subscriber::fmt::Subscriber;

#[tokio::main]
async fn main() {
    let subscriber = Subscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let (mut sub) = StockPricingClient::connect(AccountType::Paper, Feed::SIP)
        .await
        .unwrap();
    let subscriptions = SubscriptionList::new()
        .add_quotes("BTC/USD")
        .add_trades("BTC/USD");
    //sub.subscribe(subscriptions);
    let mut handled = 0;
    /*while handled < 5 {
        let message = stream.next().await;
        println!("Message: {:?}", message);
        handled += 1;
    }
    sub.shutdown();*/
    println!("Done!");
}
