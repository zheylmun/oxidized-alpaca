use snafu::Snafu;

#[derive(Debug, Eq, PartialEq, Snafu)]
#[snafu(visibility(pub(crate)))]
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
    #[snafu(display("Required environment variable not set: {}", "variable_name"))]
    MissingEnvironmentVariable {
        variable_name: String,
        source: std::env::VarError,
    },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
