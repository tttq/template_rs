use summer_web::{get, post, put, delete, Router};
use summer_web::extractor::{Component, Json, Query, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_web::handler::TypeRouter;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use common::pagination::PageQuery;
use system_application::dict_type::dto::{CreateDictTypeDto, UpdateDictTypeDto};
use system_application::dict_type::service::DictTypeAppService;
use system_application::dict_item::dto::{CreateDictItemDto, UpdateDictItemDto};
use system_application::dict_item::service::DictItemAppService;

pub fn routes() -> Router {
    Router::new()
        .typed_route(list_dict_types)
        .typed_route(create_dict_type)
        .typed_route(update_dict_type)
        .typed_route(delete_dict_type)
        .typed_route(list_dict_items)
        .typed_route(list_items_by_type_id)
        .typed_route(list_items_by_type_code)
        .typed_route(create_dict_item)
        .typed_route(update_dict_item)
        .typed_route(delete_dict_item)
}

#[get("/dicts/types")]
#[sa_check_permission("dict:list")]
async fn list_dict_types(
    Component(service): Component<DictTypeAppService>,
    Query(query): Query<PageQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/dicts/types")]
#[sa_check_permission("dict:add")]
async fn create_dict_type(
    Component(service): Component<DictTypeAppService>,
    Json(dto): Json<CreateDictTypeDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create(dto).await {
        Ok(dict) => Json(ApiResponse::success(dict)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/dicts/types/{id}")]
#[sa_check_permission("dict:edit")]
async fn update_dict_type(
    Component(service): Component<DictTypeAppService>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateDictTypeDto>,
) -> Result<impl IntoResponse, WebError> {
    let mut dto = dto;
    dto.id = Some(id);
    Ok(match service.update(dto).await {
        Ok(dict) => Json(ApiResponse::success(dict)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[delete("/dicts/types/{id}")]
#[sa_check_permission("dict:delete")]
async fn delete_dict_type(
    Component(service): Component<DictTypeAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("删除成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/dicts/items")]
#[sa_check_permission("dict:list")]
async fn list_dict_items(
    Component(service): Component<DictItemAppService>,
    Query(query): Query<PageQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/dicts/items/by-type/{typeId}")]
#[sa_check_permission("dict:list")]
async fn list_items_by_type_id(
    Component(service): Component<DictItemAppService>,
    Path(type_id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_dict_type_id(type_id).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/dicts/types/{typeCode}/items")]
#[sa_check_permission("dict:list")]
async fn list_items_by_type_code(
    Component(service): Component<DictItemAppService>,
    Path(type_code): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_dict_type_code(type_code).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/dicts/items")]
#[sa_check_permission("dict:add")]
async fn create_dict_item(
    Component(service): Component<DictItemAppService>,
    Json(dto): Json<CreateDictItemDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create(dto).await {
        Ok(item) => Json(ApiResponse::success(item)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/dicts/items/{id}")]
#[sa_check_permission("dict:edit")]
async fn update_dict_item(
    Component(service): Component<DictItemAppService>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateDictItemDto>,
) -> Result<impl IntoResponse, WebError> {
    let mut dto = dto;
    dto.id = Some(id);
    Ok(match service.update(dto).await {
        Ok(item) => Json(ApiResponse::success(item)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[delete("/dicts/items/{id}")]
#[sa_check_permission("dict:delete")]
async fn delete_dict_item(
    Component(service): Component<DictItemAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("删除成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}
