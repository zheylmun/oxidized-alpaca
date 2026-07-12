use crate::restful::{
    TradingClient, decimal_as_string, optional_decimal_as_string, string_as_decimal,
};
use reqwest::Method;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// DTBP (Day Trading Buying Power) check setting.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum DtbpCheck {
    /// Check on both entry and exit.
    Both,
    /// Check on entry only.
    Entry,
    /// Check on exit only.
    Exit,
}

/// Pattern day trader check setting.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum PdtCheck {
    /// Check on both entry and exit.
    Both,
    /// Check on entry only.
    Entry,
    /// Check on exit only.
    Exit,
}

/// Trade confirmation email setting.
#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum TradeConfirmEmail {
    /// Send confirmation emails for all trades.
    All,
    /// Do not send confirmation emails.
    None,
}

/// Account configuration settings.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct AccountConfig {
    /// Day trading buying power check setting.
    ///
    /// `None` when Alpaca omits the field. Following changes to pattern day
    /// trading regulation in 2026, this and [`pdt_check`](Self::pdt_check) are
    /// not returned for accounts where they no longer apply.
    #[serde(default)]
    pub dtbp_check: Option<DtbpCheck>,
    /// Trade confirmation email preference.
    pub trade_confirm_email: TradeConfirmEmail,
    /// Whether trading is suspended.
    pub suspend_trade: bool,
    /// Whether shorting is disabled.
    pub no_shorting: bool,
    /// Whether fractional trading is enabled.
    #[serde(default)]
    pub fractional_trading: bool,
    /// Maximum margin multiplier (e.g. 1, 2, 4).
    #[serde(deserialize_with = "string_as_decimal")]
    #[serde(serialize_with = "decimal_as_string")]
    pub max_margin_multiplier: Decimal,
    /// Maximum options trading level.
    #[serde(default)]
    pub max_options_trading_level: Option<u8>,
    /// Pattern day trader check setting.
    ///
    /// `None` when Alpaca omits the field (see
    /// [`dtbp_check`](Self::dtbp_check)).
    #[serde(default)]
    pub pdt_check: Option<PdtCheck>,
    /// Whether PTP no-exception entry is enabled.
    #[serde(default)]
    pub ptp_no_exception_entry: bool,
    /// Whether overnight trading is disabled.
    #[serde(default)]
    pub disable_overnight_trading: bool,
}

/// Builder for updating account configuration.
#[derive(Debug, Serialize)]
#[must_use]
pub struct UpdateAccountConfigRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
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
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        serialize_with = "optional_decimal_as_string"
    )]
    max_margin_multiplier: Option<Decimal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_options_trading_level: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pdt_check: Option<PdtCheck>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ptp_no_exception_entry: Option<bool>,
}

impl UpdateAccountConfigRequest<'_> {
    /// Set the DTBP check mode.
    pub fn dtbp_check(mut self, check: DtbpCheck) -> Self {
        self.dtbp_check = Some(check);
        self
    }
    /// Set the trade confirmation email preference.
    pub fn trade_confirm_email(mut self, email: TradeConfirmEmail) -> Self {
        self.trade_confirm_email = Some(email);
        self
    }
    /// Set whether trading is suspended.
    pub fn suspend_trade(mut self, suspend: bool) -> Self {
        self.suspend_trade = Some(suspend);
        self
    }
    /// Set whether shorting is disabled.
    pub fn no_shorting(mut self, no_shorting: bool) -> Self {
        self.no_shorting = Some(no_shorting);
        self
    }
    /// Set whether fractional trading is enabled.
    pub fn fractional_trading(mut self, fractional: bool) -> Self {
        self.fractional_trading = Some(fractional);
        self
    }
    /// Set the maximum options trading level.
    pub fn max_options_trading_level(mut self, level: u8) -> Self {
        self.max_options_trading_level = Some(level);
        self
    }

    /// Set the maximum margin multiplier (e.g. 1, 2, 4).
    pub fn max_margin_multiplier(mut self, multiplier: Decimal) -> Self {
        self.max_margin_multiplier = Some(multiplier);
        self
    }

    /// Set the pattern day trader check mode.
    pub fn pdt_check(mut self, check: PdtCheck) -> Self {
        self.pdt_check = Some(check);
        self
    }

    /// Set whether PTP no-exception entry is enabled.
    pub fn ptp_no_exception_entry(mut self, enabled: bool) -> Self {
        self.ptp_no_exception_entry = Some(enabled);
        self
    }

    /// Submit the configuration update.
    pub async fn execute(self) -> crate::Result<AccountConfig> {
        let request = self
            .client
            .request(Method::PATCH, "v2/account/configurations")?
            .json(&self);
        self.client.send_and_deserialize(request).await
    }
}

impl TradingClient {
    /// Get current account configuration.
    pub async fn get_account_config(&self) -> crate::Result<AccountConfig> {
        let request = self.request(Method::GET, "v2/account/configurations")?;
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
            client: self,
            dtbp_check: None,
            trade_confirm_email: None,
            suspend_trade: None,
            no_shorting: None,
            fractional_trading: None,
            max_margin_multiplier: None,
            max_options_trading_level: None,
            pdt_check: None,
            ptp_no_exception_entry: None,
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
            "ptp_no_exception_entry": false,
            "disable_overnight_trading": true
        }"#;
        let config: AccountConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.dtbp_check, Some(DtbpCheck::Both));
        assert_eq!(config.pdt_check, Some(PdtCheck::Entry));
        assert!(!config.suspend_trade);
        assert!(config.disable_overnight_trading);
    }

    /// As of mid-2026, following changes to pattern day trading regulation,
    /// Alpaca omits the `dtbp_check` and `pdt_check` fields for accounts where
    /// they no longer apply. `AccountConfig` must still deserialize such
    /// responses.
    #[test]
    fn test_account_config_without_daytrading_fields() {
        let json = r#"{
            "closing_transactions_only": false,
            "disable_overnight_trading": false,
            "fractional_trading": true,
            "max_margin_multiplier": "4",
            "no_shorting": false,
            "ptp_no_exception_entry": false,
            "suspend_trade": false,
            "trade_confirm_email": "all"
        }"#;
        let config: AccountConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.dtbp_check, None);
        assert_eq!(config.pdt_check, None);
        assert_eq!(config.trade_confirm_email, TradeConfirmEmail::All);
    }
}
