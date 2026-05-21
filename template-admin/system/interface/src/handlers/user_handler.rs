use summer_web::{get, post, put, delete, Router};
use summer_web::extractor::{Component, Json, Query, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_web::handler::TypeRouter;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use common::pagination::PageQuery;
use system_application::user::dto::{CreateUserDto, UpdateUserDto};
use system_application::user::service::UserAppService;

pub fn routes() -> Router {
    Router::new()
        .typed_route(list_users)
        .typed_route(create_user)
        .typed_route(get_user_by_id)
        .typed_route(update_user)
        .typed_route(update_user_status)
        .typed_route(delete_user)
        .typed_route(assign_user_roles)
        .typed_route(get_user_role_ids)
}

#[get("/users")]
#[sa_check_permission("user:list")]
async fn list_users(
    Component(service): Component<UserAppService>,
    Query(query): Query<PageQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/users/{id}")]
#[sa_check_permission("user:list")]
async fn get_user_by_id(
    Component(service): Component<UserAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_id(id).await {
        Ok(user) => Json(ApiResponse::success(user)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/users")]
#[sa_check_permission("user:add")]
async fn create_user(
    Component(service): Component<UserAppService>,
    Json(dto): Json<CreateUserDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create(dto).await {
        Ok(user) => Json(ApiResponse::success(user)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/users/{id}")]
#[sa_check_permission("user:edit")]
async fn update_user(
    Component(service): Component<UserAppService>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<impl IntoResponse, WebError> {
    let mut dto = dto;
    dto.id = Some(id);
    Ok(match service.update(dto).await {
        Ok(user) => Json(ApiResponse::success(user)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/users/{id}/status")]
#[sa_check_permission("user:edit")]
async fn update_user_status(
    Component(service): Component<UserAppService>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, WebError> {
    let status = body.get("status").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
    Ok(match service.update_status(id, status).await {
        Ok(()) => Json(ApiResponse::success("状态更新成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[delete("/users/{id}")]
#[sa_check_permission("user:delete")]
async fn delete_user(
    Component(service): Component<UserAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("删除成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/users/{id}/roles")]
#[sa_check_permission("user:edit")]
async fn assign_user_roles(
    Component(service): Component<UserAppService>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, WebError> {
    let role_ids: Vec<String> = body.get("roleIds")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    Ok(match service.assign_roles(id, role_ids).await {
        Ok(()) => Json(ApiResponse::success("分配成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/users/{id}/roles")]
#[sa_check_permission("user:list")]
async fn get_user_role_ids(
    Component(service): Component<UserAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_role_ids(id).await {
        Ok(ids) => Json(ApiResponse::success(ids)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}
