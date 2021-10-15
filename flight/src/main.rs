use actix::Actor;
use actix_web::{middleware, web, App, HttpServer};

mod routes;
mod error;
mod types;
mod utils;
mod resources;

use crate::{
    types::AppState,
    resources::Dispatcher,
    utils::env
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "flight=info,actix_web=info,actix_redis=info,watchtower_client=info");
    env_logger::init();

    let dispatcher = Dispatcher::new().start();
    let app_state = AppState {
        dispatcher,
    };

    let instance_info = env::get_instance_info();

    HttpServer::new(move || App::new()
        .wrap(middleware::Logger::default())
        .data(app_state.clone())
        .service(
            web::scope("/api/v1")
            .configure(routes::api::v1::event::config)
        )
        .service(web::resource("/ws").route(web::get().to(routes::ws::index)))
    )
    .bind(format!("{}:{}", instance_info.host, instance_info.port))?
    .run()
    .await
}
