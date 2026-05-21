use summer_web::{get, post, put, delete, Router};
use summer_web::extractor::{Component, Json, Query, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_web::handler::TypeRouter;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use common::pagination::PageQuery;
use system_application::config::dto::{CreateConfigDto, UpdateConfigDto};
use system_application::config::service::ConfigAppService;

pub fn routes() -> Router {
    Router::new()
        .typed_route(list_configs)
        .typed_route(create_config)
        .typed_route(get_config_by_id)
        .typed_route(update_config)
        .typed_route(delete_config)
        .typed_route(get_config_by_key)
}

#[get("/configs")]
#[sa_check_permission("config:list")]
async fn list_configs(
    Component(service): Component<ConfigAppService>,
    Query(query): Query<PageQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/configs/{id}")]
#[sa_check_permission("config:list")]
async fn get_config_by_id(
    Component(service): Component<ConfigAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_id(id).await {
        Ok(config) => Json(ApiResponse::success(config)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/configs/key/{key}")]
#[sa_check_permission("config:list")]
async fn get_config_by_key(
    Component(service): Component<ConfigAppService>,
    Path(key): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_key(&key).await {
        Ok(config) => Json(ApiResponse::success(config)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/configs")]
#[sa_check_permission("config:add")]
async fn create_config(
    Component(service): Component<ConfigAppService>,
    Json(dto): Json<CreateConfigDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create(dto).await {
        Ok(config) => Json(ApiResponse::success(config)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/configs/{id}")]
#[sa_check_permission("config:edit")]
async fn update_config(
    Component(service): Component<ConfigAppService>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateConfigDto>,
) -> Result<impl IntoResponse, WebError> {
    let mut dto = dto;
    dto.id = Some(id);
    Ok(match service.update(dto).await {
        Ok(config) => Json(ApiResponse::success(config)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[delete("/configs/{id}")]
#[sa_check_permission("config:delete")]
async fn delete_config(
    Component(service): Component<ConfigAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("删除成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}
