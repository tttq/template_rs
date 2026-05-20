use summer_web::{get, post, put, delete, Router};
use summer_web::extractor::{Component, Json, Query, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_web::handler::TypeRouter;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use common::pagination::PageQuery;
use system_application::role::dto::{CreateRoleDto, UpdateRoleDto};
use system_application::role::service::RoleAppService;

pub fn routes() -> Router {
    Router::new()
        .typed_route(list_roles)
        .typed_route(list_roles_tree)
        .typed_route(list_all_roles)
        .typed_route(create_role)
        .typed_route(get_role_by_id)
        .typed_route(update_role)
        .typed_route(delete_role)
        .typed_route(assign_role_menus)
        .typed_route(get_role_menu_ids)
}

#[get("/roles")]
#[sa_check_permission("role:list")]
async fn list_roles(
    Component(service): Component<RoleAppService>,
    Query(query): Query<PageQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/roles/tree")]
#[sa_check_permission("role:list")]
async fn list_roles_tree(
    Component(service): Component<RoleAppService>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list_tree().await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/roles/all")]
#[sa_check_permission("role:list")]
async fn list_all_roles(
    Component(service): Component<RoleAppService>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list_all().await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/roles/{id}")]
#[sa_check_permission("role:list")]
async fn get_role_by_id(
    Component(service): Component<RoleAppService>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_id(id).await {
        Ok(role) => Json(ApiResponse::success(role)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/roles")]
#[sa_check_permission("role:add")]
async fn create_role(
    Component(service): Component<RoleAppService>,
    Json(dto): Json<CreateRoleDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create(dto).await {
        Ok(role) => Json(ApiResponse::success(role)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/roles/{id}")]
#[sa_check_permission("role:edit")]
async fn update_role(
    Component(service): Component<RoleAppService>,
    Path(id): Path<i64>,
    Json(dto): Json<UpdateRoleDto>,
) -> Result<impl IntoResponse, WebError> {
    let mut dto = dto;
    dto.id = Some(id);
    Ok(match service.update(dto).await {
        Ok(role) => Json(ApiResponse::success(role)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[delete("/roles/{id}")]
#[sa_check_permission("role:delete")]
async fn delete_role(
    Component(service): Component<RoleAppService>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("删除成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/roles/{id}/menus")]
#[sa_check_permission("role:edit")]
async fn assign_role_menus(
    Component(service): Component<RoleAppService>,
    Path(id): Path<i64>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, WebError> {
    let menu_ids: Vec<i64> = body.get("menuIds")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_i64()).collect())
        .unwrap_or_default();
    Ok(match service.assign_menus(id, menu_ids).await {
        Ok(()) => Json(ApiResponse::success("分配成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/roles/{id}/menus")]
#[sa_check_permission("role:list")]
async fn get_role_menu_ids(
    Component(service): Component<RoleAppService>,
    Path(id): Path<i64>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_role_menu_ids(id).await {
        Ok(ids) => Json(ApiResponse::success(ids)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}
