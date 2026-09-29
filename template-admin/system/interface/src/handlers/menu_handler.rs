use summer_web::{get, nest, post, put, delete};
use summer_web::extractor::{Component, Json, Query, Path};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_sa_token::sa_check_permission;
use common::response::ApiResponse;
use system_application::menu::dto::{CreateMenuDto, UpdateMenuDto, MenuQuery, MenuExportQuery, MenuTreeQuery};
use system_application::menu::service::MenuAppService;
use system_application::menu::{MENU_HEADERS, menu_rows};
use system_application::export_task::service::ExportTaskAppService;
use super::list_export::{MAX_SYNC_ROWS, error_response, export_center_hint, sync_export_response};

#[nest("/system")]
mod controller {
    use super::*;

#[get("/menus")]
#[sa_check_permission("menu:list")]
async fn list_menus(
    Component(service): Component<MenuAppService>,
    Query(query): Query<MenuQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/menus/tree")]
#[sa_check_permission("menu:list")]
async fn list_menu_tree(
    Component(service): Component<MenuAppService>,
    Query(query): Query<MenuTreeQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list_tree(query).await {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/menus/{id}")]
#[sa_check_permission("menu:list")]
async fn get_menu_by_id(
    Component(service): Component<MenuAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_by_id(id).await {
        Ok(menu) => Json(ApiResponse::success(menu)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/menus")]
#[sa_check_permission("menu:add")]
async fn create_menu(
    Component(service): Component<MenuAppService>,
    Json(dto): Json<CreateMenuDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.create(dto).await {
        Ok(menu) => Json(ApiResponse::success(menu)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[put("/menus/{id}")]
#[sa_check_permission("menu:edit")]
async fn update_menu(
    Component(service): Component<MenuAppService>,
    Path(id): Path<String>,
    Json(dto): Json<UpdateMenuDto>,
) -> Result<impl IntoResponse, WebError> {
    let mut dto = dto;
    dto.id = Some(id);
    Ok(match service.update(dto).await {
        Ok(menu) => Json(ApiResponse::success(menu)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[delete("/menus/{id}")]
#[sa_check_permission("menu:delete")]
async fn delete_menu(
    Component(service): Component<MenuAppService>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.delete(id).await {
        Ok(()) => Json(ApiResponse::success("@deleted_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 可导出行数（同步/异步分流判定）
#[get("/menus/export/count")]
#[sa_check_permission("menu:export")]
async fn export_menu_count(
    Component(service): Component<MenuAppService>,
    Query(query): Query<MenuExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.export_count(&query).await {
        Ok(total) => Json(ApiResponse::success(serde_json::json!({ "total": total }))),
        Err(e) => Json(ApiResponse::<serde_json::Value>::error(500, &e.to_string())),
    })
}

/// Excel 导出菜单列表（≤ [`MAX_SYNC_ROWS`] 同步下载并在导出中心留记录；超过走异步任务）
#[get("/menus/export")]
#[sa_check_permission("menu:export")]
async fn export_menus(
    Component(service): Component<MenuAppService>,
    Component(exports): Component<ExportTaskAppService>,
    Query(query): Query<MenuExportQuery>,
) -> Result<impl IntoResponse, WebError> {
    match service.export_count(&query).await {
        Ok(total) if total > MAX_SYNC_ROWS => Ok(export_center_hint(total)),
        Ok(total) => {
            let query_json = serde_json::to_value(&query).unwrap_or_default();
            match service.export(query).await {
                Ok(vos) => Ok(sync_export_response(
                    "菜单数据",
                    &exports,
                    "menu",
                    &query_json,
                    "menus-export",
                    MENU_HEADERS,
                    menu_rows(&vos),
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
