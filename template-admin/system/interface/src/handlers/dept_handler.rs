use summer_web::{get, post, put, delete, Router};
use summer_web::extractor::{Component, Json, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_web::handler::TypeRouter;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use system_application::dept::dto::{CreateDeptDto, UpdateDeptDto};
use system_application::dept::service::DeptAppService;

pub fn routes() -> Router {
    Router::new()
        .typed_route(list_depts)
        .typed_route(list_dept_tree)
        .typed_route(create_dept)
        .typed_route(get_dept_by_id)
        .typed_route(update_dept)
        .typed_route(delete_dept)
}

#[get("/depts")]
#[sa_check_permission("dept:list")]
async fn list_depts(
    Component(service): Component<DeptAppService>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list().await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/depts/tree")]
#[sa_check_permission("dept:list")]
async fn list_dept_tree(
    Component(service): Component<DeptAppService>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list_tree().await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/depts/{id}")]
#[sa_check_permission("dept:list")]
async fn get_dept_by_id(
    Component(service): Component<DeptAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_id(id).await {
        Ok(dept) => Json(ApiResponse::success(dept)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/depts")]
#[sa_check_permission("dept:add")]
async fn create_dept(
    Component(service): Component<DeptAppService>,
    Json(dto): Json<CreateDeptDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create(dto).await {
        Ok(dept) => Json(ApiResponse::success(dept)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/depts/{id}")]
#[sa_check_permission("dept:edit")]
async fn update_dept(
    Component(service): Component<DeptAppService>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateDeptDto>,
) -> Result<impl IntoResponse, WebError> {
    let mut dto = dto;
    dto.id = Some(id);
    Ok(match service.update(dto).await {
        Ok(dept) => Json(ApiResponse::success(dept)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[delete("/depts/{id}")]
#[sa_check_permission("dept:delete")]
async fn delete_dept(
    Component(service): Component<DeptAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("删除成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}
