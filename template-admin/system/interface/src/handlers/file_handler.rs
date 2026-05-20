use summer_web::{post, Router};
use summer_web::extractor::Json;
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_web::handler::TypeRouter;
use summer_sa_token::sa_check_login;
use common::response::ApiResponse;

pub fn routes() -> Router {
    Router::new()
        .typed_route(upload_file)
}

#[post("/upload")]
#[sa_check_login]
async fn upload_file() -> Result<impl IntoResponse, WebError> {
    Ok(Json(ApiResponse::success(serde_json::json!({
        "url": "",
        "fileName": "",
    }))))
}
