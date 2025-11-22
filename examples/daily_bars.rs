use chrono::DateTime;
use oxidized_alpaca::{
    AccountType, Error,
    restful::{
        RestClient,
        market_data::stock::{TimeFrame, bars},
    },
};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = RestClient::new(AccountType::Paper)?;
    tracing_subscriber::fmt().init();
    let bars = bars::get(&client, "AAPL", TimeFrame::OneDay)
        .start(DateTime::from_str("2023-01-01T00:00:00Z").unwrap())
        .end(DateTime::from_str("2023-01-31T23:59:59Z").unwrap())
        .execute()
        .await?;
    print!("{:?}", bars);
    Ok(())
}
