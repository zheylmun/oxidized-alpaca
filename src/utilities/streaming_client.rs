use futures::{future, Stream};
use futures_util::{SinkExt, StreamExt};
use std::sync::{
    mpsc::{self, Sender},
    Arc, Mutex,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{event, Level};

use crate::{env::Env, error::Error, market_data::Request, AccountType};

#[derive(Debug)]
pub struct StreamingClient {
    pub(crate) env: Env,
    pub url: url::Url,
    send_channel: Option<Sender<Message>>,
    shutdown_signal: Arc<Mutex<bool>>,
}

impl StreamingClient {
    /// Create a new [`StreamingClient`] instance with the given [`AccountType`] and client url
    ///
    /// # Errors
    ///
    /// - This function will return an error if the required environment variables are not set
    #[tracing::instrument]
    pub(crate) fn new(
        account_type: &AccountType,
        client_url: &str,
    ) -> Result<StreamingClient, Error> {
        let env = Env::new(account_type)?;
        let url = url::Url::parse(client_url).map_err(|e| Error::UrlParse(e))?;
        Ok(StreamingClient {
            env,
            url,
            send_channel: None,
            shutdown_signal: Arc::new(Mutex::new(false)),
        })
    }

    /// Initialize the websocket connection
    #[tracing::instrument]
    pub(crate) async fn connect(&mut self) -> Result<impl Stream<Item = String>, Error> {
        let (socket, response) = connect_async(&self.url)
            .await
            .map_err(|e| Error::TungsteniteConnection(e))?;

        assert_eq!(response.status(), 101);
        let (mut sink, source) = socket.split();

        let (tx, rx) = mpsc::channel();
        self.send_channel = Some(tx.clone());

        let shutdown = self.shutdown_signal.clone();
        tokio::spawn(async move {
            loop {
                // If shutdown is true, break out of the loop and end this threads execution
                if *(shutdown.lock().unwrap()) {
                    event!(
                        Level::INFO,
                        "Send thread received shutdown signal, exiting..."
                    );
                    break;
                }

                // If we have a message, send it to the websocket
                let result = rx.recv_timeout(std::time::Duration::from_millis(100));
                if let Ok(message) = result {
                    sink.send(message).await.unwrap();
                }
            }
        });
        // Next - set up our stream & remap stuff coming in
        let shutdown = self.shutdown_signal.clone();

        let stream = source.filter_map(move |msg| {
            match msg.unwrap() {
                Message::Ping(_) => {
                    tx.send(Message::Pong("pong".as_bytes().to_vec())).unwrap();
                }
                Message::Close(_) => {
                    *(shutdown.lock().unwrap()) = true;
                }
                Message::Text(value) => {
                    return future::ready(Some(value));
                }
                Message::Binary(value) => {
                    return future::ready(Some(String::from_utf8(value).unwrap()));
                }
                _ => {}
            };
            future::ready(None)
        });
        Ok(stream)
    }

    /// Stops the stream of events
    pub fn shutdown(&mut self) {
        event!(Level::INFO, "Sending shutdown signal...");
        let mut shutdown = self.shutdown_signal.lock().unwrap();
        *shutdown = true;

        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    pub fn send(&self, request: Request) {
        if let Some(channel) = &self.send_channel {
            let request_string = serde_json::to_string(&request).unwrap();
            println!("Sending: {}", request_string);
            let message = Message::Text(request_string);
            channel.send(message).unwrap();
        }
    }
}