# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.0.9](https://github.com/zheylmun/oxidized-alpaca/compare/v0.0.8...v0.0.9) - 2026-07-06

### Fixed

- *(streaming)* surface mid-stream errors as StreamingError from next_message
- *(streaming)* surface rejected subscribe/unsubscribe as StreamingSubscribe
- *(streaming)* return StreamingAuth on rejected market-data credentials

### Other

- cargo fmt

## [0.0.8](https://github.com/zheylmun/oxidized-alpaca/compare/v0.0.7...v0.0.8) - 2026-07-04

### Added

- add TradingUpdatesClient::new_with_credentials
- add new_with_credentials to streaming market-data clients
- add MarketDataClient::new_with_credentials
- add TradingClient::new_with_credentials

### Fixed

- model trade-activity side separately to handle sell_short

### Other

- clarify MarketDataClient::new_with_credentials account_type is unused
- document explicit ApiKey credential constructors
- promote Env to public ApiKey type with explicit constructor

## [0.0.7](https://github.com/zheylmun/oxidized-alpaca/compare/v0.0.6...v0.0.7) - 2026-06-30

### Added

- *(market-data)* add currency to stock snapshots via builders
- *(market-data)* add feed/currency to latest stock quotes via builders
- *(market-data)* add logo placeholder param via builder
- *(market-data)* add most-actives ranking metric (by)
- *(trading)* add cancel_orders to bulk position close
- *(trading)* allow filtering orders by multiple asset classes
- *(market-data)* add crypto bars sort param
- *(market-data)* add stock auctions asof, currency, sort
- *(trading)* add portfolio-history cashflow_types filter
- *(market-data)* add corporate-actions cusips, region, page_token
- *(trading)* add option-contract show_deliverables and ppind filters
- *(trading)* add calendar date_type filter
- *(trading)* enrich account Activity with trade/non-trade fields
- *(market-data)* capture remaining corporate-action categories
- *(market-data)* capture option snapshot daily/minute/prev bars
- *(trading)* capture option contract multiplier, deliverables, OI date
- *(market-data)* capture stock trade update/correction flag
- *(trading)* capture portfolio-history base_value_asof and cashflow
- *(trading)* capture calendar session_open/session_close
- *(trading)* capture asset cusip, borrow_status, maintenance margin
- *(trading)* capture disable_overnight_trading config flag
- *(trading)* capture account pending_transfer_in/out
- *(market-data)* capture fixed-income ytm and ytw

### Fixed

- *(api-compliance)* address PR review feedback
- *(market-data)* capture bar trade_count (n) and vwap (vw)
- *(trading)* use colon path for get-order-by-client-id
- *(trading)* send spec values for activity category filter
- *(market-data)* correct fixed-income latest-prices wire format

### Other

- *(deps)* upgrade socketeer to 0.5 and refresh lockfile

## [0.0.6](https://github.com/zheylmun/oxidized-alpaca/compare/v0.0.5...v0.0.6) - 2026-05-10

### Other

