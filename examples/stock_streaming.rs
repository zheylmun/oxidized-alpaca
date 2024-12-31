use std::time::Duration;

use oxidized_alpaca::{
    streaming::{stock_data, StreamingMarketDataClient},
    AccountType,
};
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
