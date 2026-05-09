use std::time::{SystemTime, UNIX_EPOCH};

use oxidized_alpaca::{
    AccountType, Error, TradingClient,
    restful::trading::{
        orders::{OrderStatusFilter, OrderType, Side, TimeInForce},
        portfolio_history::{HistoryPeriod, HistoryTimeFrame},
    },
};
use rust_decimal::Decimal;

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

fn unique_suffix() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}

#[tokio::test]
async fn trading_endpoints_live_smoke() {
    let client = TradingClient::new(AccountType::Paper).unwrap();

    let account = client.get_account().await.unwrap();
    assert_eq!(
        account.currency,
        oxidized_alpaca::restful::trading::accounts::Currency::USD
    );

    let config = client.get_account_config().await.unwrap();
    let _ = client
        .update_account_config()
        .dtbp_check(config.dtbp_check)
        .execute()
        .await
        .unwrap();

    let recent_activities = client.list_activities().limit(1).execute().await.unwrap();
    if let Some(activity) = recent_activities.first() {
        let _ = expect_ok_or_status(
            client.get_activity(&activity.id).await,
            &[404, 422],
            "get_activity",
        );
    }

    let assets = client.list_assets().execute().await.unwrap();
    assert!(!assets.is_empty());

    client.get_asset("AAPL").await.unwrap();
    let _ = client.get_calendar().execute().await.unwrap();
    client.get_clock().await.unwrap();

    let contracts = expect_ok_or_status(
        client
            .list_option_contracts()
            .underlying_symbols(&["AAPL"])
            .limit(1)
            .execute()
            .await,
        &[403, 404, 422, 500],
        "list_option_contracts",
    )
    .unwrap_or_default();

    let option_symbol = contracts
        .first()
        .map(|contract| contract.symbol.clone())
        .unwrap_or_else(|| "AAPL250117C00150000".to_string());

    let _ = expect_ok_or_status(
        client.get_option_contract(&option_symbol).await,
        &[403, 404, 422, 500],
        "get_option_contract",
    );

    let order_client_id = format!("coverage-{}", unique_suffix());
    let order = expect_ok_or_status(
        client
            .create_order("AAPL", Side::Buy, OrderType::Limit)
            .qty(Decimal::from_str_exact("1").unwrap())
            .limit_price(Decimal::from_str_exact("100").unwrap())
            .time_in_force(TimeInForce::Gtc)
            .client_order_id(&order_client_id)
            .execute()
            .await,
        &[400, 403, 404, 422, 500],
        "create_order",
    );

    let _ = client
        .list_orders()
        .status(OrderStatusFilter::All)
        .limit(5)
        .execute()
        .await
        .unwrap();

    let order_id = order
        .as_ref()
        .map(|created| created.id.clone())
        .unwrap_or_else(|| "00000000-0000-0000-0000-000000000000".to_string());

    let _ = expect_ok_or_status(
        client.get_order(&order_id).await,
        &[404, 422, 500],
        "get_order",
    );
    let _ = expect_ok_or_status(
        client.get_order_by_client_id(&order_client_id).await,
        &[404, 422, 500],
        "get_order_by_client_id",
    );

    let _ = expect_ok_or_status(
        client
            .replace_order(&order_id)
            .limit_price(Decimal::from_str_exact("90").unwrap())
            .execute()
            .await,
        &[400, 403, 404, 422, 500],
        "replace_order",
    );

    let _ = expect_ok_or_status(
        client.cancel_order(&order_id).await,
        &[404, 422, 500],
        "cancel_order",
    );

    client.cancel_all_orders().await.unwrap();

    let portfolio = client
        .portfolio_history()
        .period(HistoryPeriod::OneDay)
        .timeframe(HistoryTimeFrame::OneMinute)
        .execute()
        .await
        .unwrap();
    let _ = portfolio;

    let _positions = client.list_positions().await.unwrap();

    let _ = expect_ok_or_status(
        client.get_position("INVALID_POSITION_SYMBOL").await,
        &[404, 422],
        "get_position",
    );
    let _ = expect_ok_or_status(
        client
            .close_position("INVALID_POSITION_SYMBOL")
            .qty(Decimal::from_str_exact("1").unwrap())
            .execute()
            .await,
        &[404, 422],
        "close_position",
    );

    let _ = client.close_all_positions().await.unwrap();

    let _ = expect_ok_or_status(
        client.exercise_option(&option_symbol).await,
        &[400, 403, 404, 422],
        "exercise_option",
    );
    let _ = expect_ok_or_status(
        client.do_not_exercise(&option_symbol).await,
        &[400, 403, 404, 422],
        "do_not_exercise",
    );

    let _ = client.list_watchlists().await.unwrap();
    let watchlist_name = format!("coverage-watchlist-{}", unique_suffix());
    let watchlist = expect_ok_or_status(
        client
            .create_watchlist(&watchlist_name)
            .symbols(&["AAPL"])
            .execute()
            .await,
        &[403, 404, 422],
        "create_watchlist",
    );

    let watchlist_id = watchlist
        .as_ref()
        .map(|created| created.id.clone())
        .unwrap_or_else(|| "00000000-0000-0000-0000-000000000000".to_string());

    let _ = expect_ok_or_status(
        client.get_watchlist(&watchlist_id).await,
        &[404, 422],
        "get_watchlist",
    );
    let _ = expect_ok_or_status(
        client
            .update_watchlist(&watchlist_id)
            .name(&(watchlist_name.clone() + "-updated"))
            .symbols(&["AAPL", "MSFT"])
            .execute()
            .await,
        &[400, 403, 404, 422],
        "update_watchlist",
    );
    let _ = expect_ok_or_status(
        client.add_to_watchlist(&watchlist_id, "GOOG").await,
        &[400, 403, 404, 422],
        "add_to_watchlist",
    );
    let _ = expect_ok_or_status(
        client.remove_from_watchlist(&watchlist_id, "GOOG").await,
        &[400, 403, 404, 422],
        "remove_from_watchlist",
    );
    let _ = expect_ok_or_status(
        client.delete_watchlist(&watchlist_id).await,
        &[404, 422],
        "delete_watchlist",
    );
}
