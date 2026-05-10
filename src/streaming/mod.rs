/// Wire-shape types for each streaming feed (stocks, crypto, news, options,
/// and trade updates). The most commonly used items — the per-feed
/// `…StreamMessage` enum and `…SubscriptionList` builder — are also re-exported
/// at [`crate::streaming`]; reach for the deeper path here when you need the
/// individual event payload structs.
pub mod messages;
pub use messages::{
    CryptoStreamMessage, CryptoSubscriptionList, NewsStreamMessage, NewsSubscriptionList,
    OptionStreamMessage, OptionSubscriptionList, StockStreamMessage, StockSubscriptionList,
    TradeUpdate, TradingUpdatesMessage,
};

mod wire;
pub use wire::{ControlMessage, StreamError, StreamErrorCode};

mod client;
pub use client::{StreamProtocol, StreamingClient};

mod stock_client;
pub use stock_client::{StockProtocol, StreamingStockClient};

mod crypto_client;
pub use crypto_client::{CryptoProtocol, StreamingCryptoClient};

mod news_client;
pub use news_client::{NewsProtocol, StreamingNewsClient};

mod option_client;
pub use option_client::{OptionProtocol, StreamingOptionClient};

mod trading_updates_client;
pub use trading_updates_client::TradingUpdatesClient;
