use summer_web::{get, nest, post, put, delete};
use summer_web::extractor::{Component, Json, Query, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use system_application::user::dto::{CreateUserDto, UpdateUserDto, UserQuery, UserExportQuery};
use system_application::user::service::UserAppService;
use system_application::user::{USER_HEADERS, user_rows};
use system_application::export_task::service::ExportTaskAppService;
use super::list_export::{MAX_SYNC_ROWS, error_response, export_center_hint, sync_export_response};

#[nest("/system")]
mod controller {
    use super::*;

#[get("/users")]
#[sa_check_permission("user:list")]
async fn list_users(
    Component(service): Component<UserAppService>,
    Query(query): Query<UserQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/users/{id}")]
#[sa_check_permission("user:list")]
async fn get_user_by_id(
    Component(service): Component<UserAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_id(id).await {
        Ok(user) => Json(ApiResponse::success(user)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/users")]
#[sa_check_permission("user:add")]
async fn create_user(
    Component(service): Component<UserAppService>,
    Json(dto): Json<CreateUserDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create(dto).await {
        Ok(user) => Json(ApiResponse::success(user)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/users/{id}")]
#[sa_check_permission("user:edit")]
async fn update_user(
    Component(service): Component<UserAppService>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<impl IntoResponse, WebError> {
    let mut dto = dto;
    dto.id = Some(id);
    Ok(match service.update(dto).await {
        Ok(user) => Json(ApiResponse::success(user)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/users/{id}/status")]
#[sa_check_permission("user:edit")]
async fn update_user_status(
    Component(service): Component<UserAppService>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, WebError> {
    let status = body.get("status").and_then(|v| v.as_i64()).unwrap_or(1) as i32;
    Ok(match service.update_status(id, status).await {
        Ok(()) => Json(ApiResponse::success("@status_updated_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[delete("/users/{id}")]
#[sa_check_permission("user:delete")]
async fn delete_user(
    Component(service): Component<UserAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("@deleted_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/users/{id}/roles")]
#[sa_check_permission("user:edit")]
async fn assign_user_roles(
    Component(service): Component<UserAppService>,
    Path(id): Path<String>,
    Json(body): Json<serde_json::Value>,
) -> Result<impl IntoResponse, WebError> {
    let role_ids: Vec<String> = body.get("roleIds")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    Ok(match service.assign_roles(id, role_ids).await {
        Ok(()) => Json(ApiResponse::success("@assigned_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/users/{id}/roles")]
#[sa_check_permission("user:list")]
async fn get_user_role_ids(
    Component(service): Component<UserAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_role_ids(id).await {
        Ok(ids) => Json(ApiResponse::success(ids)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 可导出行数（同步/异步分流判定）
#[get("/users/export/count")]
#[sa_check_permission("user:export")]
async fn export_user_count(
    Component(service): Component<UserAppService>,
    Query(query): Query<UserExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.export_count(&query).await {
        Ok(total) => Json(ApiResponse::success(serde_json::json!({ "total": total }))),
        Err(e) => Json(ApiResponse::<serde_json::Value>::error(500, &e.to_string())),
    })
}

/// Excel 导出用户列表（≤ [`MAX_SYNC_ROWS`] 同步下载并在导出中心留记录；超过走异步任务）
#[get("/users/export")]
#[sa_check_permission("user:export")]
async fn export_users(
    Component(service): Component<UserAppService>,
    Component(exports): Component<ExportTaskAppService>,
    Query(query): Query<UserExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    match service.export_count(&query).await {
        Ok(total) if total > MAX_SYNC_ROWS => Ok(export_center_hint(total)),
        Ok(total) => {
            let query_json = serde_json::to_value(&query).unwrap_or_default();
            match service.export(query).await {
                Ok(vos) => Ok(sync_export_response(
                    "用户数据",
                    &exports,
                    "user",
                    &query_json,
                    "users-export",
                    USER_HEADERS,
                    user_rows(&vos),
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
