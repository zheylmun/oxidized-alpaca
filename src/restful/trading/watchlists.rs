use crate::restful::TradingClient;
use crate::{AccountId, WatchlistId};
use chrono::{DateTime, Utc};
use reqwest::Method;
use serde::{Deserialize, Serialize};

use super::assets::Asset;

/// A watchlist as returned by the Alpaca API.
#[derive(Clone, Debug, Deserialize)]
#[non_exhaustive]
pub struct Watchlist {
    /// Watchlist ID.
    pub id: WatchlistId,
    /// Account ID that owns this watchlist.
    pub account_id: AccountId,
    /// Timestamp when the watchlist was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the watchlist was last updated.
    pub updated_at: DateTime<Utc>,
    /// Watchlist name.
    pub name: String,
    /// Assets in the watchlist.
    #[serde(default)]
    pub assets: Option<Vec<Asset>>,
}

/// Builder for creating a watchlist.
#[derive(Debug, Serialize)]
#[must_use]
pub struct CreateWatchlistRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbols: Option<Vec<String>>,
}

impl CreateWatchlistRequest<'_> {
    /// Add symbols to the new watchlist.
    pub fn symbols(mut self, symbols: &[&str]) -> Self {
        self.symbols = Some(symbols.iter().map(|s| (*s).to_string()).collect());
        self
    }

    /// Submit the create request.
    pub async fn execute(self) -> crate::Result<Watchlist> {
        let request = self
            .client
            .request(Method::POST, "v2/watchlists")?
            .json(&self);
        self.client.send_and_deserialize(request).await
    }
}

/// Builder for updating a watchlist.
#[derive(Debug, Serialize)]
#[must_use]
pub struct UpdateWatchlistRequest<'a> {
    #[serde(skip)]
    client: &'a TradingClient,
    #[serde(skip)]
    watchlist_id: WatchlistId,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    symbols: Option<Vec<String>>,
}

impl UpdateWatchlistRequest<'_> {
    /// Set the new name.
    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Set the new list of symbols.
    pub fn symbols(mut self, symbols: &[&str]) -> Self {
        self.symbols = Some(symbols.iter().map(|s| (*s).to_string()).collect());
        self
    }

    /// Submit the update.
    pub async fn execute(self) -> crate::Result<Watchlist> {
        let watchlist_id = &self.watchlist_id;
        let path = format!("v2/watchlists/{watchlist_id}");
        let request = self.client.request(Method::PUT, &path)?.json(&self);
        self.client.send_and_deserialize(request).await
    }
}

impl TradingClient {
    /// List all watchlists.
    pub async fn list_watchlists(&self) -> crate::Result<Vec<Watchlist>> {
        let request = self.request(Method::GET, "v2/watchlists")?;
        self.send_and_deserialize(request).await
    }

    /// Get a watchlist by ID.
    pub async fn get_watchlist(&self, watchlist_id: &WatchlistId) -> crate::Result<Watchlist> {
        let request = self.request(Method::GET, &format!("v2/watchlists/{watchlist_id}"))?;
        self.send_and_deserialize(request).await
    }

    /// Create a new watchlist.
    ///
    /// ```ignore
    /// let wl = client.create_watchlist("Tech")
    ///     .symbols(&["AAPL", "GOOG", "MSFT"])
    ///     .execute().await?;
    /// ```
    pub fn create_watchlist(&self, name: &str) -> CreateWatchlistRequest<'_> {
        CreateWatchlistRequest {
            client: self,
            name: name.to_string(),
            symbols: None,
        }
    }

    /// Update an existing watchlist.
    pub fn update_watchlist(&self, watchlist_id: &WatchlistId) -> UpdateWatchlistRequest<'_> {
        UpdateWatchlistRequest {
            client: self,
            watchlist_id: watchlist_id.clone(),
            name: None,
            symbols: None,
        }
    }

    /// Add a symbol to an existing watchlist.
    pub async fn add_to_watchlist(
        &self,
        watchlist_id: &WatchlistId,
        symbol: &str,
    ) -> crate::Result<Watchlist> {
        #[derive(Serialize)]
        struct Body {
            symbol: String,
        }
        let request = self
            .request(Method::POST, &format!("v2/watchlists/{watchlist_id}"))?
            .json(&Body {
                symbol: symbol.to_string(),
            });
        self.send_and_deserialize(request).await
    }

    /// Remove a symbol from a watchlist.
    pub async fn remove_from_watchlist(
        &self,
        watchlist_id: &WatchlistId,
        symbol: &str,
    ) -> crate::Result<Watchlist> {
        let request = self.request(
            Method::DELETE,
            &format!("v2/watchlists/{watchlist_id}/{symbol}"),
        )?;
        self.send_and_deserialize(request).await
    }

    /// Delete a watchlist.
    pub async fn delete_watchlist(&self, watchlist_id: &WatchlistId) -> crate::Result<()> {
        let request = self.request(Method::DELETE, &format!("v2/watchlists/{watchlist_id}"))?;
        let response = request.send().await.map_err(crate::Error::ReqwestSend)?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(crate::Error::ApiError {
                status: status.as_u16(),
                body,
            });
        }
        Ok(())
    }
}
