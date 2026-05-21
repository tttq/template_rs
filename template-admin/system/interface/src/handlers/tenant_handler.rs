use summer_web::{get, post, Router};
use summer_web::extractor::{Component, Json, Query, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_web::handler::TypeRouter;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use common::pagination::PageQuery;
use system_application::tenant::dto::{CreateTenantDto, UpdateTenantDto, TestConnectionDto, CreateDatabaseDto, InitDatabaseDto, CreateTenantFullDto};
use system_application::tenant::service::TenantAppService;

pub fn routes() -> Router {
    Router::new()
        .typed_route(list_tenants)
        .typed_route(create_tenant)
        .typed_route(create_tenant_full)
        .typed_route(get_tenant_by_id)
        .typed_route(update_tenant)
        .typed_route(delete_tenant)
        .typed_route(test_connection)
        .typed_route(create_database)
        .typed_route(init_database)
}

#[get("/tenants")]
#[sa_check_permission("tenant:list")]
async fn list_tenants(
    Component(service): Component<TenantAppService>,
    Query(query): Query<PageQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/tenants/{id}")]
#[sa_check_permission("tenant:list")]
async fn get_tenant_by_id(
    Component(service): Component<TenantAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_id(id).await {
        Ok(tenant) => Json(ApiResponse::success(tenant)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/tenants")]
#[sa_check_permission("tenant:add")]
async fn create_tenant(
    Component(service): Component<TenantAppService>,
    Json(dto): Json<CreateTenantDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create(dto).await {
        Ok(tenant) => Json(ApiResponse::success(tenant)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/tenants/{id}")]
#[sa_check_permission("tenant:edit")]
async fn update_tenant(
    Component(service): Component<TenantAppService>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateTenantDto>,
) -> Result<impl IntoResponse, WebError> {
    let mut dto = dto;
    dto.id = Some(id);
    Ok(match service.update(dto).await {
        Ok(tenant) => Json(ApiResponse::success(tenant)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/tenants/{id}/delete")]
#[sa_check_permission("tenant:delete")]
async fn delete_tenant(
    Component(service): Component<TenantAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("删除成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/tenants/test-connection")]
#[sa_check_permission("tenant:add")]
async fn test_connection(
    Component(service): Component<TenantAppService>,
    Json(dto): Json<TestConnectionDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.test_connection(dto).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/tenants/create-database")]
#[sa_check_permission("tenant:add")]
async fn create_database(
    Component(service): Component<TenantAppService>,
    Json(dto): Json<CreateDatabaseDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create_database(dto).await {
        Ok(msg) => Json(ApiResponse::success(msg)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/tenants/init-database")]
#[sa_check_permission("tenant:add")]
async fn init_database(
    Component(service): Component<TenantAppService>,
    Json(dto): Json<InitDatabaseDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.init_database(dto).await {
        Ok(msg) => Json(ApiResponse::success(msg)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/tenants/create-full")]
#[sa_check_permission("tenant:add")]
async fn create_tenant_full(
    Component(service): Component<TenantAppService>,
    Json(dto): Json<CreateTenantFullDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create_full(dto).await {
        Ok(tenant) => Json(ApiResponse::success(tenant)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}
