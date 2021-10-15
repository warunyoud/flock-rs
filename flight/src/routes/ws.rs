use actix_web::{web, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use uuid::Uuid;

use crate::{
    types::{AppState, Result},
    resources::{DispatcherMessage, MyWs}
};

pub async fn index(req: HttpRequest, stream: web::Payload, app_state: web::Data<AppState>) -> Result<HttpResponse> {
    let socket_id = Uuid::new_v4();
    let (addr, resp) = ws::start_with_addr(MyWs::new(socket_id.to_string(), app_state.clone()), &req, stream)?;
    app_state.get_ref().dispatcher.send(DispatcherMessage::RegisterWS {
        socket_id: socket_id.to_string(),
        addr
    }).await??;
    Ok(resp)
}
