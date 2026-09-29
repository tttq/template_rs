use summer_web::{get, nest, post};
use summer_web::extractor::{Component, Json, Query, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use system_application::tenant::dto::{CreateTenantDto, UpdateTenantDto, TestConnectionDto, CreateDatabaseDto, InitDatabaseDto, CreateTenantFullDto, TenantQuery, TenantExportQuery};
use system_application::tenant::service::TenantAppService;
use system_application::tenant::{TENANT_HEADERS, tenant_rows};
use system_application::export_task::service::ExportTaskAppService;
use super::list_export::{MAX_SYNC_ROWS, error_response, export_center_hint, sync_export_response};

#[nest("/system")]
mod controller {
    use super::*;

#[get("/tenants")]
#[sa_check_permission("tenant:list")]
async fn list_tenants(
    Component(service): Component<TenantAppService>,
    Query(query): Query<TenantQuery>,
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
        Ok(()) => Json(ApiResponse::success("@deleted_ok")),
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

/// 可导出行数（同步/异步分流判定）
#[get("/tenants/export/count")]
#[sa_check_permission("tenant:export")]
async fn export_tenant_count(
    Component(service): Component<TenantAppService>,
    Query(query): Query<TenantExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.export_count(&query).await {
        Ok(total) => Json(ApiResponse::success(serde_json::json!({ "total": total }))),
        Err(e) => Json(ApiResponse::<serde_json::Value>::error(500, &e.to_string())),
    })
}

/// Excel 导出租户列表（≤ [`MAX_SYNC_ROWS`] 同步下载并在导出中心留记录；超过走异步任务）
#[get("/tenants/export")]
#[sa_check_permission("tenant:export")]
async fn export_tenants(
    Component(service): Component<TenantAppService>,
    Component(exports): Component<ExportTaskAppService>,
    Query(query): Query<TenantExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    match service.export_count(&query).await {
        Ok(total) if total > MAX_SYNC_ROWS => Ok(export_center_hint(total)),
        Ok(total) => {
            let query_json = serde_json::to_value(&query).unwrap_or_default();
            match service.export(query).await {
                Ok(vos) => Ok(sync_export_response(
                    "租户数据",
                    &exports,
                    "tenant",
                    &query_json,
                    "tenants-export",
                    TENANT_HEADERS,
                    tenant_rows(&vos),
                    total,
                )
                .await),
                Err(e) => Ok(error_response(e)),
            }
        }
        Err(e) => Ok(error_response(e)),
    }
}
}
