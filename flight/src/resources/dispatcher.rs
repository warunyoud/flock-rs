use actix::{
    Actor, AsyncContext, Context, Handler, Message, Addr,
    prelude::ResponseFuture,
};
use std::{
    collections::HashMap,
    sync::Arc
};
use log::warn;
use serde_json::{json, Value, to_string};
use watchtower_client::WatchtowerClient;

use crate::{
    resources::{SubscriptionTable, MyWs, ws::WsMessage},
    utils::env,
    types::{Event, Result}
};

pub enum DispatcherMessage {
    Event(Event),
    RegisterWS {
        addr: Addr<MyWs>,
        socket_id: String
    },
    Subscribe {
        socket_id: String,
        topic: String,
        request_id: String 
    },
    Unsubscribe {
        socket_id: String,
        topic: String,
        request_id: String 
    },
    Close(String),
    Reset
}

impl Message for DispatcherMessage {
    type Result = Result<bool>;
}

pub struct Dispatcher {
    http_client: Arc<reqwest::Client>,
    watchtower_client: Arc<WatchtowerClient>,
    subscription_table: SubscriptionTable,
    ws_table: HashMap<String, Arc<Addr<MyWs>>>,
    auth: env::AuthInfo
}

const TRAFFIC_CONTROL_SERVICE_ID: &str = "traffic-control";

impl Dispatcher {
    pub fn new() -> Dispatcher {
        let watchtower_config = env::get_watchtower_config();
        Dispatcher {
            http_client: Arc::new(reqwest::Client::new()),
            watchtower_client: Arc::new(WatchtowerClient::new(watchtower_config.urls, &watchtower_config.username, &watchtower_config.password)),
            subscription_table: SubscriptionTable::new(),
            ws_table: HashMap::new(),
            auth: env::get_auth_info()
        }
    }

    fn broadcast_event(&self, topic: &str, message: &str) -> ResponseFuture<Result<bool>> {
        match serde_json::from_str(message) {
            Ok(payload) => {
                let payload: Value = payload;
                let value = json!({
                    "topic": topic,
                    "payload": payload,
                    "type": "event"
                });
        
                let socket_ids = self.subscription_table.get(topic);
        
                let mut sockets = Vec::new();
                for socket_id in socket_ids {
                    if let Some(socket) = self.ws_table.get(socket_id) {
                        sockets.push(socket.clone());
                    }
                }
                
                let topic = topic.to_string();
                Box::pin(async move {
                    for socket in sockets {
                        match socket.send(WsMessage::Text(value.to_string())).await {
                            Ok(Ok(_)) => (),
                            _ => {
                                warn!("Failed to send an event for topic {}", topic);
                            }
                        };
                    }
                    Ok(true)
                })
            },
            Err(error) => {
                warn!("Failed to parse message: {}", error); 
                Box::pin(async move {
                    Ok(true)
                })
            }
        }
    }

    async fn subscribe(topic: String, watchtower_client: Arc<WatchtowerClient>, http_client: Arc<reqwest::Client>, auth: env::AuthInfo) -> Result<()> {
        let base_url = watchtower_client.get_service_url(TRAFFIC_CONTROL_SERVICE_ID).await?;
        let url = format!("http://{}/api/v1/subscription/{}", base_url, topic);

        let instance_info = &env::get_instance_info();
        http_client
            .put(&url)
            .query(&[("host", &instance_info.host), ("port", &instance_info.port.to_string())])
            .basic_auth(auth.username, Some(auth.password))
            .body(to_string(&env::get_instance_info())?).header("content-type", "application/json")
            .send().await?;
        Ok(())
    }

    async fn unsubscribe(topic: String, watchtower_client: Arc<WatchtowerClient>, http_client: Arc<reqwest::Client>, auth: env::AuthInfo) -> Result<()> {
        let base_url = watchtower_client.get_service_url(TRAFFIC_CONTROL_SERVICE_ID).await?;
        let url = format!("http://{}/api/v1/subscription/{}", base_url, topic);

        let instance_info = &env::get_instance_info();
        http_client
            .delete(&url)
            .query(&[("host", &instance_info.host), ("port", &instance_info.port.to_string())])
            .basic_auth(auth.username, Some(auth.password))
            .body(to_string(&env::get_instance_info())?).header("content-type", "application/json")
            .send().await?;
        Ok(()) 
    }
    
