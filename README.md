# oxidized-alpaca

![Build](https://github.com/VoidstarSolutions/oxidized-alpaca/actions/workflows/ci.yml/badge.svg)
[![codecov](https://codecov.io/gh/VoidstarSolutions/oxidized-alpaca/branch/main/graph/badge.svg?token=FPSZKGUEZ4)](https://codecov.io/gh/VoidstarSolutions/oxidized-alpaca)

Oxidized Alpaca is a pure Rust wrapper for the [Alpaca](https://alpaca.markets)
trading and market data APIs.

## Features

The crate is organized around three opt-in feature flags, all enabled by default:

- `restful` — REST clients for the trading and market data APIs (built on `reqwest`).
- `streaming` — WebSocket streaming clients for real-time market data.
- `tracing` — emit `tracing` spans/events from internal request handling.

Disable defaults and pick what you need, e.g.
`oxidized_alpaca = { version = "*", default-features = false, features = ["restful"] }`.

## Authentication

Credentials are loaded from environment variables based on the `AccountType` you
select when constructing a client:

| Account type | Key ID env var             | Secret key env var             |
| ------------ | -------------------------- | ------------------------------ |
| `Paper`      | `ALPACA_PAPER_API_KEY_ID`  | `ALPACA_PAPER_API_SECRET_KEY`  |
| `Live`       | `ALPACA_LIVE_API_KEY_ID`   | `ALPACA_LIVE_API_SECRET_KEY`   |

Construction returns `Err(Error::MissingEnvironmentVariable)` if the required
variables for the chosen account are not set.

## REST API overview

Two REST clients are exposed at the crate root:

- `TradingClient` — talks to `paper-api.alpaca.markets` or `api.alpaca.markets`
  depending on the `AccountType`. Used for everything that mutates or queries
  account state (orders, positions, watchlists, etc.).
- `MarketDataClient` — always talks to `data.alpaca.markets`. Used for stock,
  crypto, and options market data, news, screeners, logos, and reference data.

Both clients are cheap to clone and safe to share across tasks/threads —
internally they wrap a single `reqwest::Client`. Generally create one instance
per account type and reuse it.

### Request style

Endpoints come in two flavors:

1. **Direct async methods** for simple calls — e.g. `client.get_account().await?`
   or `client.stock_latest_trade("AAPL").await?`.
2. **Builder methods** for endpoints with optional parameters — these return a
   request struct with chainable setters and a terminal `.execute().await?`. For
   example:

   ```rust
   let bars = client
       .stock_bars("AAPL", TimeFrame::OneDay)
       .start(start)
       .end(end)
       .feed(Feed::IEX)
       .limit(1000)
       .execute()
       .await?;
   ```

All endpoints return `crate::Result<T>`, where errors are normalized into the
`Error` enum. Non-2xx HTTP responses surface as `Error::ApiError { status, body }`
with the raw payload preserved for inspection; transport, deserialization, and
URL-build failures are reported separately as `Error::ReqwestSend`,
`Error::ReqwestDeserialize`, and `Error::UrlParse`.

### Strongly-typed parameters

Closed-vocabulary parameters are modeled as enums rather than free-form strings.
Filters and direction hints (`SortDirection`, `OrderStatusFilter`,
`ActivityCategory`, `ContractStatus`, `OptionStyle`, `MoverMarket`,
`CorporateActionType`, `Tape`), durations and resolutions
(`HistoryPeriod`, `HistoryTimeFrame`, `IntradayReporting`, `PnlReset`,
`TimeFrame`), and account-config knobs (`DtbpCheck`, `PdtCheck`,
`TradeConfirmEmail`) all expose typed values that round-trip through serde.

Multi-symbol parameters take `&[&str]` slices (`stock_latest_quotes(&["AAPL",
"MSFT"])`), and monetary or quantity fields on responses come back as
`rust_decimal::Decimal` rather than strings.

### Pagination

Endpoints that paginate auto-fetch the entire result set. Setting `.limit(n)`
on a paginated builder caps the total number of items returned across all
pages — there are no `page_token` or `page_size` knobs on the public API.

## REST API coverage

The REST surface currently covers the following Alpaca endpoints. Method names
listed below are the public entry points on the relevant client.

### Trading API (`TradingClient`)

| Area                    | Methods |
| ----------------------- | ------- |
| Account                 | `get_account` |
| Account configuration   | `get_account_config`, `update_account_config` |
| Account activities      | `list_activities`, `get_activity` |
| Assets                  | `list_assets`, `get_asset` |
| Calendar                | `get_calendar` |
| Clock                   | `get_clock` |
| Options contracts       | `list_option_contracts`, `get_option_contract` |
| Orders                  | `create_order`, `list_orders`, `get_order`, `get_order_by_client_id`, `replace_order`, `cancel_order`, `cancel_all_orders` |
| Portfolio history       | `portfolio_history` |
| Positions               | `list_positions`, `get_position`, `close_position`, `close_all_positions`, `exercise_option`, `do_not_exercise` |
| Watchlists              | `list_watchlists`, `get_watchlist`, `create_watchlist`, `update_watchlist`, `add_to_watchlist`, `remove_from_watchlist`, `delete_watchlist` |

`create_order` and `replace_order` return builders that support market, limit,
stop, stop-limit, trailing-stop, and bracket/OCO/OTO order classes via setters
such as `.qty`, `.notional`, `.time_in_force`, `.limit_price`, `.stop_price`,
`.trail_price`, `.trail_percent`, `.extended_hours`, `.client_order_id`,
`.order_class`, `.take_profit`, and `.stop_loss`.

### Market Data API (`MarketDataClient`)

#### Stocks (`v2/stocks/...`)

| Endpoint | Methods |
| -------- | ------- |
| Historical bars        | `stock_bars` (builder: `.start`, `.end`, `.limit`, `.adjustment`, `.feed`, `.asof`, `.currency`, `.sort`) |
| Trades                 | `stock_trades` (builder: `.start`, `.end`, `.limit`, `.feed`, `.asof`, `.currency`, `.sort`), `stock_latest_trade`, `stock_latest_trades` |
| Quotes                 | `stock_quotes` (builder: `.start`, `.end`, `.limit`, `.feed`, `.asof`, `.currency`, `.sort`), `stock_latest_quote`, `stock_latest_quotes` |
| Auctions               | `stock_auctions` |
| Snapshots              | `stock_snapshot`, `stock_snapshots` |
| Reference / metadata   | `stock_conditions(tick_type, tape)`, `stock_exchanges` |

#### Crypto (`v1beta3/crypto/...`)

| Endpoint           | Methods |
| ------------------ | ------- |
| Historical bars    | `crypto_bars` (builder: `.start`, `.end`, `.limit`) |
| Latest bars        | `crypto_latest_bars` |
| Latest trades      | `crypto_latest_trades` |
| Latest quotes      | `crypto_latest_quotes` |
| Snapshots          | `crypto_snapshots` |
| Latest orderbooks  | `crypto_latest_orderbooks` |

#### Options (`v1beta1/options/...`)

| Endpoint           | Methods |
| ------------------ | ------- |
| Historical bars    | `option_bars` (builder: `.start`, `.end`, `.limit`) |
| Latest trades      | `option_latest_trades` |
| Latest quotes      | `option_latest_quotes` |
| Snapshots          | `option_snapshots` |
| Option chain       | `option_chain` |

#### News, screener, and reference data

| Area                | Methods |
| ------------------- | ------- |
| News                | `news` (builder: `.symbols`, `.start`, `.end`, `.limit`, `.sort`, `.include_content`, `.exclude_contentless`) |
| Screener            | `most_actives`, `market_movers` |
| Logos               | `logo` (returns raw image bytes) |
| Corporate actions   | `corporate_actions` (builder: `.symbols`, `.types`, `.ids`, `.start`, `.end`, `.limit`, `.sort`) |
| Forex               | `forex_latest_rates` |
| Fixed income        | `fixed_income_latest_prices` |

## Quick start

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
oxidized_alpaca = "0.0"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Fetch account details from a paper trading account:

```rust
use oxidized_alpaca::{AccountType, Error, TradingClient};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = TradingClient::new(AccountType::Paper)?;
    let account = client.get_account().await?;
    println!("{account:?}");
    Ok(())
}
```

Pull a month of daily bars for AAPL:

```rust
use chrono::DateTime;
use std::str::FromStr;
use oxidized_alpaca::{
    AccountType, Error, MarketDataClient, restful::market_data::stock::TimeFrame,
};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client = MarketDataClient::new(AccountType::Paper)?;
    let bars = client
        .stock_bars("AAPL", TimeFrame::OneDay)
        .start(DateTime::from_str("2023-01-01T00:00:00Z").unwrap())
        .end(DateTime::from_str("2023-01-31T23:59:59Z").unwrap())
        .execute()
        .await?;
    println!("{bars:?}");
    Ok(())
}
```

Additional runnable examples live in the [`examples/`](examples) directory.

## License

Dual-licensed under either of [Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT)
at your option.
