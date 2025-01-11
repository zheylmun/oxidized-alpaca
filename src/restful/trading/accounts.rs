use crate::{
    restful::{rest_client::RequestAPI, string_as_f64, RestClient},
    Error, Result,
};
use chrono::{DateTime, NaiveDate, Utc};
use reqwest::Method;
use serde::Deserialize;

/// `AccountStatus` represents the current status of an Alpaca account
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

/// `Currency` represents the currency of an Alpaca account
/// Currently, only USD is supported.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub enum Currency {
    USD,
}

/// `AccountDetails` is returned by the Alpaca API when requesting account information
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct AccountDetails {
    /// Alpaca account ID
    pub id: String,
    /// Alpaca account number
    pub account_number: String,
    /// Current status of the account
    pub status: AccountStatus,
    /// Default currency for account
    pub currency: Currency,
    /// Account cash balance
    #[serde(deserialize_with = "string_as_f64")]
    pub cash: f64,
    /// Current available non-margin dollar buying power
    #[serde(deserialize_with = "string_as_f64")]
    pub non_marginable_buying_power: f64,
    /// The fees collected.
    #[serde(deserialize_with = "string_as_f64")]
    pub accrued_fees: f64,
    /// Cash pending transfer into the account
    //[serde(deserialize_with = "string_as_f64")]
    //pub pending_transfer_in: Option<f64>,
    /// Cash pending transfer out of the account
    //#[serde(deserialize_with = "string_as_f64")]
    //pub pending_transfer_out: f64,
    ///Whether or not the account has been flagged as a pattern day trader
    pub pattern_day_trader: bool,
    /// User setting. If true, the account is not allowed to place orders.
    pub trade_suspended_by_user: bool,
    /// If true, the account is not allowed to place orders.
    pub trading_blocked: bool,
    /// If true, the account is not allowed to request money transfers.
    pub transfers_blocked: bool,
    /// If true, account activity by user is prohibited.
    pub account_blocked: bool,
    /// Timestamp this account was created at
    pub created_at: DateTime<Utc>,
    /// Flag to denote whether or not the account is permitted to short
    pub shorting_enabled: bool,
    /// Real-time MtM value of all long positions held in the account
    #[serde(deserialize_with = "string_as_f64")]
    pub long_market_value: f64,
    /// Real-time MtM value of all short positions held in the account
    #[serde(deserialize_with = "string_as_f64")]
    pub short_market_value: f64,
    /// Cash + long_market_value + short_market_value
    #[serde(deserialize_with = "string_as_f64")]
    pub equity: f64,
    /// Equity as of previous trading day at 16:00:00 ET
    #[serde(deserialize_with = "string_as_f64")]
    pub last_equity: f64,
    /// Buying power multiplier that represents account margin classification;
    /// valid values:
    /// - 1 (standard limited margin account with 1x buying power)
    /// - 2 (reg T margin account with 2x intraday and overnight buying power; this is the default for all non-PDT accounts with $2,000 or more equity)
    /// - 4 (PDT account with 4x intraday buying power and 2x reg T overnight buying power)
    #[serde(deserialize_with = "string_as_f64")]
    pub multiplier: f64,
    /// Current available $ buying power:
    /// - If multiplier = 4, account daytrade buying power which is calculated as (last_equity - (last) maintenance_margin) 4
    /// - If multiplier = 2, buying_power = max(equity – initial_margin,0) 2
    /// - If multiplier = 1, buying_power = cash
    #[serde(deserialize_with = "string_as_f64")]
    pub buying_power: f64,
    /// Reg T initial margin requirement (continuously updated value)
    #[serde(deserialize_with = "string_as_f64")]
    pub initial_margin: f64,
    /// Maintenance margin requirement (continuously updated value)
    #[serde(deserialize_with = "string_as_f64")]
    pub maintenance_margin: f64,
    /// Value of special memorandum account (will be used at a later date to provide additional buying_power)
    #[serde(deserialize_with = "string_as_f64")]
    pub sma: f64,
    /// The current number of daytrades that have been made in the last 5 trading days (inclusive of today)
    pub daytrade_count: u32,
    /// The date of the snapshot for last_* fields
    pub balance_asof: NaiveDate,
    /// Account maintenance margin requirement on the previous trading day
    #[serde(deserialize_with = "string_as_f64")]
    pub last_maintenance_margin: f64,
    /// Account buying power for day trades (continuously updated value)
    #[serde(deserialize_with = "string_as_f64")]
    pub daytrading_buying_power: f64,
    ///Account buying power under Regulation T (account excess equity - equity minus margin value - times account margin multiplier)
    #[serde(deserialize_with = "string_as_f64")]
    pub regt_buying_power: f64,
    /// Account buying power for options trading
    #[serde(deserialize_with = "string_as_f64", default)]
    pub options_bying_power: f64,
    ///The options trading level that was approved for this account.
    /// - 0=disabled
    /// - 1=Covered Call/Cash-Secured Put
    /// - 2=Long Call/Put
    pub options_approved_level: u8,
    /// The effective options trading level of the account.
    /// This is the minimum between account options_approved_level and account configurations max_options_trading_level:
    /// - 0=disabled
    /// - 1=Covered Call/Cash-Secured Put
    /// - 2=Long Call/Put.
    pub options_trading_level: u8,
    /// The intraday adjustment by non_trade_activities such as fund deposit/withdraw.
    #[serde(deserialize_with = "string_as_f64")]
    pub intraday_adjustments: f64,
    /// Pending regulatory fees for the account.
    #[serde(deserialize_with = "string_as_f64")]
    pub pending_reg_taf_fees: f64,
}

/// Get the account information associated with the Alpaca API key
pub async fn get(client: &RestClient) -> Result<AccountDetails> {
    let request = client.request(Method::GET, RequestAPI::Trading, "account");
    let response = request.send().await.map_err(Error::ReqwestSend)?;
    response.json().await.map_err(Error::ReqwestDeserialize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_account_status_deserialization() {
        let json = r#"{
          "id": "ccd4e0fc-5416-4b75-bf7d-463c8dcad0fd",
          "admin_configurations": {},
          "user_configurations": null,
          "account_number": "PA3L2HG811OS",
          "status": "ACTIVE",
          "crypto_status": "ACTIVE",
          "options_approved_level": 2,
          "options_trading_level": 2,
          "currency": "USD",
          "buying_power": "189805.46",
          "regt_buying_power": "189805.46",
          "daytrading_buying_power": "0",
          "effective_buying_power": "189805.46",
          "non_marginable_buying_power": "94902.73",
          "options_buying_power": "94902.73",
          "bod_dtbp": "0",
          "cash": "94902.73",
          "accrued_fees": "0",
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
          "balance_asof": "2024-12-30",
          "crypto_tier": 1,
          "intraday_adjustments": "0",
          "pending_reg_taf_fees": "0"
        }"#;
        let account: AccountDetails = serde_json::from_str(json).unwrap();
        assert_eq!(account.status, AccountStatus::Active);
    }
}
