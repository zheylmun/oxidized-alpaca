use futures_util::StreamExt;
use oxidized_alpaca::{
    market_data::stock_pricing::streaming::{Feed, StockDataClient},
    AccountType,
};

#[tokio::main]
pub async fn main() {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
    let mut client = StockDataClient::new(&AccountType::Paper, &Feed::SIP).unwrap();
    {
        let mut result = client.connect().await;
        let mut handled = 0;
        while handled < 2 {
            let message = result.next().await;
            println!("Message: {:?}", message);
            handled += 1;
        }
    }
    client.shutdown();
    println!("Done!");
}
