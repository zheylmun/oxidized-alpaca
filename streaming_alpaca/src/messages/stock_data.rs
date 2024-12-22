use serde::{Deserialize, Serialize};

/// List of subscriptions to market data types available for streaming clients
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SubscriptionList {
    /// List of symbols for bars subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bars: Option<Vec<String>>,
    /// List of symbols for quotes subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quotes: Option<Vec<String>>,
    /// List of symbols for trades subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trades: Option<Vec<String>>,
    /// List of symbols for news subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub news: Option<Vec<String>>,
}

impl SubscriptionList {
    /// Create a new SubscriptionList
    pub fn new() -> Self {
        Self {
            bars: None,
            quotes: None,
            trades: None,
            news: None,
        }
    }

    /// Add a symbol to the bars subscription list
    pub fn add_bars(mut self, symbol: &str) -> Self {
        if let Some(bars) = &mut self.bars {
            bars.push(symbol.to_string());
        } else {
            self.bars = Some(vec![symbol.to_string()]);
        }
        self
    }

    /// Add a symbol to the quotes subscription list
    pub fn add_quotes(mut self, symbol: &str) -> Self {
        if let Some(quotes) = &mut self.quotes {
            quotes.push(symbol.to_string());
        } else {
            self.quotes = Some(vec![symbol.to_string()]);
        }
        self
    }

    /// Add a symbol to the trades subscription list
    pub fn add_trades(mut self, symbol: &str) -> Self {
        if let Some(trades) = &mut self.trades {
            trades.push(symbol.to_string());
        } else {
            self.trades = Some(vec![symbol.to_string()]);
        }
        self
    }

    /// Add a symbol to the news subscription list
    pub fn add_news(mut self, symbol: &str) -> Self {
        if let Some(news) = &mut self.news {
            news.push(symbol.to_string());
        } else {
            self.news = Some(vec![symbol.to_string()]);
        }
        self
    }
}

impl Default for SubscriptionList {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlMessage {
    Connected,
    Authenticated,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename = "lowercase", tag = "code")]
pub enum Error {
    #[serde(rename = "400")]
    InvalidSyntax,
    #[serde(rename = "401")]
    NotAuthenticated,
    #[serde(rename = "402")]
    AuthFailed,
    #[serde(rename = "403")]
    AlreadyAuthorized,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Quote {
    #[serde(rename = "S")]
    pub symbol: String,
    #[serde(rename = "ax")]
    pub ask_exchange: Option<String>,
    #[serde(rename = "ap")]
    pub ask_price: f64,
    #[serde(rename = "as")]
    pub ask_size: f64,
    #[serde(rename = "bx")]
    pub bid_exchange: Option<String>,
    #[serde(rename = "bp")]
    pub bid_price: f64,
    #[serde(rename = "bs")]
    pub bid_size: f64,
    #[serde(rename = "s")]
    pub trade_size: Option<f64>,
    #[serde(rename = "t")]
    pub timestamp: String,
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Trade {
    #[serde(rename = "S")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub trade_id: i64,
    #[serde(rename = "x")]
    pub exchange: Option<String>,
    #[serde(rename = "p")]
    pub price: f64,
    #[serde(rename = "s")]
    pub size: f64,
    #[serde(rename = "t")]
    pub timestamp: String,
    #[serde(rename = "c")]
    pub conditions: Option<Vec<String>>,
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// The following represent messages we can listen for
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "T")]
pub enum StreamMessage {
    /// Internally consumed stream acknowledging successful completion of requests
    #[serde(rename = "success")]
    Control { msg: ControlMessage },
    #[serde(rename = "error")]
    Error(Error),
    #[serde(rename = "subscription")]
    Subscription(SubscriptionList),
    #[serde(rename = "t")]
    Trade(Trade),
    #[serde(rename = "q")]
    Quote(Quote),
}

/// Streaming Authentication Message
#[derive(Debug, Serialize)]
#[serde(tag = "action")]
pub enum Request {
    #[serde(rename = "auth")]
    AuthMessage { key: String, secret: String },
    #[serde(rename = "subscribe")]
    Subscribe(SubscriptionList),
    #[serde(rename = "unsubscribe")]
    Unsubscribe(SubscriptionList),
}
