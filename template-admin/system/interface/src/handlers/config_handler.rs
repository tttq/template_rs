use summer_web::{get, nest, post, put, delete};
use summer_web::extractor::{Component, Json, Query, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use system_application::config::dto::{CreateConfigDto, UpdateConfigDto, ConfigQuery, ConfigExportQuery};
use system_application::config::service::ConfigAppService;
use system_application::config::{CONFIG_HEADERS, config_rows};
use system_application::export_task::service::ExportTaskAppService;
use super::list_export::{MAX_SYNC_ROWS, error_response, export_center_hint, sync_export_response};

#[nest("/system")]
mod controller {
    use super::*;

#[get("/configs")]
#[sa_check_permission("config:list")]
async fn list_configs(
    Component(service): Component<ConfigAppService>,
    Query(query): Query<ConfigQuery>,
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
        Ok(()) => Json(ApiResponse::success("@deleted_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 可导出行数（同步/异步分流判定）
#[get("/configs/export/count")]
#[sa_check_permission("config:export")]
async fn export_config_count(
    Component(service): Component<ConfigAppService>,
    Query(query): Query<ConfigExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.export_count(&query).await {
        Ok(total) => Json(ApiResponse::success(serde_json::json!({ "total": total }))),
        Err(e) => Json(ApiResponse::<serde_json::Value>::error(500, &e.to_string())),
    })
}

/// Excel 导出配置列表（≤ [`MAX_SYNC_ROWS`] 同步下载并在导出中心留记录；超过走异步任务）
#[get("/configs/export")]
#[sa_check_permission("config:export")]
async fn export_configs(
    Component(service): Component<ConfigAppService>,
    Component(exports): Component<ExportTaskAppService>,
    Query(query): Query<ConfigExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    match service.export_count(&query).await {
        Ok(total) if total > MAX_SYNC_ROWS => Ok(export_center_hint(total)),
        Ok(total) => {
            let query_json = serde_json::to_value(&query).unwrap_or_default();
            match service.export(query).await {
                Ok(vos) => Ok(sync_export_response(
                    "参数配置",
                    &exports,
                    "config",
                    &query_json,
                    "configs-export",
                    CONFIG_HEADERS,
                    config_rows(&vos),
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
