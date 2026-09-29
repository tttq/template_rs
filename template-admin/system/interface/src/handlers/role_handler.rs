use summer_web::{get, nest, post, put, delete};
use summer_web::extractor::{Component, Json, Query, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use system_application::role::dto::{CreateRoleDto, UpdateRoleDto, RoleQuery, RoleExportQuery, RoleTreeQuery};
use system_application::role::service::RoleAppService;
use system_application::role::{ROLE_HEADERS, role_rows};
use system_application::export_task::service::ExportTaskAppService;
use super::list_export::{MAX_SYNC_ROWS, error_response, export_center_hint, sync_export_response};

#[nest("/system")]
mod controller {
    use super::*;

#[get("/roles")]
#[sa_check_permission("role:list")]
async fn list_roles(
    Component(service): Component<RoleAppService>,
    Query(query): Query<RoleQuery>,
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
    Query(query): Query<RoleTreeQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list_tree(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/roles/all")]
#[sa_check_permission("role:list")]
async fn list_all_roles(
    Component(service): Component<RoleAppService>,
    Query(query): Query<RoleTreeQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list_all(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/roles/{id}")]
#[sa_check_permission("role:list")]
async fn get_role_by_id(
    Component(service): Component<RoleAppService>,
    Path(id): Path<String>,
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
    Path(id): Path<String>,
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
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("@deleted_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/roles/{id}/menus")]
#[sa_check_permission("role:edit")]
async fn assign_role_menus(
    Component(service): Component<RoleAppService>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, WebError> {
    let menu_ids: Vec<String> = body.get("menuIds")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    Ok(match service.assign_menus(id, menu_ids).await {
        Ok(()) => Json(ApiResponse::success("@assigned_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/roles/{id}/menus")]
#[sa_check_permission("role:list")]
async fn get_role_menu_ids(
    Component(service): Component<RoleAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_role_menu_ids(id).await {
        Ok(ids) => Json(ApiResponse::success(ids)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 可导出行数（同步/异步分流判定）
#[get("/roles/export/count")]
#[sa_check_permission("role:export")]
async fn export_role_count(
    Component(service): Component<RoleAppService>,
    Query(query): Query<RoleExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.export_count(&query).await {
        Ok(total) => Json(ApiResponse::success(serde_json::json!({ "total": total }))),
        Err(e) => Json(ApiResponse::<serde_json::Value>::error(500, &e.to_string())),
    })
}

/// Excel 导出角色列表（≤ [`MAX_SYNC_ROWS`] 同步下载并在导出中心留记录；超过走异步任务）
#[get("/roles/export")]
#[sa_check_permission("role:export")]
async fn export_roles(
    Component(service): Component<RoleAppService>,
    Component(exports): Component<ExportTaskAppService>,
    Query(query): Query<RoleExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    match service.export_count(&query).await {
        Ok(total) if total > MAX_SYNC_ROWS => Ok(export_center_hint(total)),
        Ok(total) => {
            let query_json = serde_json::to_value(&query).unwrap_or_default();
            match service.export(query).await {
                Ok(vos) => Ok(sync_export_response(
                    "角色数据",
                    &exports,
                    "role",
                    &query_json,
                    "roles-export",
                    ROLE_HEADERS,
                    role_rows(&vos),
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
