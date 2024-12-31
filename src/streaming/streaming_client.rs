use crate::{
    streaming::{
        stock_data::{self, ControlMessage, Request, StreamMessage, SubscriptionList},
        ClientState,
    },
    AccountType, Env, Error, Feed,
};
use serde::{Deserialize, Serialize};
use socketeer::Socketeer;
use std::{collections::VecDeque, fmt};

#[derive(Debug)]
pub struct StreamingMarketDataClient<
    RxMessage: for<'a> Deserialize<'a> + fmt::Debug,
    TxMessage: fmt::Debug + Serialize,
> {
    websocket: Socketeer<RxMessage, TxMessage>,
    messages: VecDeque<StreamMessage>,
    state: ClientState<SubscriptionList>,
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
            state: ClientState::Connecting,
        };
        let connection_confirmation = client.next_message_internal().await?;
        assert!(client
            .handle_control_messages(connection_confirmation)
            .is_none());
        client
            .websocket
            .send(Request::AuthMessage {
                key: env.key_id().to_string(),
                secret: env.secret_key().to_string(),
            })
            .await?;
        let auth_response = client.next_message_internal().await?;
        assert!(client.handle_control_messages(auth_response).is_none());
        assert!(matches!(client.state, ClientState::Authenticated(_)));
        Ok(client)
    }

    pub async fn next_message(&mut self) -> Result<StreamMessage, Error> {
        let mut message: Option<StreamMessage> = None;
        while message.is_none() {
            let incoming_message = self.next_message_internal().await?;
            message = self.handle_control_messages(incoming_message);
        }
        Ok(message.unwrap())
    }

    pub async fn add_subscriptions(
        &mut self,
        subscriptions: &SubscriptionList,
    ) -> Result<(), Error> {
        self.websocket
            .send(Request::Subscribe(subscriptions.clone()))
            .await?;
        Ok(())
    }

    pub async fn remove_subscriptions(
        &mut self,
        subscriptions: &SubscriptionList,
    ) -> Result<(), Error> {
        self.websocket
            .send(Request::Unsubscribe(subscriptions.clone()))
            .await?;
        Ok(())
    }

    async fn next_message_internal(&mut self) -> Result<StreamMessage, Error> {
        if self.messages.is_empty() {
            let messages = self.websocket.next_message().await?;
            self.messages.extend(messages);
        }
        Ok(self.messages.pop_front().unwrap())
    }

    fn handle_control_messages(&mut self, message: StreamMessage) -> Option<StreamMessage> {
        match message {
            StreamMessage::Control { msg } => match msg {
                ControlMessage::Connected => {
                    self.state = ClientState::Connected;
                    None
                }
                ControlMessage::Authenticated => {
                    self.state = ClientState::Authenticated(SubscriptionList::new());
                    None
                }
            },
            StreamMessage::Subscription(updated_subsciptions) => {
                if let ClientState::Authenticated(subscriptions) = &mut self.state {
                    *subscriptions = updated_subsciptions;
                    None
                } else {
                    unreachable!("Received subscription update in unexpected state");
                }
            }
            _ => Some(message),
        }
    }
}
