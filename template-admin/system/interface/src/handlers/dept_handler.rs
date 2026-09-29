use summer_web::{get, nest, post, put, delete};
use summer_web::extractor::{Component, Json, Query, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use system_application::dept::dto::{CreateDeptDto, UpdateDeptDto, DeptQuery, DeptExportQuery};
use system_application::dept::service::DeptAppService;
use system_application::dept::{DEPT_HEADERS, dept_rows};
use system_application::export_task::service::ExportTaskAppService;
use super::list_export::{MAX_SYNC_ROWS, error_response, export_center_hint, sync_export_response};

#[nest("/system")]
mod controller {
    use super::*;

#[get("/depts")]
#[sa_check_permission("dept:list")]
async fn list_depts(
    Component(service): Component<DeptAppService>,
    Query(query): Query<DeptQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list(query).await {
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
        Ok(()) => Json(ApiResponse::success("@deleted_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 可导出行数（同步/异步分流判定）
#[get("/depts/export/count")]
#[sa_check_permission("dept:export")]
async fn export_dept_count(
    Component(service): Component<DeptAppService>,
    Query(query): Query<DeptExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.export_count(&query).await {
        Ok(total) => Json(ApiResponse::success(serde_json::json!({ "total": total }))),
        Err(e) => Json(ApiResponse::<serde_json::Value>::error(500, &e.to_string())),
    })
}

/// Excel 导出部门列表（≤ [`MAX_SYNC_ROWS`] 同步下载并在导出中心留记录；超过走异步任务）
#[get("/depts/export")]
#[sa_check_permission("dept:export")]
async fn export_depts(
    Component(service): Component<DeptAppService>,
    Component(exports): Component<ExportTaskAppService>,
    Query(query): Query<DeptExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    match service.export_count(&query).await {
        Ok(total) if total > MAX_SYNC_ROWS => Ok(export_center_hint(total)),
        Ok(total) => {
            let query_json = serde_json::to_value(&query).unwrap_or_default();
            match service.export(query).await {
                Ok(vos) => Ok(sync_export_response(
                    "部门数据",
                    &exports,
                    "dept",
                    &query_json,
                    "depts-export",
                    DEPT_HEADERS,
                    dept_rows(&vos),
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
