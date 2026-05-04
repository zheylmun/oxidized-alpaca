use oxidized_alpaca::{AccountType, Error, TradingClient};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = TradingClient::new(AccountType::Paper)?;
    let account_details = client.get_account().await?;
    print!("{:?}", account_details);
    Ok(())
}
