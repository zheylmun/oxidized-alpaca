use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// Oxidized Alpaca requires the following environment variables to be set:
    ///
    /// ### Paper Trading:
    /// - `ALPACA_PAPER_API_KEY_ID`
    /// - `ALPACA_PAPER_API_SECRET_KEY`
    ///
    /// ### Live Trading:
    /// - `ALPACA_LIVE_API_KEY_ID`
    /// - `ALPACA_LIVE_API_SECRET_KEY`
    #[error("Required environment variable not set: {}", variable_name)]
    MissingEnvironmentVariable {
        variable_name: String,
        #[source]
        source: std::env::VarError,
    },

    /// Url Parse Error
    #[error("Url parse error: {}", 0)]
    UrlParse(#[source] url::ParseError),
}
