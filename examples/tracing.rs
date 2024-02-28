use std::str::FromStr;

use chrono::DateTime;
use oxidized_alpaca::{
    market_data::stock_pricing::historical::{bars::Request, TimeFrame},
    utilities::RestClient,
    AccountType,
};

#[tokio::main]
pub async fn main() {
    let subscriber = tracing_subscriber::fmt::Subscriber::builder()
        .with_max_level(tracing::Level::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber).unwrap();
    let client = RestClient::new(&AccountType::Paper).unwrap();
    let start = DateTime::from_str("2021-12-05T00:00:00Z").unwrap();
    let end = DateTime::from_str("2022-12-24T00:00:00Z").unwrap();
    let request = Request::new(client, "LAZR", TimeFrame::OneDay)
        .start(start)
        .end(end)
        .limit(30);

    let res = request.execute().await;

    assert!(res.is_ok());
    let res = res.unwrap();
    assert_eq!(res.len(), 266);
}
