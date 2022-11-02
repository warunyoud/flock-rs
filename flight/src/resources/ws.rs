use actix::{
    Actor, StreamHandler, Handler, Message,
    prelude::AsyncContext
};
use actix_web::web;
use actix_web_actors::ws;
use serde::Deserialize;
use serde_json::json;
use std::time::Duration;
use crate::{
    types::AppState,
    resources::DispatcherMessage,
    utils::time::get_time_since_epoch
};

const PING_TTL_SECONDS: u64 = 30;
const RUN_INTERVAL_SEC: u64 = 15;

pub struct MyWs {
    socket_id: String,
    last_updated_timestamp: u64,
    app_state: web::Data<AppState>
}

pub enum WsMessage {
    Text(String),
    Subscription {
        topic: String,
        request_id: String,
        subscribed: bool
    }
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type")]
pub enum WsCommand {
    Subscribe { topic: String, request_id: String },
    Unsubscribe { topic: String, request_id: String },
    Ping
}

impl Message for WsMessage {
    type Result = Result<bool, std::io::Error>;
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(Duration::from_secs(RUN_INTERVAL_SEC), move |this, ctx| {
            if this.is_expired() {
                this.app_state.get_ref().dispatcher.do_send(DispatcherMessage::Close(this.socket_id.to_string()));
                ctx.close(Some(ws::CloseReason::from((ws::CloseCode::Normal, "ping timeout"))));
            }
        });
    }
}

impl MyWs {
    pub fn new(socket_id: String, app_state: web::Data<AppState>) -> MyWs {
        MyWs {
            socket_id,
            last_updated_timestamp: get_time_since_epoch().unwrap(),
            app_state
        }
    }

    pub fn is_expired(&self) -> bool {
        (self.last_updated_timestamp + PING_TTL_SECONDS) < get_time_since_epoch().unwrap()
    }

    pub fn create_subscription_response(topic: String, request_id: String, subscribed: bool) -> String {
        json!({
            "topic": topic,
            "subscribed": subscribed,
            "type": "response",
            "request_id": request_id
        }).to_string()
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(
        &mut self,
        msg: Result<ws::Message, ws::ProtocolError>,
        ctx: &mut Self::Context,
    ) {
        self.last_updated_timestamp = get_time_since_epoch().unwrap();
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                ctx.pong(&msg)
            }
            Ok(ws::Message::Text(text)) => {
                match serde_json::from_str(&text) {
                    Ok(WsCommand::Subscribe { topic, request_id }) => {
                        self.app_state.get_ref().dispatcher.do_send(DispatcherMessage::Subscribe {
                            socket_id: self.socket_id.to_string(), 
                            topic: topic.to_string(), 
                            request_id: request_id.to_string()
                        });
                    }
                    Ok(WsCommand::Unsubscribe { topic, request_id }) => {
                        self.app_state.get_ref().dispatcher.do_send(DispatcherMessage::Unsubscribe {
                            socket_id: self.socket_id.to_string(), 
                            topic: topic.to_string(), 
                            request_id: request_id.to_string()
                        });
                    }
                    Ok(WsCommand::Ping) => {
                        ctx.pong(&[]);
                    }
                    Err(error) => {
                        self.app_state.get_ref().dispatcher.do_send(DispatcherMessage::Close(self.socket_id.to_string()));
                        ctx.close(Some(ws::CloseReason::from((ws::CloseCode::Invalid, error.to_string()))));
                    }
                }
            }
            Ok(ws::Message::Close(_reason)) => {
                self.app_state.get_ref().dispatcher.do_send(DispatcherMessage::Close(self.socket_id.to_string()));
                ctx.close(Some(ws::CloseReason::from((ws::CloseCode::Normal, "closed by server"))));
            }
            _ => ()
        };
    }
}

impl Handler<WsMessage> for MyWs {
    type Result = Result<bool, std::io::Error>;

    fn handle(&mut self, msg: WsMessage, ctx: &mut ws::WebsocketContext<Self>) -> Self::Result {
        match msg {
            WsMessage::Text(text) => ctx.text(text),
            WsMessage::Subscription { topic, request_id, subscribed } => ctx.text(Self::create_subscription_response(topic, request_id, subscribed))
        };
        Ok(true)
    }
}