    async fn reset(watchtower_client: Arc<WatchtowerClient>, http_client: Arc<reqwest::Client>, auth: env::AuthInfo) -> Result<()> {
        let base_url = watchtower_client.get_service_url(TRAFFIC_CONTROL_SERVICE_ID).await?;
        let url = format!("http://{}/api/v1/subscription", base_url);

        let instance_info = &env::get_instance_info();
        http_client
            .delete(&url)
            .query(&[("host", &instance_info.host), ("port", &instance_info.port.to_string())])
            .basic_auth(auth.username, Some(auth.password))
            .header("content-type", "application/json")
            .send().await?;
        Ok(()) 
    }
}

impl Actor for Dispatcher {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.notify(DispatcherMessage::Reset);
    }
}

impl Handler<DispatcherMessage> for Dispatcher {
    type Result = ResponseFuture<Result<bool>>;

    fn handle(&mut self, event: DispatcherMessage, _ctx: &mut Context<Self>) -> Self::Result {
        match event {
            DispatcherMessage::Event(event) => {
                self.broadcast_event(&event.topic, &event.message)
            }
            DispatcherMessage::RegisterWS {
                socket_id,
                addr
            } => {
                self.ws_table.insert(socket_id, Arc::new(addr));
                Box::pin(async {
                    Ok(true)
                })
            }
            DispatcherMessage::Subscribe { socket_id, topic, request_id } => {
                if let Some(socket) = self.ws_table.get(&socket_id) {
                    let row_change = self.subscription_table.insert(&socket_id, &topic);
                    let watchtower_client = self.watchtower_client.clone();
                    let http_client = self.http_client.clone();
                    let auth = self.auth.clone();

                    let socket = socket.clone();
                    Box::pin(async move {
                        if row_change {
                            Self::subscribe(topic.to_string(), watchtower_client, http_client, auth).await?;
                        }
                        
                        socket.send(WsMessage::Subscription {
                            topic, 
                            request_id,
                            subscribed: true
                        }).await??;
                        Ok(true)
                    })
                } else {
                    Box::pin(async move {
                        Ok(true)
                    })
                }
            }
            DispatcherMessage::Unsubscribe { socket_id, topic, request_id } => {
                if let Some(socket) = self.ws_table.get(&socket_id) {
                    let row_change = self.subscription_table.remove(&socket_id, &topic);
                    let watchtower_client = self.watchtower_client.clone();
                    let http_client = self.http_client.clone();
                    let auth = self.auth.clone();

                    let socket = socket.clone();
                    Box::pin(async move {
                        if row_change {
                            Self::unsubscribe(topic.to_string(), watchtower_client, http_client, auth).await?;
                        }
                        
                        socket.send(WsMessage::Subscription {
                            topic, 
                            request_id,
                            subscribed: false
                        }).await??;
                        Ok(true)
                    })
                } else {
                    Box::pin(async move {
                        Ok(true)
                    })
                }
            }
            DispatcherMessage::Close(socket_id) => {
                let topics = self.subscription_table.remove_all(&socket_id);
                let watchtower_client = self.watchtower_client.clone();
                let http_client = self.http_client.clone();
                let auth = self.auth.clone();
                Box::pin(async move {
                    for topic in topics {
                        Self::unsubscribe(topic, watchtower_client.clone(), http_client.clone(), auth.clone()).await?;
                    }
                    Ok(true)
                })
            }
            DispatcherMessage::Reset => {
                let watchtower_client = self.watchtower_client.clone();
                let http_client = self.http_client.clone();
                let auth = self.auth.clone();
                Box::pin(async {
                    Self::reset(watchtower_client, http_client, auth).await?;
                    Ok(true)
                })
            }
        }
    }
}
