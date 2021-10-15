use actix_web::{web, guard, HttpResponse};

use crate::{
    resources::DispatcherMessage,
    types::{Result, AppState, Event, AuthorizedReq},
};

async fn publish_event(_: AuthorizedReq, path: web::Path<(String,)>, req_body: String, app_state: web::Data<AppState>) -> Result<HttpResponse> {
    let (topic,) = path.into_inner();
    app_state.get_ref().dispatcher.send(DispatcherMessage::Event(Event {
        topic,
        message: req_body
    })).await??;
    Ok(HttpResponse::Ok().finish())
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("/event/{topic:.*}")
            .guard(guard::Header("content-type", "application/json"))
            .route(web::post().to(publish_event))
    );
}
