use oxidized_alpaca::{
    AccountType,
    streaming::{StreamingMarketDataClient, stock_data},
};

#[tokio::main]
async fn main() {
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

    let mut count = 0;
    while count < 10 {
        let message = client.next_message().await.unwrap();
        println!("{message:?}");
        count += 1;
    }
}
