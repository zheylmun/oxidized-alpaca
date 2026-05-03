use oxidized_alpaca::{
    AccountType, Error, MarketDataClient,
    restful::market_data::{
        crypto::CryptoLocation,
        stock::{
            TimeFrame,
            meta::{Tape, TickType},
        },
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

    // Stocks
    let bars = client
        .stock_bars("AAPL", TimeFrame::OneDay)
        .limit(1)
        .execute()
        .await
        .unwrap();
    let _ = bars;

    let trades = client
        .stock_trades("AAPL")
        .limit(1)
        .execute()
        .await
        .unwrap();
    let _ = trades;

    client.stock_latest_trade("AAPL").await.unwrap();

    let latest_trades = client.stock_latest_trades(&["AAPL", "MSFT"]).await.unwrap();
    let _ = latest_trades;

    let quotes = client
        .stock_quotes("AAPL")
        .limit(1)
        .execute()
        .await
        .unwrap();
    let _ = quotes;

    client.stock_latest_quote("AAPL").await.unwrap();

    let latest_quotes = client.stock_latest_quotes(&["AAPL", "MSFT"]).await.unwrap();
    let _ = latest_quotes;

    let _ = expect_ok_or_status(
        client.stock_auctions("AAPL").limit(1).execute().await,
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
        .crypto_bars("BTC/USD", "1Day", CryptoLocation::Us)
        .limit(1)
        .execute()
        .await
        .unwrap();
    let _ = crypto_bars;

    let latest_crypto_bars = client
        .crypto_latest_bars("BTC/USD", CryptoLocation::Us)
        .await
        .unwrap();
    let _ = latest_crypto_bars;

    let latest_crypto_trades = client
        .crypto_latest_trades("BTC/USD", CryptoLocation::Us)
        .await
        .unwrap();
    let _ = latest_crypto_trades;

    let latest_crypto_quotes = client
        .crypto_latest_quotes("BTC/USD", CryptoLocation::Us)
        .await
        .unwrap();
    let _ = latest_crypto_quotes;

    let crypto_snapshots = client
        .crypto_snapshots("BTC/USD", CryptoLocation::Us)
        .await
        .unwrap();
    let _ = crypto_snapshots;

    let crypto_orderbooks = client
        .crypto_latest_orderbooks("BTC/USD", CryptoLocation::Us)
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
            .option_bars(&option_symbol, "1Day")
            .limit(1)
            .execute()
            .await,
        &[403, 404, 422],
        "option_bars",
    );

    let _ = expect_ok_or_status(
        client.option_latest_trades(&option_symbol).await,
        &[403, 404, 422],
        "option_latest_trades",
    );

    let _ = expect_ok_or_status(
        client.option_latest_quotes(&option_symbol).await,
        &[403, 404, 422],
        "option_latest_quotes",
    );

    let _ = expect_ok_or_status(
        client.option_snapshots(&option_symbol).await,
        &[403, 404, 422],
        "option_snapshots",
    );

    // News / screener / reference data
    let news = client
        .news()
        .symbols("AAPL")
        .limit(1)
        .execute()
        .await
        .unwrap();
    let _ = news;

    let most_actives = client.most_actives(Some(5)).await.unwrap();
    let _ = most_actives;

    let movers = client.market_movers("stocks", Some(5)).await.unwrap();
    let _ = movers;

    let _ = expect_ok_or_status(client.logo("AAPL").await, &[403, 404, 422], "logo");

    let _ = expect_ok_or_status(
        client
            .corporate_actions(Some("AAPL"), Some("cash_dividend"))
            .await,
        &[403, 404, 422],
        "corporate_actions",
    );

    let _ = expect_ok_or_status(
        client.forex_latest_rates("EUR/USD").await,
        &[403, 404, 422],
        "forex_latest_rates",
    );

    let _ = expect_ok_or_status(
        client.fixed_income_latest_prices("US10Y").await,
        &[403, 404, 422],
        "fixed_income_latest_prices",
    );
}
