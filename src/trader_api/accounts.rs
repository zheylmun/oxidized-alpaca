use crate::{
    utilities::{string_as_f64, RestClient},
    AccountType, Error, Result,
};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountStatus {
    /// The account is onboarding.
    Onboarding,
    /// The account application submission failed for some reason.
    SubmissionFailed,
    /// The account has been submitted for review.
    Submitted,
    /// The account information is being updated.
    AccountUpdated,
    /// The final account approval is pending.
    ApprovalPending,
    /// The account is active and ready for trading.
    Active,
    /// The account application has been rejected.
    Rejected,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Currency {
    Usd,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AccountDetails {
    pub id: String,
    pub account_number: String,
    pub status: AccountStatus,
    pub currency: Currency,
    #[serde(deserialize_with = "string_as_f64")]
    pub cash: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub non_marginable_buying_power: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub accrued_fees: f64,
    pub pattern_day_trader: bool,
    pub trade_suspended_by_user: bool,
    pub trading_blocked: bool,
    pub transfers_blocked: bool,
    pub account_blocked: bool,
    pub created_at: DateTime<Utc>,
    pub shorting_enabled: bool,
    #[serde(deserialize_with = "string_as_f64")]
    pub long_market_value: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub short_market_value: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub equity: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub multiplier: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub buying_power: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub initial_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub maintenance_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub sma: f64,
    pub daytrade_count: u32,
    #[serde(deserialize_with = "string_as_f64")]
    pub last_maintenance_margin: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub daytrading_buying_power: f64,
    #[serde(deserialize_with = "string_as_f64")]
    pub regt_buying_power: f64,
}

pub struct Accounts {
    client: RestClient,
}

impl Accounts {
    pub(crate) fn new(client: RestClient) -> Self {
        Self { client }
    }

    pub async fn get(&self) -> Result<AccountDetails> {
        let host = match self.client.account_type {
            AccountType::Paper => "https://paper-api.alpaca.markets/v2/",
            AccountType::Live => "https://api.alpaca.markets/v2/",
        };
        let request = self.client.request(Method::GET, host, "account");
        let response = request.send().await.map_err(Error::ReqwestSend)?;
        response.json().await.map_err(Error::ReqwestDeserialize)
    }
}

#[cfg(test)]
mod tests {
    use serial_test::parallel;

    use super::*;

    #[tokio::test]
    #[parallel]
    async fn test_account_status_deserialization() {
        let json = r#"{
            "id": "ccd4e0fc-5416-4b75-bf7d-463c8dcad0fd",
            "admin_configurations": {},
            "user_configurations": null,
            "account_number": "PA3L2HG811OS",
            "status": "ACTIVE",
            "crypto_status": "ACTIVE",
            "currency": "USD",
            "buying_power": "189805.46",
            "regt_buying_power": "189805.46",
            "daytrading_buying_power": "0",
            "effective_buying_power": "189805.46",
            "non_marginable_buying_power": "94902.73",
            "bod_dtbp": "0",
            "cash": "94902.73",
            "accrued_fees": "0",
            "pending_transfer_in": "0",
            "portfolio_value": "94902.73",
            "pattern_day_trader": true,
            "trading_blocked": false,
            "transfers_blocked": false,
            "account_blocked": false,
            "created_at": "2021-12-22T01:09:20.724911Z",
            "trade_suspended_by_user": false,
            "multiplier": "2",
            "shorting_enabled": true,
            "equity": "94902.73",
            "last_equity": "94902.73",
            "long_market_value": "0",
            "short_market_value": "0",
            "position_market_value": "0",
            "initial_margin": "0",
            "maintenance_margin": "0",
            "last_maintenance_margin": "0",
            "sma": "94902.73",
            "daytrade_count": 0,
            "balance_asof": "2024-02-26",
            "crypto_tier": 1
          }"#;
        let account: AccountDetails = serde_json::from_str(json).unwrap();
        assert_eq!(account.status, AccountStatus::Active);
    }

    #[tokio::test]
    #[parallel]
    async fn test_get_account() {
        let client = RestClient::new(AccountType::Paper).unwrap();
        let accounts = Accounts::new(client);
        let account = accounts.get().await.unwrap();
        assert_eq!(account.currency, Currency::Usd);
    }
}
