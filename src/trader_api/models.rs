use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum AccountStatus {
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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum Currency {
    Usd,
}

#[derive(Clone, Debug, Deserialize)]
struct account_details {
    id: String,
    account_number: String,
    status: AccountStatus,
    currency: Currency,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_status() {
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
        let account: account_details = serde_json::from_str(json).unwrap();
        assert_eq!(account.status, AccountStatus::Active);
    }
}
