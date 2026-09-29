//! 系统管理列表导出的统一响应（功能页内直出 Excel；超阈值改走导出中心异步任务）
//!
//! 分流（阈值 [`MAX_SYNC_ROWS`] = 10 万条）：
//! - ≤ 阈值：`GET /xxx/export` 同步导出直接下载，**同时在导出中心登记一条 success 记录**
//!   （`ExportTaskAppService::record_sync_export`，登记失败只记日志、不阻断下载）；
//! - > 阈值：前端提交 `POST /api/system/exports`（`{ taskType, query }`）异步生成，
//!   生成文件统一落在附件中心，用户在「导出中心」查看并下载。
//!
//! 于是同步/异步两种导出结果都能在「导出中心」正确记录与展示。

use common::error::AppError;
use common::response::ApiResponse;
use common::xlsx::{XLSX_MIME, too_large_message, xlsx_bytes, xlsx_file_name};
use summer_web::axum::body::Body;
use summer_web::axum::http::StatusCode;
use summer_web::axum::response::{IntoResponse, Json, Response};
use system_application::export_task::service::ExportTaskAppService;

/// 导出的同步上限（超过即改走「导出中心」异步任务；前端同名常量需保持一致）
pub use common::xlsx::MAX_SYNC_ROWS;

/// 同步导出（≤ 上限）：生成 xlsx → 登记导出中心 → 返回下载响应。
///
/// 登记失败只记日志，保证用户仍能拿到文件。
pub async fn sync_export_response(
    sheet_name: &str,
    exports: &ExportTaskAppService,
    task_type: &str,
    query: &serde_json::Value,
    prefix: &str,
    headers: &[&str],
    rows: Vec<Vec<String>>,
    total: u64,
) -> Response {
    let bytes = match xlsx_bytes(sheet_name, headers, rows) {
        Ok(v) => v,
        Err(e) => return error_response(e),
    };
    let file_name = xlsx_file_name(prefix);
    if let Err(e) = exports
        .record_sync_export(
            task_type,
            query,
            &file_name,
            XLSX_MIME,
            total as i32,
            &bytes,
        )
        .await
    {
        log::warn!("同步导出登记导出中心失败（task_type={task_type}）: {e}");
    }
    xlsx_response(&file_name, bytes)
}

/// Excel 下载响应（仅本模块内部使用）
fn xlsx_response(file_name: &str, bytes: Vec<u8>) -> Response {
    (
        StatusCode::OK,
        [
            ("content-type", XLSX_MIME),
            (
                "content-disposition",
                &format!("attachment; filename=\"{}\"", file_name),
            ),
        ],
        Body::from(bytes),
    )
        .into_response()
}

/// 超过同步上限：提示前端改走「导出中心」异步任务
pub fn export_center_hint(total: u64) -> Response {
    Json(ApiResponse::<()>::error(500, &too_large_message(total))).into_response()
}

/// 统一错误响应（业务错误 HTTP 200 + code 500，与项目其它 handler 一致）
pub fn error_response(e: AppError) -> Response {
    Json(ApiResponse::<()>::error(500, &e.to_string())).into_response()
}
