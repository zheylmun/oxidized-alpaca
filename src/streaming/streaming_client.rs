use crate::{
    AccountType, Env, Error, Feed,
    streaming::stock_data::{self, ControlMessage, Request, StreamMessage, SubscriptionList},
};
use socketeer::Socketeer;
use std::collections::VecDeque;
#[cfg(feature = "tracing")]
use tracing::{error, info};

#[derive(Debug)]
pub struct StreamingMarketDataClient<RxMessage, TxMessage> {
    websocket: Socketeer<RxMessage, TxMessage>,
    messages: VecDeque<StreamMessage>,
    subscriptions: SubscriptionList,
}

impl StreamingMarketDataClient<Vec<stock_data::StreamMessage>, stock_data::Request> {
    pub async fn new_test_client(
        account_type: AccountType,
    ) -> Result<StreamingMarketDataClient<Vec<StreamMessage>, Request>, Error> {
        let env = Env::new(&account_type)?;
        let websocket: Socketeer<Vec<StreamMessage>, Request> =
            Socketeer::connect(Feed::Test.streaming_url(account_type)).await?;
        Self::initialize_with_websocket(env, websocket).await
    }

    pub async fn new_iex_client(
        account_type: AccountType,
    ) -> Result<StreamingMarketDataClient<Vec<StreamMessage>, Request>, Error> {
        let env = Env::new(&account_type)?;
        let websocket: Socketeer<Vec<StreamMessage>, Request> =
            Socketeer::connect(Feed::IEX.streaming_url(account_type)).await?;
        Self::initialize_with_websocket(env, websocket).await
    }

    pub async fn new_sip_client(
        account_type: AccountType,
    ) -> Result<StreamingMarketDataClient<Vec<StreamMessage>, Request>, Error> {
        let env = Env::new(&account_type)?;
        let websocket: Socketeer<Vec<StreamMessage>, Request> =
            Socketeer::connect(Feed::SIP.streaming_url(account_type)).await?;
        Self::initialize_with_websocket(env, websocket).await
    }

    async fn initialize_with_websocket(
        env: Env,
        websocket: Socketeer<Vec<StreamMessage>, Request>,
    ) -> Result<StreamingMarketDataClient<Vec<StreamMessage>, Request>, Error> {
        let mut client = StreamingMarketDataClient {
            websocket,
            messages: VecDeque::new(),
            subscriptions: SubscriptionList::new(),
        };
        // Wait for the server to confirm our connection
        let connection_confirmation = client.next_message_internal().await?;
        // Make sure we get the connection confirmation message
        if let Some(ControlMessage::Connected) = connection_confirmation.control() {
            info!("Connected to Alpaca Streaming API");
        } else {
            return Err(Error::UnexpectedConnectionMessage(Box::new(
                connection_confirmation,
            )));
        }

        // Send our auth information
        client
            .websocket
            .send(Request::AuthMessage {
                key: env.key_id().to_string(),
                secret: env.secret_key().to_string(),
            })
            .await?;
        // Await the authentication response
        let auth_response = client.next_message_internal().await?;
        if let Some(ControlMessage::Connected) = auth_response.control() {
            info!("Authenticated with Alpaca Streaming API");
        }
        Ok(client)
    }

    pub async fn next_message(&mut self) -> Result<StreamMessage, Error> {
        let mut message: Option<StreamMessage> = None;
        while message.is_none() {
            let incoming_message = self.next_message_internal().await?;
            message = self.handle_subscription_update(incoming_message);
        }
        Ok(message.unwrap())
    }

    pub async fn add_subscriptions(
        &mut self,
        subscriptions: &SubscriptionList,
    ) -> Result<SubscriptionList, Error> {
        self.websocket
            .send(Request::Subscribe(subscriptions.clone()))
            .await?;
        self.await_subscription_update_message().await?;
        Ok(self.subscriptions.clone())
    }

    pub async fn remove_subscriptions(
        &mut self,
        subscriptions: &SubscriptionList,
    ) -> Result<SubscriptionList, Error> {
        self.websocket
            .send(Request::Unsubscribe(subscriptions.clone()))
            .await?;
        self.await_subscription_update_message().await?;
        Ok(self.subscriptions.clone())
    }

    pub async fn shut_down(self) -> Result<(), Error> {
        self.websocket.close_connection().await?;
        Ok(())
    }

    /// Pull messages from the socket until we receive a control message
    /// normal messages go into our message queue
    async fn await_subscription_update_message(&mut self) -> Result<(), Error> {
        let mut received = false;
        while !received {
            match self.websocket.next_message().await {
                Ok(messages) => {
                    for message in messages {
                        match self.handle_subscription_update(message) {
                            None => {
                                received = true;
                            }
                            Some(message) => {
                                self.messages.push_back(message);
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("Error retrieving next message: {e:?}");
                    return Err(Error::WebsocketError(e));
                }
            }
        }
        Ok(())
    }

    async fn next_message_internal(&mut self) -> Result<StreamMessage, Error> {
        if self.messages.is_empty() {
            match self.websocket.next_message().await {
                Ok(messages) => self.messages.extend(messages),
                Err(e) => {
                    error!("Error retrieving next message: {e:?}");
                    return Err(Error::WebsocketError(e));
                }
            }
        }
        Ok(self.messages.pop_front().unwrap())
    }

    fn handle_subscription_update(&mut self, message: StreamMessage) -> Option<StreamMessage> {
        match message {
            StreamMessage::Subscription(updated_subsciptions) => {
                self.subscriptions = updated_subsciptions;
                None
            }
            _ => Some(message),
        }
    }
}
