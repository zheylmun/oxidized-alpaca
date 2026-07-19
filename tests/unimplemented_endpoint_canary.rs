//! Inverted canary for Alpaca endpoints that are documented but not served.
//!
//! Alpaca's market data OpenAPI spec describes five crypto perpetual futures
//! endpoints under `/v1beta1/crypto-perps/{loc}/latest/*`, but as of this
//! writing every one of them returns 404 in production while spot crypto on
//! the same host and credentials returns 200. There is also no perps page in
//! Alpaca's documentation index and `/v2/assets?asset_class=crypto_perp`
//! returns an empty list, so the endpoints appear to be an unfinished spec
//! entry rather than a live surface.
//!
//! Rather than periodically re-checking by hand, this test asserts the
//! endpoints are still absent. It **fails when they start working**, which
//! surfaces through the scheduled compatibility workflow as an issue. When
//! that happens the fix is to implement the endpoints (a branch already
//! exists) and delete this file.
//!
//! Deliberately uses raw HTTP rather than the crate: the whole point is to
//! probe a surface the crate does not implement.
#![cfg(feature = "restful")]

const PERP_ENDPOINTS: &[&str] = &[
    "https://data.alpaca.markets/v1beta1/crypto-perps/global/latest/bars",
    "https://data.alpaca.markets/v1beta1/crypto-perps/global/latest/trades",
    "https://data.alpaca.markets/v1beta1/crypto-perps/global/latest/quotes",
    "https://data.alpaca.markets/v1beta1/crypto-perps/global/latest/orderbooks",
    "https://data.alpaca.markets/v1beta1/crypto-perps/global/latest/pricing",
];

#[tokio::test]
async fn crypto_perps_endpoints_are_still_unavailable() {
    let (key, secret) = match (
        std::env::var("ALPACA_PAPER_API_KEY_ID"),
        std::env::var("ALPACA_PAPER_API_SECRET_KEY"),
    ) {
        (Ok(key), Ok(secret)) if !key.is_empty() && !secret.is_empty() => (key, secret),
        // Mirrors the other live tests: without credentials there is nothing
        // meaningful to assert, and failing here would be a config problem
        // rather than API drift.
        _ => return,
    };

    let client = reqwest::Client::new();
    let mut now_available = Vec::new();

    for endpoint in PERP_ENDPOINTS {
        let response = client
            .get(*endpoint)
            .query(&[("symbols", "BTCUSDT.P")])
            .header("APCA-API-KEY-ID", &key)
            .header("APCA-API-SECRET-KEY", &secret)
            .send()
            .await
            .unwrap_or_else(|err| panic!("{endpoint} could not be reached: {err}"));

        if response.status() != reqwest::StatusCode::NOT_FOUND {
            now_available.push(format!("{endpoint} -> {}", response.status()));
        }
    }

    assert!(
        now_available.is_empty(),
        "Alpaca now serves crypto perpetual futures endpoints that this crate \
         does not implement:\n  {}\n\nImplement them (see the crypto_perps \
         work) and delete tests/unimplemented_endpoint_canary.rs.",
        now_available.join("\n  "),
    );
}
