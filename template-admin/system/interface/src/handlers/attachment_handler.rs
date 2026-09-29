//! 统一附件管理（/api/system/attachment*）：上传 + 列表 + 分页 + 直读 + 删除
//! body 为原始文件字节，业务参数走 query。
//! 直读 GET /api/system/attachment/{id} 供前端预览（登录态校验）。

use common::response::ApiResponse;
use system_application::attachment::dto::{AttachmentMetaUpdateRequest, AttachmentQuery, AttachmentReadQuery};
use system_application::attachment::service::AttachmentService;
use summer_sa_token::sa_check_permission;
use summer_web::axum::body::{Body, Bytes};
use summer_web::axum::http::{HeaderMap, StatusCode};
use summer_web::axum::response::{IntoResponse, Response};
use summer_web::error::WebError;
use summer_web::extractor::{Component, Json, Path, Query};
use summer_web::{delete, get, nest, post, put};

use common::get_current_user_id;
use common::get_current_user_name;

#[nest("/system")]
mod controller {
    use super::*;

#[post("/attachment")]
#[sa_check_permission("attachment:upload")]
async fn upload(
    Component(service): Component<AttachmentService>,
    headers: HeaderMap,
    Query(params): Query<serde_json::Value>,
    body: Bytes,
) -> Result<Response, WebError> {
    let uid = get_current_user_id().unwrap_or_default();
    let uname = get_current_user_name().unwrap_or_default();

    let category = params
        .get("category")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    if category.is_empty() {
        return Ok(error_response(400, "category 不能为空"));
    }
    let biz_type = params
        .get("bizType")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    if biz_type.is_empty() {
        return Ok(error_response(400, "bizType 不能为空"));
    }
    let biz_id = params
        .get("bizId")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let image_type = params
        .get("imageType")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    // sort 等数值查询参数经 serde_urlencoded 反序列化后为字符串 Value，
    // as_i64() 会失效导致恒为 0，需兼容字符串解析
    let sort = params
        .get("sort")
        .and_then(|v| v.as_i64().or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok())))
        .unwrap_or(0) as i32;
    let caption = params
        .get("caption")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let file_name = params
        .get("fileName")
        .and_then(|v| v.as_str())
        .unwrap_or("upload.bin")
        .to_string();
    let remark = params
        .get("remark")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let mime = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    Ok(match service
        .upload(
            &uid,
            &uname,
            &category,
            &biz_type,
            biz_id.as_deref(),
            image_type.as_deref(),
            sort,
            caption.as_deref(),
            &file_name,
            &mime,
            remark.as_deref(),
            body.to_vec(),
        )
        .await
    {
        Ok(v) => Json(ApiResponse::success(v)).into_response(),
        Err(e) => error_response(500, &e.to_string()),
    })
}

#[get("/attachment")]
#[sa_check_permission("attachment:list")]
async fn list(
    Component(service): Component<AttachmentService>,
    Query(query): Query<AttachmentQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list(&query).await {
        Ok(v) => Json(ApiResponse::success(v)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 更新附件元信息（排序 / 主图标记），供图片位置调整使用
#[put("/attachment/{id}/meta")]
#[sa_check_permission("attachment:upload")]
async fn update_meta(
    Component(service): Component<AttachmentService>,
    Path(id): Path<String>,
    Json(req): Json<AttachmentMetaUpdateRequest>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.update_meta(&id, &req).await {
        Ok(v) => Json(ApiResponse::success(v)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/attachment/{id}")]
#[sa_check_permission("attachment:read")]
async fn read(
    Component(service): Component<AttachmentService>,
    Path(id): Path<String>,
    Query(query): Query<AttachmentReadQuery>,
) -> Result<Response, WebError> {
    Ok(match service.read_with_biz(&id, query.biz_type.as_deref(), query.biz_id.as_deref()).await {
        Ok((bytes, mime, _name)) => (
            StatusCode::OK,
            [("content-type", Box::leak(mime.into_boxed_str()) as &'static str)],
            Body::from(bytes),
        )
            .into_response(),
        Err(e) => error_response(404, &e.to_string()),
    })
}

#[delete("/attachment/{id}")]
#[sa_check_permission("attachment:delete")]
async fn delete_one(
    Component(service): Component<AttachmentService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(&id).await {
        Ok(_) => Json(ApiResponse::success("@deleted_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[delete("/attachment/batch")]
#[sa_check_permission("attachment:delete")]
async fn batch_delete(
    Component(service): Component<AttachmentService>,
    Json(ids): Json<Vec<String>>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.batch_delete(&ids).await {
        Ok(_) => Json(ApiResponse::success("@batch_delete_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}
}

fn error_response(code: u16, msg: &str) -> Response {
    (
        StatusCode::from_u16(code).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        [("content-type", "application/json")],
        Body::from(
            serde_json::to_string(&ApiResponse::<()>::error(code as i32, msg)).unwrap_or_default(),
        ),
    )
        .into_response()
}