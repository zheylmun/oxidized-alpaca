use socketeer::{JsonCodec, Socketeer};
use std::collections::VecDeque;

use crate::{
    AccountType, Error,
    env::Env,
    streaming::messages::trade_update::{
        AuthorizationStatus, ListenStreams, TradeUpdate, TradingUpdatesMessage,
        TradingUpdatesRequest,
    },
};

macro_rules! info {
    ($($arg:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::info!($($arg)*);
    };
}
macro_rules! error {
    ($($arg:tt)*) => {
        #[cfg(feature = "tracing")]
        tracing::error!($($arg)*);
    };
}

const TRADING_UPDATES_LIVE_URL: &str = "wss://api.alpaca.markets/stream";
const TRADING_UPDATES_PAPER_URL: &str = "wss://paper-api.alpaca.markets/stream";

type TradingUpdatesSocket = Socketeer<JsonCodec<TradingUpdatesMessage, TradingUpdatesRequest>>;

/// Client for streaming order / fill / cancellation events from the Alpaca
/// trading API.
///
/// The trade-updates stream uses a different envelope and handshake from
/// the market-data feeds, so it has its own client rather than reusing
/// [`crate::streaming::StreamingClient`].
#[derive(Debug)]
pub struct TradingUpdatesClient {
    websocket: TradingUpdatesSocket,
    queue: VecDeque<TradeUpdate>,
}

impl TradingUpdatesClient {
    /// Connect to the trade-updates stream for `account_type` and complete
    /// the auth + `listen` handshake.
    pub async fn new(account_type: AccountType) -> Result<Self, Error> {
        let env = Env::new(&account_type)?;
        let url = match account_type {
            AccountType::Live => TRADING_UPDATES_LIVE_URL,
            AccountType::Paper => TRADING_UPDATES_PAPER_URL,
        };
        let mut websocket = TradingUpdatesSocket::connect(url).await?;

        websocket
            .send(TradingUpdatesRequest::Auth {
                key: env.key_id().to_string(),
                secret: env.secret_key().to_string(),
            })
            .await?;
        match Self::recv(&mut websocket).await? {
            TradingUpdatesMessage::Authorization(auth) => match auth.status {
                AuthorizationStatus::Authorized => {
                    info!("Authenticated with Alpaca trade-updates stream");
                }
                AuthorizationStatus::Unauthorized => {
                    error!("Trade-updates stream rejected credentials");
                    return Err(Error::StreamingAuth);
                }
            },
            other => {
                return Err(Error::UnexpectedConnectionMessage(format!("{other:?}")));
            }
        }

        websocket
            .send(TradingUpdatesRequest::Listen {
                data: ListenStreams {
                    streams: vec!["trade_updates".to_string()],
                },
            })
            .await?;
        match Self::recv(&mut websocket).await? {
            TradingUpdatesMessage::Listening(_) => {
                info!("Subscribed to trade_updates");
            }
            other => {
                return Err(Error::UnexpectedConnectionMessage(format!("{other:?}")));
            }
        }

        Ok(Self {
            websocket,
            queue: VecDeque::new(),
        })
    }

    /// Receive the next trade-update event. Drops handshake-level envelopes
    /// the server may resend.
    pub async fn next_trade_update(&mut self) -> Result<TradeUpdate, Error> {
        loop {
            if let Some(update) = self.queue.pop_front() {
                return Ok(update);
            }
            match Self::recv(&mut self.websocket).await? {
                TradingUpdatesMessage::TradeUpdate(update) => self.queue.push_back(update),
                TradingUpdatesMessage::Authorization(_) | TradingUpdatesMessage::Listening(_) => {
                    // re-emitted handshake envelopes; ignore.
                }
            }
        }
    }

    /// Close the WebSocket connection and shut down the client.
    pub async fn shut_down(self) -> Result<(), Error> {
        self.websocket.close_connection().await?;
        Ok(())
    }

    async fn recv(socket: &mut TradingUpdatesSocket) -> Result<TradingUpdatesMessage, Error> {
        socket.next_message().await.map_err(|e| {
            error!("Error retrieving next message: {e:?}");
            Error::from(e)
        })
    }
}
