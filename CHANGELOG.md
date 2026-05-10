# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Mark `Error` `#[non_exhaustive]` so future error variants can be added without
  a major bump.
- Wrap `reqwest::Error` and `url::ParseError` in opaque `RestError` and
  `UrlError` newtypes (mirroring the existing `WebsocketError` pattern) so the
  underlying transport crates can be upgraded across major versions without a
  breaking release. Use `std::error::Error::source` to inspect the chain.
- Mark the streaming server-tagged enums (`StockStreamMessage`,
  `CryptoStreamMessage`, `NewsStreamMessage`, `OptionStreamMessage`,
  `TradingUpdatesMessage`) `#[non_exhaustive]` so newly documented message types
  from Alpaca can be added without a major bump.
- Mark `AdjustmentList` `#[non_exhaustive]`.
- **Breaking:** the streaming event payloads that share a name with their REST
  counterparts gained an `Event` suffix to disambiguate the two:
  `NewsArticle` → `NewsArticleEvent`, `StockQuote` → `StockQuoteEvent`,
  `StockTrade` → `StockTradeEvent`, `CryptoBar` → `CryptoBarEvent`,
  `CryptoQuote` → `CryptoQuoteEvent`, `CryptoTrade` → `CryptoTradeEvent`,
  `CryptoOrderbook` → `CryptoOrderbookEvent`, `OptionTrade` → `OptionTradeEvent`,
  `OptionQuote` → `OptionQuoteEvent`. The wire format is unchanged.
- **Breaking:** `streaming::messages` is now a curated module — only each
  per-feed `…StreamMessage` enum and `…SubscriptionList` builder are
  re-exported at `streaming::*`. The individual event payload structs are
  reachable at `streaming::messages::{stock,crypto,news,option,trade_update}::…`.
- **Breaking:** `CorporateActions` event lists changed from
  `Vec<serde_json::Value>` to `Vec<CorporateActionPayload>`, a newtype that
  exposes `id()` and `deserialize_into::<T>()` so callers no longer need a
  `serde_json` dependency to inspect payloads.

### Added

- `AdjustmentList` now exposes `iter()`, `as_slice()`, and `IntoIterator`
  impls so callers can read back the values they constructed.
- `CorporateActionType` and `MoverMarket` now derive `Serialize` /
  `Deserialize` (matching their existing wire vocabularies) so callers can
  round-trip them through configuration files or other wire formats.
- `JsonError` (under `crate::error`) wraps `serde_json::Error` for typed
  payload deserializers, so the underlying JSON dependency stays out of the
  public type graph.

### Fixed

- Resolve the ambiguous rustdoc link on the `Result` alias in `crate::error`.

## [0.0.5](https://github.com/VoidstarSolutions/oxidized-alpaca/compare/v0.0.4...v0.0.5) - 2026-05-07

### Fixed

- fix ci config

### Other

- Update version
- Add manual dispatch and debugging
- Push local changes
- Refresh the README after the API refactor
- Surface URL build errors as Error::UrlParse instead of panicking
- Compile cleanly under any feature combination
- Rename stock::bars::Request to StockBarsRequest
- Drop StreamingMarketDataClient generics and hide Request
- Hide Env from the public API
- Drop the unused ClientState public type
- Standardize builder setter signatures
- Align result-cap and timeframe parameter names
- Take multi-symbol parameters as &[&str] across the API
- Build endpoint paths with a single format! style
- Strongly type OptionContract size and open_interest
- Parse MarketDay open and close as NaiveTime
- Strongly type AccountConfig pdt_check and max_margin_multiplier
- Strongly type Activity numeric and side fields
- Strongly type market_movers and corporate_actions filters
- Strongly type the portfolio_history builder
- Reuse ContractStatus and OptionStyle for option-contracts filters
- Replace stringly-typed list filters with enums
- Document URL parse/join invariants with expect messages
- Drop UpdateAccountConfigRequest::default footgun
- Fix panic when streaming websocket yields an empty batch
- Auto-paginate list_activities and tolerate unknown ActivityType codes
- Auto-paginate list_option_contracts
- Auto-paginate news
- Auto-paginate option_bars
- Auto-paginate crypto_bars
- Treat limit as a total cap on stock list endpoints
- Cargo fmt
- Add trading endpoint coverage smoke test
- Add market data endpoint coverage smoke test
- Fix get_order_by_client_id URL path
- Treat empty order_class string as None
- Send required tape parameter for stock_conditions
- Tolerate null attributes array on Asset
- Tolerate null arrays in market data list responses
- Document the market data modules
- document the trading modules
- Document the streaming interfaces
- Add docs to root
- Update README.md
- Add news, screener, logos, corporate actions, forex, fixed income endpoints
- Add options market data endpoints (bars, trades, quotes, snapshots, chain)
- Add crypto market data endpoints (bars, trades, quotes, snapshots, orderbooks)
- Add stock trades, quotes, snapshots, auctions, and metadata endpoints
- Add Options Contracts API with list and get endpoints
- Add account config, activities, portfolio history, watchlists, calendar, clock
- Add Positions API with list, get, close, exercise, and do-not-exercise
- Add complete Orders API with create, list, get, cancel, and replace
- Split RestClient into TradingClient + MarketDataClient, add rust_decimal
- Apply suggestions from code review
- Update all the actions
- Add release-plz configuration for automated release PRs
- Add cargo-semver-checks and cargo-deny to justfile
- Add cargo-deny configuration for license and advisory checking
