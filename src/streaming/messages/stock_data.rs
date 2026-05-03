use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// List of subscriptions to market data types available for streaming clients
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SubscriptionList {
    /// List of symbols for minute bars subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bars: Option<Vec<String>>,
    /// List of symbols for daily bars subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub daily_bars: Option<Vec<String>>,
    /// List of symbols for bars subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_bars: Option<Vec<String>>,
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
            daily_bars: None,
            updated_bars: None,
            quotes: None,
            trades: None,
            news: None,
        }
    }

    /// Add a symbol to the minute bars subscription list
    pub fn add_minute_bars(mut self, symbol: &str) -> Self {
        if let Some(bars) = &mut self.bars {
            if !bars.contains(&symbol.to_string()) {
                bars.push(symbol.to_string());
            }
        } else {
            self.bars = Some(vec![symbol.to_string()]);
        }
        self
    }

    /// Add a symbol to the daily bars subscription list
    pub fn add_daily_bars(mut self, symbol: &str) -> Self {
        if let Some(bars) = &mut self.daily_bars {
            if !bars.contains(&symbol.to_string()) {
                bars.push(symbol.to_string());
            }
        } else {
            self.daily_bars = Some(vec![symbol.to_string()]);
        }
        self
    }

    /// Add a symbol to the minute bars subscription list
    pub fn add_updated_bars(mut self, symbol: &str) -> Self {
        if let Some(bars) = &mut self.updated_bars {
            if !bars.contains(&symbol.to_string()) {
                bars.push(symbol.to_string());
            }
        } else {
            self.updated_bars = Some(vec![symbol.to_string()]);
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
    #[must_use]
    pub fn add_trades(mut self, symbol: &str) -> Self {
        if let Some(trades) = &mut self.trades {
            trades.push(symbol.to_string());
        } else {
            self.trades = Some(vec![symbol.to_string()]);
        }
        self
    }

    /// Add a symbol to the news subscription list
    #[must_use]
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

/// Server control message indicating connection or authentication success.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ControlMessage {
    /// Connection to the streaming server was successful.
    Connected,
    /// Authentication was successful.
    Authenticated,
}
/// Error codes returned by the streaming API.
#[derive(Clone, Debug, Deserialize_repr, Serialize_repr)]
#[repr(u16)]
pub enum ErrorCode {
    /// The request had invalid syntax.
    InvalidSyntax = 400,
    /// The client is not authenticated.
    NotAuthenticated = 401,
    /// Authentication credentials were rejected.
    AuthFailed = 402,
    /// The client is already authorized.
    AlreadyAuthorized = 403,
}

/// Error message from the streaming API.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Error {
    /// Error code indicating the type of error.
    pub code: ErrorCode,
    /// Human-readable error message.
    #[serde(rename = "msg")]
    pub message: String,
}

/// OHLCV bar for a stock symbol.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Bar {
    /// Ticker symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Opening price.
    #[serde(rename = "o")]
    pub open: f64,
    /// High price.
    #[serde(rename = "h")]
    pub high: f64,
    /// Low price.
    #[serde(rename = "l")]
    pub low: f64,
    /// Closing price.
    #[serde(rename = "c")]
    pub close: f64,
    /// Trade volume.
    #[serde(rename = "v")]
    pub volume: i64,
    /// Bar timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
}

/// Real-time quote with bid and ask data.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Quote {
    /// Ticker symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Ask exchange code.
    #[serde(rename = "ax")]
    pub ask_exchange: Option<String>,
    /// Ask price.
    #[serde(rename = "ap")]
    pub ask_price: f64,
    /// Ask size.
    #[serde(rename = "as")]
    pub ask_size: f64,
    /// Bid exchange code.
    #[serde(rename = "bx")]
    pub bid_exchange: Option<String>,
    /// Bid price.
    #[serde(rename = "bp")]
    pub bid_price: f64,
    /// Bid size.
    #[serde(rename = "bs")]
    pub bid_size: f64,
    /// Trade size.
    #[serde(rename = "s")]
    pub trade_size: Option<f64>,
    /// Quote timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Tape identifier.
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// Real-time trade event.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Trade {
    /// Ticker symbol.
    #[serde(rename = "S")]
    pub symbol: String,
    /// Trade ID.
    #[serde(rename = "i")]
    pub trade_id: i64,
    /// Exchange code where the trade occurred.
    #[serde(rename = "x")]
    pub exchange: Option<String>,
    /// Trade price.
    #[serde(rename = "p")]
    pub price: f64,
    /// Trade size.
    #[serde(rename = "s")]
    pub size: f64,
    /// Trade timestamp.
    #[serde(rename = "t")]
    pub timestamp: DateTime<Utc>,
    /// Trade condition flags.
    #[serde(rename = "c")]
    pub conditions: Option<Vec<String>>,
    /// Tape identifier.
    #[serde(rename = "z")]
    pub tape: Option<String>,
}

/// The following represent messages we can listen for
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "T")]
pub enum StreamMessage {
    /// Internally consumed stream acknowledging successful completion of requests
    #[serde(rename = "success")]
    Control {
        /// The control message payload.
        msg: ControlMessage,
    },
    /// Error message from the server.
    #[serde(rename = "error")]
    Error(Error),
    /// Subscription confirmation with the current subscription list.
    #[serde(rename = "subscription")]
    Subscription(SubscriptionList),
    /// Minute bar update.
    #[serde(rename = "b")]
    Bar(Bar),
    /// Daily bar update.
    #[serde(rename = "d")]
    DailyBar(Bar),
    /// Updated (corrected) bar.
    #[serde(rename = "u")]
    UpdatedBar(Bar),
    /// Trade event.
    #[serde(rename = "t")]
    Trade(Trade),
    /// Quote update.
    #[serde(rename = "q")]
    Quote(Quote),
}

impl StreamMessage {
    pub(crate) const fn control(&self) -> Option<&ControlMessage> {
        match self {
            StreamMessage::Control { msg } => Some(msg),
            _ => None,
        }
    }
}

/// Outgoing wire-protocol message used by the streaming client to talk
/// to the Alpaca server. Crate-internal: callers go through
/// [`StreamingMarketDataClient::add_subscriptions`] /
/// [`remove_subscriptions`] instead of constructing these by hand.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "action")]
pub(crate) enum Request {
    /// Authenticate with API key and secret.
    #[serde(rename = "auth")]
    AuthMessage {
        /// API key ID.
        key: String,
        /// API secret key.
        secret: String,
    },
    /// Subscribe to market data streams.
    #[serde(rename = "subscribe")]
    Subscribe(SubscriptionList),
    /// Unsubscribe from market data streams.
    #[serde(rename = "unsubscribe")]
    Unsubscribe(SubscriptionList),
}
