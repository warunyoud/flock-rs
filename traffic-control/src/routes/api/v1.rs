use actix_redis::Command;
use actix_web::{web, guard, HttpResponse};
use futures_util::future::join_all;
use redis_async::{resp::RespValue, resp_array};
use std::convert::TryFrom;
use crate::types::{Result, Error, TargetInfo, AuthorizedReq, AppState, AuthInfo};


async fn subscribe(_: AuthorizedReq, path: web::Path<(String,)>, info: web::Query<TargetInfo>, app_state: web::Data<AppState>) -> Result<HttpResponse> {
    let (topic,) = path.into_inner();
    let str_info = info.into_inner().to_string();

    let redis = &app_state.redis_addr;

    let one = redis.send(Command(resp_array!["SADD", format!("topic:{}", topic), str_info.to_string()]));
    let two = redis.send(Command(resp_array!["SADD", format!("subscription:{}", str_info), topic]));

    let res: Vec<Result<RespValue>> =
        join_all(vec![one, two].into_iter())
            .await
            .into_iter()
            .map(|item| {
                item.map_err(Error::from)
                    .and_then(|res| res.map_err(Error::from))
            })
            .collect();

    // successful operations return an integer: 1 for added and 0 for already added
    if res.iter().all(|res| match res {
        Ok(RespValue::Integer(_)) => true,
        _ => false
    }) {
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::InternalServerError().finish())
    }
}

async fn unsubscribe(_: AuthorizedReq, path: web::Path<(String,)>, info: web::Query<TargetInfo>, app_state: web::Data<AppState>) -> Result<HttpResponse> {
    let (topic,) = path.into_inner();
    let str_info = info.into_inner().to_string();

    let redis = &app_state.redis_addr;

    let one = redis.send(Command(resp_array!["SREM", format!("topic:{}", topic), str_info.to_string()]));
    let two = redis.send(Command(resp_array!["SREM", format!("subscription:{}", str_info), topic]));

    let res: Vec<Result<RespValue>> =
        join_all(vec![one, two].into_iter())
            .await
            .into_iter()
            .map(|item| {
                item.map_err(Error::from)
                    .and_then(|res| res.map_err(Error::from))
            })
            .collect();

   // successful operations return an integer: 1 for removed and 0 for already removed
    if res.iter().all(|res| match res {
        Ok(RespValue::Integer(_)) => true,
        _ => false
    }) {
        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::InternalServerError().finish())
    }
}

async fn remove_from_topic(str_info: String, topic: RespValue, redis: &actix::Addr<actix_redis::RedisActor>) -> Result<()> {
    match topic {
        RespValue::SimpleString(topic) => {
            redis.send(Command(resp_array!["SREM", format!("topic:{}", topic), str_info])).await??;
            Ok(())
        }
        RespValue::BulkString(vec) => {
            let topic = std::str::from_utf8(&vec).map_err(|_| Error::InternalError)?;
            redis.send(Command(resp_array!["SREM", format!("topic:{}", topic), str_info])).await??;
            Ok(())
        }
        _ => Err(Error::InternalError)
    }
}

async fn reset(_: AuthorizedReq, info: web::Query<TargetInfo>, app_state: web::Data<AppState>) -> Result<HttpResponse> {
    let str_info = info.into_inner().to_string();

    let redis = &app_state.redis_addr;

    match redis.send(Command(resp_array!["SMEMBERS", format!("subscription:{}", str_info)])).await? {
        Ok(RespValue::Array(topics)) => {
            redis.send(Command(resp_array!["DEL", format!("subscription:{}", str_info)])).await??; 

            let sends =  topics.into_iter().map(|topic| {
                remove_from_topic(str_info.to_string(), topic, redis)
            });
            join_all(sends).await;
            Ok(HttpResponse::Ok().finish())
        }
        _ => {
            Ok(HttpResponse::Ok().finish())    
        }
    }
}

async fn send_event(req_body: String, target: RespValue, topic: String, http_client: &reqwest::Client, auth: &AuthInfo) -> Result<()> {
    if let Ok(target) = TargetInfo::try_from(target) {
        println!("{:?}", target);
        let url = format!("http://{}/api/v1/event/{}", target, topic);
        let auth = auth.clone();
        let res = http_client
            .post(&url)
            .basic_auth(auth.username, Some(auth.password))
            .body(req_body).header("content-type", "application/json")
            .send().await?;

        if res.status() != reqwest::StatusCode::OK {
            log::error!("Unable to send request to {}", target);
        }
    }
    Ok(())
}

async fn publish_event(_: AuthorizedReq, path: web::Path<(String,)>, req_body: String, app_state: web::Data<AppState>) -> Result<HttpResponse> {
    let (topic,) = path.into_inner();
    
    let redis = &app_state.redis_addr;

    match redis.send(Command(resp_array!["SMEMBERS", format!("topic:{}", topic)])).await? {
        Ok(RespValue::Array(targets)) => {
            let sends =  targets.into_iter().map(|target| {
                send_event(req_body.to_string(), target, topic.to_string(), &app_state.http_client, &app_state.auth)
            });
            join_all(sends).await;
            Ok(HttpResponse::Ok().finish())
        }
        Ok(RespValue::Nil) => Ok(HttpResponse::Ok().finish()),
        _ => Ok(HttpResponse::InternalServerError().finish())
    }    
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service( 
        web::resource("/subscription/{topic:.*}")
            .route(web::put().to(subscribe))
            .route(web::delete().to(unsubscribe)) 
    ).service(
        web::resource("/subscription")
            .route(web::delete().to(reset))
    ).service(
        web::resource("/event/{topic:.*}")
            .guard(guard::Header("content-type", "application/json"))
            .route(web::post().to(publish_event))
    );
}
