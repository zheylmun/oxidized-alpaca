use chrono::DateTime;
use oxidized_alpaca::{
    AccountType, Error, MarketDataClient, restful::market_data::stock::TimeFrame,
};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = MarketDataClient::new(AccountType::Paper)?;
    tracing_subscriber::fmt().init();
    let bars = client
        .stock_bars("AAPL", TimeFrame::OneDay)
        .start(DateTime::from_str("2023-01-01T00:00:00Z").unwrap())
        .end(DateTime::from_str("2023-01-31T23:59:59Z").unwrap())
        .execute()
        .await?;
    print!("{:?}", bars);
    Ok(())
}
