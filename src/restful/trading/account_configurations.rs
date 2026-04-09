use crate::restful::TradingClient;
use reqwest::Method;
use serde::{Deserialize, Serialize};

/// DTBP (Day Trading Buying Power) check setting.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum DtbpCheck {
    Both,
    Entry,
    Exit,
}

/// Trade confirmation email setting.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum TradeConfirmEmail {
    All,
    None,
}

/// Account configuration settings.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct AccountConfig {
    pub dtbp_check: DtbpCheck,
    pub trade_confirm_email: TradeConfirmEmail,
    pub suspend_trade: bool,
    pub no_shorting: bool,
    #[serde(default)]
    pub fractional_trading: bool,
    pub max_margin_multiplier: String,
    #[serde(default)]
    pub max_options_trading_level: Option<u8>,
    pub pdt_check: String,
    #[serde(default)]
    pub ptp_no_exception_entry: bool,
}

/// Builder for updating account configuration.
#[derive(Debug, Default, Serialize)]
#[must_use]
pub struct UpdateAccountConfigRequest<'a> {
    #[serde(skip)]
    client: Option<&'a TradingClient>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dtbp_check: Option<DtbpCheck>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trade_confirm_email: Option<TradeConfirmEmail>,
    #[serde(skip_serializing_if = "Option::is_none")]
    suspend_trade: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    no_shorting: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    fractional_trading: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_margin_multiplier: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_options_trading_level: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pdt_check: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ptp_no_exception_entry: Option<bool>,
}

impl UpdateAccountConfigRequest<'_> {
    pub fn dtbp_check(mut self, check: DtbpCheck) -> Self {
        self.dtbp_check = Some(check);
        self
    }
    pub fn trade_confirm_email(mut self, email: TradeConfirmEmail) -> Self {
        self.trade_confirm_email = Some(email);
        self
    }
    pub fn suspend_trade(mut self, suspend: bool) -> Self {
        self.suspend_trade = Some(suspend);
        self
    }
    pub fn no_shorting(mut self, no_shorting: bool) -> Self {
        self.no_shorting = Some(no_shorting);
        self
    }
    pub fn fractional_trading(mut self, fractional: bool) -> Self {
        self.fractional_trading = Some(fractional);
        self
    }
    pub fn max_options_trading_level(mut self, level: u8) -> Self {
        self.max_options_trading_level = Some(level);
        self
    }

    /// Submit the configuration update.
    pub async fn execute(self) -> crate::Result<AccountConfig> {
        let client = self.client.unwrap();
        let request = client
            .request(Method::PATCH, "account/configurations")
            .json(&self);
        client.send_and_deserialize(request).await
    }
}

impl TradingClient {
    /// Get current account configuration.
    pub async fn get_account_config(&self) -> crate::Result<AccountConfig> {
        let request = self.request(Method::GET, "account/configurations");
        self.send_and_deserialize(request).await
    }

    /// Update account configuration.
    ///
    /// ```ignore
    /// let config = client.update_account_config()
    ///     .dtbp_check(DtbpCheck::Entry)
    ///     .execute().await?;
    /// ```
    pub fn update_account_config(&self) -> UpdateAccountConfigRequest<'_> {
        UpdateAccountConfigRequest {
            client: Some(self),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_config_deserialization() {
        let json = r#"{
            "dtbp_check": "both",
            "trade_confirm_email": "all",
            "suspend_trade": false,
            "no_shorting": false,
            "fractional_trading": true,
            "max_margin_multiplier": "4",
            "max_options_trading_level": 2,
            "pdt_check": "entry",
            "ptp_no_exception_entry": false
        }"#;
        let config: AccountConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.dtbp_check, DtbpCheck::Both);
        assert!(!config.suspend_trade);
    }
}
