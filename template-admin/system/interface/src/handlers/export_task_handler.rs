//! 通用导出中心（系统管理内）：创建 / 列表 / 详情 / 下载
//!
//! - `POST /system/exports`                   提交导出任务（body `{ taskType, query }`，立即返回任务状态）
//! - `GET  /system/exports`                   任务列表（本人，分页）
//! - `GET  /system/exports/{id}`              任务详情（轮询用）
//! - `GET  /system/exports/{id}/download`     下载生成的 xlsx

use common::pagination::PageQuery;
use common::pagination::PageResult;
use common::response::ApiResponse;
use summer_sa_token::sa_check_permission;
use summer_web::axum::body::Body;
use summer_web::axum::http::StatusCode;
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_web::extractor::{Component, Json, Path, Query};
use summer_web::{get, nest, post};
use system_application::export_task::dto::{CreateExportTaskDto, ExportTaskVo};
use system_application::export_task::service::ExportTaskAppService;

#[nest("/system")]
mod controller {
use super::*;

#[post("/exports")]
#[sa_check_permission("system:export:create")]
async fn create_export_task(
    Component(service): Component<ExportTaskAppService>,
    Json(dto): Json<CreateExportTaskDto>,
) -> Result<impl IntoResponse, WebError> {
    match service.create(dto).await {
        Ok(v) => Ok(Json(ApiResponse::success(v))),
        Err(e) => Ok(Json(ApiResponse::<ExportTaskVo>::error(500, &e.to_string()))),
    }
}

#[get("/exports")]
#[sa_check_permission("system:export:list")]
async fn list_export_tasks(
    Component(service): Component<ExportTaskAppService>,
    Query(query): Query<PageQuery>,
) -> Result<impl IntoResponse, WebError> {
    match service.list(query).await {
        Ok(v) => Ok(Json(ApiResponse::success(v))),
        Err(e) => Ok(Json(ApiResponse::<PageResult<ExportTaskVo>>::error(500, &e.to_string()))),
    }
}

#[get("/exports/{id}")]
#[sa_check_permission("system:export:list")]
async fn get_export_task(
    Component(service): Component<ExportTaskAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    match service.get(&id).await {
        Ok(v) => Ok(Json(ApiResponse::success(v))),
        Err(e) => Ok(Json(ApiResponse::<ExportTaskVo>::error(500, &e.to_string()))),
    }
}

#[get("/exports/{id}/download")]
#[sa_check_permission("system:export:download")]
async fn download_export(
    Component(service): Component<ExportTaskAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    match service.download(&id).await {
        Ok((bytes, file_name, mime)) => {
            let content_disposition = format!("attachment; filename=\"{}\"", file_name);
            Ok((
                StatusCode::OK,
                [
                    ("content-type", mime.as_str()),
                    ("content-disposition", content_disposition.as_str()),
                ],
                Body::from(bytes),
            )
                .into_response())
        }
        Err(e) => Ok((
            StatusCode::INTERNAL_SERVER_ERROR,
            [("content-type", "application/json")],
            Body::from(serde_json::to_string(&ApiResponse::<()>::error(500, &e.to_string())).unwrap_or_default()),
        )
            .into_response()),
    }
}
}
