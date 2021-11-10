use actix_redis::RedisActor;
use actix_web::{middleware, web, App, HttpServer};
use watchtower_client::WatchtowerClient;

mod routes;
mod error;
mod types;
mod resources;
mod utils;

use crate::{
    types::AppState,
    utils::env
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "traffic_control=info,actix_web=info,actix_redis=info");
    env_logger::init();

    let redis_info = env::get_redis_info();
    let redis_addr = RedisActor::start(format!("{}:{}", redis_info.host, redis_info.port));

    let instance_info = env::get_instance_info();

    let watchtower_config = env::get_watchtower_config();
    let watchtower_client = WatchtowerClient::new(watchtower_config.urls, &watchtower_config.username, &watchtower_config.password);
    watchtower_client.register("traffic-control", &instance_info.host, instance_info.port).await.unwrap();


    HttpServer::new(move || App::new()
        .wrap(middleware::Logger::default())
        .data(AppState {
            redis_addr: redis_addr.clone(),
            http_client: reqwest::Client::new(),
            auth: env::get_auth_info()
        })
        .service(
            web::scope("/api/v1")
            .configure(routes::api::v1::config)
        )
    )
    .bind(format!("0.0.0.0:{}", instance_info.port))?
    .run()
    .await
}