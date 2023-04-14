use futures_util::StreamExt;
use oxidized_alpaca::{
    market_data::{
        stock_pricing::streaming::{Feed, StockDataClient},
        SubscriptionList,
    },
    AccountType,
};

#[tokio::main]
pub async fn main() {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let (mut sub, mut stream) = StockDataClient::connect(AccountType::Paper, Feed::SIP).await;
    let subscriptions = SubscriptionList::new()
        .add_quotes("BTC/USD")
        .add_trades("BTC/USD");
    sub.subscribe(subscriptions);
    let mut handled = 0;
    while handled < 5 {
        let message = stream.next().await;
        println!("Message: {:?}", message);
        handled += 1;
    }
    sub.shutdown();
    println!("Done!");
}
