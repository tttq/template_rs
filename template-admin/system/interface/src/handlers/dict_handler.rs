use summer_web::{get, nest, post, put, delete};
use summer_web::extractor::{Component, Json, Query, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_sa_token::{sa_check_login, sa_check_permission};
use common::response::ApiResponse;
use common::pagination::PageQuery;
use system_application::dict_type::dto::{CreateDictTypeDto, UpdateDictTypeDto, DictTypeQuery, DictTypeExportQuery};
use system_application::dict_type::service::DictTypeAppService;
use system_application::dict_item::dto::{CreateDictItemDto, UpdateDictItemDto};
use system_application::dict_item::service::DictItemAppService;
use system_application::dict_type::{DICT_HEADERS, dict_rows};
use system_application::export_task::service::ExportTaskAppService;
use super::list_export::{MAX_SYNC_ROWS, error_response, export_center_hint, sync_export_response};

#[nest("/system")]
mod controller {
    use super::*;

#[get("/dicts/types")]
#[sa_check_permission("dict:list")]
async fn list_dict_types(
    Component(service): Component<DictTypeAppService>,
    Query(query): Query<DictTypeQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/dicts/types")]
#[sa_check_permission("dict:add")]
async fn create_dict_type(
    Component(service): Component<DictTypeAppService>,
    Json(dto): Json<CreateDictTypeDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create(dto).await {
        Ok(dict) => Json(ApiResponse::success(dict)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/dicts/types/{id}")]
#[sa_check_permission("dict:edit")]
async fn update_dict_type(
    Component(service): Component<DictTypeAppService>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateDictTypeDto>,
) -> Result<impl IntoResponse, WebError> {
    let mut dto = dto;
    dto.id = Some(id);
    Ok(match service.update(dto).await {
        Ok(dict) => Json(ApiResponse::success(dict)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[delete("/dicts/types/{id}")]
#[sa_check_permission("dict:delete")]
async fn delete_dict_type(
    Component(service): Component<DictTypeAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("@deleted_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/dicts/items")]
#[sa_check_permission("dict:list")]
async fn list_dict_items(
    Component(service): Component<DictItemAppService>,
    Query(query): Query<PageQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/dicts/items/by-type/{typeId}")]
#[sa_check_permission("dict:list")]
async fn list_items_by_type_id(
    Component(service): Component<DictItemAppService>,
    Path(type_id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_dict_type_id(type_id).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/dicts/types/{typeCode}/items")]
#[sa_check_permission("dict:list")]
async fn list_items_by_type_code(
    Component(service): Component<DictItemAppService>,
    Path(type_code): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_dict_type_code(type_code).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 按字典类型编码取**启用中**的字典项选项（供业务页面渲染下拉）
///
/// 仅需登录：字典属于配置数据，业务页面（如采购的工厂类型、订单状态）
/// 不应因当前角色没有 `dict:list` 而拿不到选项。
#[get("/dicts/options/{typeCode}")]
#[sa_check_login]
async fn dict_options_by_type_code(
    Component(service): Component<DictItemAppService>,
    Path(type_code): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.options_by_type_code(&type_code).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/dicts/items")]
#[sa_check_permission("dict:add")]
async fn create_dict_item(
    Component(service): Component<DictItemAppService>,
    Json(dto): Json<CreateDictItemDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create(dto).await {
        Ok(item) => Json(ApiResponse::success(item)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/dicts/items/{id}")]
#[sa_check_permission("dict:edit")]
async fn update_dict_item(
    Component(service): Component<DictItemAppService>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateDictItemDto>,
) -> Result<impl IntoResponse, WebError> {
    let mut dto = dto;
    dto.id = Some(id);
    Ok(match service.update(dto).await {
        Ok(item) => Json(ApiResponse::success(item)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[delete("/dicts/items/{id}")]
#[sa_check_permission("dict:delete")]
async fn delete_dict_item(
    Component(service): Component<DictItemAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("@deleted_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 可导出行数（同步/异步分流判定）
#[get("/dicts/types/export/count")]
#[sa_check_permission("dict:export")]
async fn export_dict_type_count(
    Component(service): Component<DictTypeAppService>,
    Query(query): Query<DictTypeExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.export_count(&query).await {
        Ok(total) => Json(ApiResponse::success(serde_json::json!({ "total": total }))),
        Err(e) => Json(ApiResponse::<serde_json::Value>::error(500, &e.to_string())),
    })
}

/// Excel 导出字典类型列表（≤ [`MAX_SYNC_ROWS`] 同步下载并在导出中心留记录；超过走异步任务）
#[get("/dicts/types/export")]
#[sa_check_permission("dict:export")]
async fn export_dict_types(
    Component(service): Component<DictTypeAppService>,
    Component(exports): Component<ExportTaskAppService>,
    Query(query): Query<DictTypeExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    match service.export_count(&query).await {
        Ok(total) if total > MAX_SYNC_ROWS => Ok(export_center_hint(total)),
        Ok(total) => {
            let query_json = serde_json::to_value(&query).unwrap_or_default();
            match service.export(query).await {
                Ok(vos) => Ok(sync_export_response(
                    "字典类型",
                    &exports,
                    "dict",
                    &query_json,
                    "dict-types-export",
                    DICT_HEADERS,
                    dict_rows(&vos),
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
