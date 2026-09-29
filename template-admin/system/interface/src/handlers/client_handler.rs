use summer_web::{get, nest, post, put, delete};
use summer_web::extractor::{Component, Json, Query, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_sa_token::{sa_check_login, sa_check_permission};
use common::response::ApiResponse;
use system_application::client::dto::{CreateClientDto, ClientQuery, ClientExportQuery, UpdateClientDto};
use system_application::client::service::ClientAppService;
use system_application::client::{CLIENT_HEADERS, client_rows};
use system_application::export_task::service::ExportTaskAppService;
use super::list_export::{MAX_SYNC_ROWS, error_response, export_center_hint, sync_export_response};

#[nest("/system")]
mod controller {
    use super::*;

#[get("/clients")]
#[sa_check_permission("client:list")]
async fn list_clients(
    Component(service): Component<ClientAppService>,
    Query(query): Query<ClientQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 启用客户端下拉项：菜单管理/角色管理按客户端筛选时使用，登录即可访问（不含密钥）
#[get("/clients/options")]
#[sa_check_login]
async fn list_client_options(
    Component(service): Component<ClientAppService>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list_options().await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/clients/{id}")]
#[sa_check_permission("client:list")]
async fn get_client_by_id(
    Component(service): Component<ClientAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_id(id).await {
        Ok(client) => Json(ApiResponse::success(client)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/clients")]
#[sa_check_permission("client:add")]
async fn create_client(
    Component(service): Component<ClientAppService>,
    Json(dto): Json<CreateClientDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create(dto).await {
        Ok(client) => Json(ApiResponse::success(client)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/clients/{id}")]
#[sa_check_permission("client:edit")]
async fn update_client(
    Component(service): Component<ClientAppService>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateClientDto>,
) -> Result<impl IntoResponse, WebError> {
    let mut dto = dto;
    dto.id = Some(id);
    Ok(match service.update(dto).await {
        Ok(client) => Json(ApiResponse::success(client)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[delete("/clients/{id}")]
#[sa_check_permission("client:delete")]
async fn delete_client(
    Component(service): Component<ClientAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("@deleted_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 可导出行数（同步/异步分流判定）
#[get("/clients/export/count")]
#[sa_check_permission("client:export")]
async fn export_client_count(
    Component(service): Component<ClientAppService>,
    Query(query): Query<ClientExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.export_count(&query).await {
        Ok(total) => Json(ApiResponse::success(serde_json::json!({ "total": total }))),
        Err(e) => Json(ApiResponse::<serde_json::Value>::error(500, &e.to_string())),
    })
}

/// Excel 导出客户端列表（≤ [`MAX_SYNC_ROWS`] 同步下载并在导出中心留记录；超过走异步任务）
#[get("/clients/export")]
#[sa_check_permission("client:export")]
async fn export_clients(
    Component(service): Component<ClientAppService>,
    Component(exports): Component<ExportTaskAppService>,
    Query(query): Query<ClientExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    match service.export_count(&query).await {
        Ok(total) if total > MAX_SYNC_ROWS => Ok(export_center_hint(total)),
        Ok(total) => {
            let query_json = serde_json::to_value(&query).unwrap_or_default();
            match service.export(query).await {
                Ok(vos) => Ok(sync_export_response(
                    "客户端数据",
                    &exports,
                    "client",
                    &query_json,
                    "clients-export",
                    CLIENT_HEADERS,
                    client_rows(&vos),
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
