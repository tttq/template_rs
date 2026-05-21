use summer_web::{get, post, Router};
use summer_web::extractor::{Component, Json, Path};
use summer_web::axum::response::{IntoResponse, Response};
use summer_web::axum::http::{StatusCode, header};
use summer_web::error::WebError;
use summer_web::handler::TypeRouter;
use summer_sa_token::sa_check_login;
use common::response::ApiResponse;
use system_application::generator::dto::GeneratorConfig;
use system_application::generator::service::GeneratorAppService;

pub fn routes() -> Router {
    Router::new()
        .typed_route(list_tables)
        .typed_route(get_table_columns)
        .typed_route(preview_code)
        .typed_route(download_code)
}

#[get("/tables")]
#[sa_check_login]
async fn list_tables(
    Component(service): Component<GeneratorAppService>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list_tables().await {
        Ok(tables) => Json(ApiResponse::success(tables)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[get("/tables/{table_name}/columns")]
#[sa_check_login]
async fn get_table_columns(
    Component(service): Component<GeneratorAppService>,
    Path(table_name): Path<String>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_table_columns(&table_name).await {
        Ok(columns) => Json(ApiResponse::success(columns)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/preview")]
#[sa_check_login]
async fn preview_code(
    Component(service): Component<GeneratorAppService>,
    Json(config): Json<GeneratorConfig>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.preview(config).await {
        Ok(preview) => Json(ApiResponse::success(preview)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/download")]
#[sa_check_login]
async fn download_code(
    Component(service): Component<GeneratorAppService>,
    Json(config): Json<GeneratorConfig>,
) -> Result<Response, WebError> {
    match service.download(config).await {
        Ok(zip_data) => {
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/zip")
                .header(header::CONTENT_DISPOSITION, "attachment; filename=generated-code.zip")
                .body(zip_data.into())
                .unwrap())
        }
        Err(e) => Ok(Json(ApiResponse::<()>::error(500, &e.to_string())).into_response()),
    }
}
