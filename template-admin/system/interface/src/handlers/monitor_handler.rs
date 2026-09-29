use summer_web::get;
use summer_web::nest;
use summer_web::extractor::{Component, Json};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_sa_token::sa_check_login;
use common::response::ApiResponse;
use system_application::monitor::service::MonitorAppService;

#[nest("/system")]
mod controller {
    use super::*;

#[get("/redis/info")]
#[sa_check_login]
async fn get_redis_info(
    Component(service): Component<MonitorAppService>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.redis_info().await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/server/info")]
#[sa_check_login]
async fn get_server_info(
    Component(service): Component<MonitorAppService>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.server_info().await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}
}
