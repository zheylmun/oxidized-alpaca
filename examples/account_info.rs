use oxidized_alpaca::{
    AccountType, Error,
    restful::{RestClient, trading::accounts::AccountDetails},
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = RestClient::new(AccountType::Paper)?;
    let account_details = AccountDetails::get(&client).await?;
    print!("{:?}", account_details);
    Ok(())
}
