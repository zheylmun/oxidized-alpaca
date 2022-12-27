use serde::Serialize;
pub mod stock_pricing;

/// An enumeration of the different supported data feeds.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Feed {
    /// Use the Investors Exchange (IEX) as the data source.
    ///
    /// This feed is available to all accounts
    IEX,
    /// This feed is only usable with the unlimited data plan
    SIP,
}
