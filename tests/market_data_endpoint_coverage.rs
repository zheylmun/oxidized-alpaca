use chrono::{Duration, Utc};
use oxidized_alpaca::{
    AccountType, Error, MarketDataClient,
    restful::market_data::{
        TimeFrame,
        corporate_actions::CorporateActionType,
        crypto::CryptoLocation,
        screener::MoverMarket,
        stock::meta::{Tape, TickType},
    },
};

fn expect_ok_or_status<T>(
    result: Result<T, Error>,
    allowed_statuses: &[u16],
    context: &str,
) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(Error::ApiError { status, .. }) if allowed_statuses.contains(&status) => None,
        Err(err) => panic!("{context} failed unexpectedly: {err:?}"),
    }
}

#[tokio::test]
async fn market_data_endpoints_live_smoke() {
    let client = MarketDataClient::new(AccountType::Paper).unwrap();

    // `.limit(1)` caps items returned but does not bound how far back the
    // server scans — without an explicit window the historical endpoints
    // can wade through years of data per call. Constrain every paginated
    // historical request to a narrow window so the smoke test stays fast
    // on CI. A 7-day window handles weekends/holidays while keeping the
    // payload tiny.
    let window_end = Utc::now();
    let window_start = window_end - Duration::days(7);

    // Stocks
    let bars = client
        .stock_bars("AAPL", TimeFrame::ONE_DAY)
        .start(window_start)
        .end(window_end)
        .limit(1)
        .execute()
        .await
        .unwrap();
    let _ = bars;

    let trades = client
        .stock_trades("AAPL")
        .start(window_start)
        .end(window_end)
        .limit(1)
        .execute()
        .await
        .unwrap();
    let _ = trades;

    client.stock_latest_trade("AAPL").await.unwrap();

    let latest_trades = client.stock_latest_trades(&["AAPL", "MSFT"]).await.unwrap();
    let _ = latest_trades;

    let multi_trades = client
        .stock_trades_multi(&["AAPL", "MSFT"])
        .start(window_start)
        .end(window_end)
        .limit(1)
        .execute()
        .await
        .unwrap();
    let _ = multi_trades;

    let multi_bars = client
        .stock_bars_multi(&["AAPL", "MSFT"], TimeFrame::ONE_DAY)
        .start(window_start)
        .end(window_end)
        .limit(1)
        .execute()
        .await
        .unwrap();
    let _ = multi_bars;

    let quotes = client
        .stock_quotes("AAPL")
        .start(window_start)
        .end(window_end)
        .limit(1)
        .execute()
        .await
        .unwrap();
    let _ = quotes;

    client.stock_latest_quote("AAPL").await.unwrap();

    let latest_quotes = client.stock_latest_quotes(&["AAPL", "MSFT"]).await.unwrap();
    let _ = latest_quotes;

    let multi_quotes = client
        .stock_quotes_multi(&["AAPL", "MSFT"])
        .start(window_start)
        .end(window_end)
        .limit(1)
        .execute()
        .await
        .unwrap();
    let _ = multi_quotes;

    let _ = expect_ok_or_status(
        client
            .stock_auctions("AAPL")
            .start(window_start)
            .end(window_end)
            .limit(1)
            .execute()
            .await,
        &[403],
        "stock_auctions",
    );

    client.stock_snapshot("AAPL", None).await.unwrap();

    let snapshots = client
        .stock_snapshots(&["AAPL", "MSFT"], None)
        .await
        .unwrap();
    let _ = snapshots;

    let trade_conditions = client
        .stock_conditions(TickType::Trade, Tape::A)
        .await
        .unwrap();
    let _ = trade_conditions;

    let quote_conditions = client
        .stock_conditions(TickType::Quote, Tape::C)
        .await
        .unwrap();
    let _ = quote_conditions;

    let exchanges = client.stock_exchanges().await.unwrap();
    let _ = exchanges;

    // Crypto
    let crypto_bars = client
        .crypto_bars(&["BTC/USD"], TimeFrame::ONE_DAY, CryptoLocation::Us)
        .start(window_start)
        .end(window_end)
        .limit(1)
        .execute()
        .await
        .unwrap();
    let _ = crypto_bars;

    let latest_crypto_bars = client
        .crypto_latest_bars(&["BTC/USD"], CryptoLocation::Us)
        .await
        .unwrap();
    let _ = latest_crypto_bars;

    let latest_crypto_trades = client
        .crypto_latest_trades(&["BTC/USD"], CryptoLocation::Us)
        .await
        .unwrap();
    let _ = latest_crypto_trades;

    let latest_crypto_quotes = client
        .crypto_latest_quotes(&["BTC/USD"], CryptoLocation::Us)
        .await
        .unwrap();
    let _ = latest_crypto_quotes;

    let crypto_snapshots = client
        .crypto_snapshots(&["BTC/USD"], CryptoLocation::Us)
        .await
        .unwrap();
    let _ = crypto_snapshots;

    let crypto_orderbooks = client
        .crypto_latest_orderbooks(&["BTC/USD"], CryptoLocation::Us)
        .await
        .unwrap();
    let _ = crypto_orderbooks;

    // Options market data
    let chain = expect_ok_or_status(
        client.option_chain("AAPL").await,
        &[403, 404, 422],
        "option_chain",
    )
    .unwrap_or_default();

    let option_symbol = chain
        .keys()
        .next()
        .cloned()
        .unwrap_or_else(|| "AAPL250117C00150000".to_string());

    let _ = expect_ok_or_status(
        client
            .option_bars(&[&option_symbol], TimeFrame::ONE_DAY)
            .start(window_start)
            .end(window_end)
            .limit(1)
            .execute()
            .await,
        &[403, 404, 422],
        "option_bars",
    );

    let _ = expect_ok_or_status(
        client.option_latest_trades(&[&option_symbol]).await,
        &[403, 404, 422],
        "option_latest_trades",
    );

    let _ = expect_ok_or_status(
        client.option_latest_quotes(&[&option_symbol]).await,
        &[403, 404, 422],
        "option_latest_quotes",
    );

    let _ = expect_ok_or_status(
        client.option_snapshots(&[&option_symbol]).await,
        &[403, 404, 422],
        "option_snapshots",
    );

    // News / screener / reference data
    let news = client
        .news()
        .symbols(&["AAPL"])
        .start(window_start)
        .end(window_end)
        .limit(1)
        .execute()
        .await
        .unwrap();
    let _ = news;

    let most_actives = client.most_actives(Some(5), None).await.unwrap();
    let _ = most_actives;

    let movers = client
        .market_movers(MoverMarket::Stocks, Some(5))
        .await
        .unwrap();
    let _ = movers;

    let _ = expect_ok_or_status(client.logo("AAPL").await, &[403, 404, 422], "logo");

    let _ = expect_ok_or_status(
        client
            .corporate_actions()
            .symbols(&["AAPL"])
            .types(&[CorporateActionType::CashDividend])
            .execute()
            .await,
        &[403, 404, 422],
        "corporate_actions",
    );

    let _ = expect_ok_or_status(
        client.forex_latest_rates(&["EUR/USD"]).await,
        &[403, 404, 422],
        "forex_latest_rates",
    );

    let _ = expect_ok_or_status(
        client.fixed_income_latest_prices(&["US10Y"]).await,
        &[403, 404, 422],
        "fixed_income_latest_prices",
    );
}
