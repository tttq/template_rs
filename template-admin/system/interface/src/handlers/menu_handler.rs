use summer_web::{get, post, put, delete, Router};
use summer_web::extractor::{Component, Json, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_web::handler::TypeRouter;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use system_application::menu::dto::{CreateMenuDto, UpdateMenuDto};
use system_application::menu::service::MenuAppService;

pub fn routes() -> Router {
    Router::new()
        .typed_route(list_menus)
        .typed_route(list_menu_tree)
        .typed_route(create_menu)
        .typed_route(get_menu_by_id)
        .typed_route(update_menu)
        .typed_route(delete_menu)
}

#[get("/menus")]
#[sa_check_permission("menu:list")]
async fn list_menus(
    Component(service): Component<MenuAppService>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list().await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/menus/tree")]
#[sa_check_permission("menu:list")]
async fn list_menu_tree(
    Component(service): Component<MenuAppService>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list_tree().await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/menus/{id}")]
#[sa_check_permission("menu:list")]
async fn get_menu_by_id(
    Component(service): Component<MenuAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_id(id).await {
        Ok(menu) => Json(ApiResponse::success(menu)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/menus")]
#[sa_check_permission("menu:add")]
async fn create_menu(
    Component(service): Component<MenuAppService>,
    Json(dto): Json<CreateMenuDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create(dto).await {
        Ok(menu) => Json(ApiResponse::success(menu)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/menus/{id}")]
#[sa_check_permission("menu:edit")]
async fn update_menu(
    Component(service): Component<MenuAppService>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateMenuDto>,
) -> Result<impl IntoResponse, WebError> {
    let mut dto = dto;
    dto.id = Some(id);
    Ok(match service.update(dto).await {
        Ok(menu) => Json(ApiResponse::success(menu)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[delete("/menus/{id}")]
#[sa_check_permission("menu:delete")]
async fn delete_menu(
    Component(service): Component<MenuAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("删除成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}