- Drop streaming::messages glob re-export to match curated CHANGELOG
- Record 0.1.0 surface-cleanup changes in CHANGELOG
- Document AsOf::SkipSymbolMapping serialize-only direction
- Derive Serialize/Deserialize for CorporateActionType and MoverMarket
- Add iter / as_slice / IntoIterator to AdjustmentList
- Type CorporateActions response with CorporateActionPayload newtype
- Rename streaming event payloads with Event suffix
- Replace wildcard re-exports in streaming::messages
- add pre-commit hooks for fmt, clippy, and conventional commits
- Cover the opaque error newtypes with unit tests
- Disambiguate [Error] rustdoc link and log stability changes
- Mark AdjustmentList #[non_exhaustive]
- Mark streaming message enums #[non_exhaustive]
- Mark Error #[non_exhaustive]
- Wrap reqwest::Error and url::ParseError in opaque error types
- Add mleg multi-leg options order entry point
- Add asset_class and order-id cursor filters to list_orders
- Open up TimeFrame to support arbitrary documented multipliers
- Pin streaming feed wire formats and broaden test coverage
- Seal StreamProtocol so socketeer stops leaking
- Consolidate streaming-client constructors behind feed enums
- Type-state CreateOrderRequest to require qty or notional
- Extract infer_order_class helper from execute()
- Fix doctests to match current builder signatures
- cargo fmt
- Replace create_order with per-OrderType entry points
- Type cancel_all_orders / close_all_positions return shapes
- Add send_no_body helper for body-less responses
- Document the numeric-type policy in README and crate docs
- Accept &id by reference in id-newtype Into bounds
- Fix stale replace_order doctest
- cargo fmt
- Newtype Alpaca-issued identifiers
- Type NewsImage.size and Asset.attributes
- Lift AssetClass to a shared crate-root module
- Rename ActivityType variants to PascalCase
- Type PortfolioHistory.timestamp and .timeframe
- Reshape Order.legs and Order.extended_hours
- Tighten standalone field types and naming
- Mark public response types and enums #[non_exhaustive]
- cargo fmt
- Test order builder leg constructors and limit setter
- Rename stock builder request structs for symmetry
- Route order builder through StopLoss and TakeProfit types
- Test the streaming wire codes, error display, and crypto routing
- Drop get_activity: trading API has no get-by-id endpoint
- Restore inner cause in Error Display strings
- Cover documented streaming error codes 404-411
- Route crypto streaming to production for both account types
- Fix doc links
- cargo fmt
- Update README quick start to the new TimeFrame path
- Standardize all builder limit parameters to usize
- Type crypto_bars and option_bars timeframe arguments
- Promote TimeFrame to the shared market_data module
- Drop filled symbols from query so multi-symbol pagination terminates
- Bound live smoke test to a 7-day window across historical endpoints
- Cover empty-symbols short-circuit on multi-symbol builders
- Drop redundant chrono dev-dependency
- Short-circuit multi-symbol builders on empty symbol lists
- cargo fmt
- Narrow live smoke test multi-symbol windows
- Send a server-side per-page limit hint on multi-symbol builders
- Match single-symbol AdjustmentList parity on multi-symbol bars
- Reword multi-quotes/trades limit doc to match bars
- Cover multi-symbol historical builders without live calls
- Extract per-symbol pagination merge into shared helpers
- Truncate per-symbol series during pagination to bound memory
- Short-circuit multi-symbol limit(0) and missing-symbol early exit
- Reword multi-symbol limit doc to acknowledge page exhaustion
- Add multi-symbol stock historical bars/quotes/trades
- Move version prefix from trading client base URL into call sites
- Add get_activity for single-event lookup by ID
- Add ids filter to corporate_actions
- Promote corporate_actions to a builder
- Add OPCA activity type and activity_sub_type field
- Fix broken intra-doc link on AdjustmentList
- Cover new adjustment builders and serialization paths
- Dedupe AdjustmentList and clarify type docs
- Treat empty adjustments iterator as unset
- Add Adjustment::SpinOff and multi-value adjustment support
- Cover symmetric null-opening and multi-day auctions cases
- Add permissive licenses to cargo.toml
- cargo fmt
- Tolerate null opening/closing auction fields
- Add asof/currency/sort to stock historical builders
- Round-trip every documented trade-updates event
- Add us-1, us-2, eu-1, bs-1 crypto market-data locations
- Add TradingUpdatesClient
- Add trade-updates message types
- Add trade-updates fields to Order
- Lift Order to a feature-agnostic module
- Add Kraken-backed US and EU crypto streaming feeds
- Add Boats and Overnight stock streaming feeds
- Split Feed into RestFeed and StreamingFeed
- Add StreamingOptionClient
- Make StreamProtocol pick its wire codec
- Collapse streaming clients onto a shared generic
- Add StreamingNewsClient
- Add StreamingCryptoClient
- Fill in missing stock streaming message types and fields
- Reorganize streaming module under stock-prefixed names
- Update dependencies

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
