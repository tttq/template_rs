use summer_web::Router;
use summer_web::axum::body::Body;
use summer_web::axum::extract::Request;
use summer_web::axum::http::StatusCode;
use summer_web::axum::middleware::{from_fn, Next};
use summer_web::axum::response::Response;

use common::response::ApiResponse;

pub fn system_routes() -> Router {
    Router::new()
        .nest("/api/auth", super::handlers::auth_routes())
        .nest("/api/system", super::handlers::system_routes())
}

/// 全局错误响应中间件
///
/// 拦截 `summer-sa-token` 权限校验失败（401/403）产生的英文错误响应，
/// 替换为项目统一的 `ApiResponse` JSON 格式与中文友好提示。
///
/// - 401 → `{ code: 401, message: "未登录或登录已过期", data: null }`
/// - 403 → `{ code: 403, message: "无权限访问", data: null }`
///
/// 注意：业务错误统一以 HTTP 200 + `code:500` 返回，不会触发本中间件。
pub async fn error_handler_middleware(req: Request, next: Next) -> Response {
    let response = next.run(req).await;
    let status = response.status();

    match status {
        StatusCode::UNAUTHORIZED => rewrite_response(
            StatusCode::UNAUTHORIZED,
            ApiResponse::<()>::unauthorized("未登录或登录已过期"),
        ),
        StatusCode::FORBIDDEN => rewrite_response(
            StatusCode::FORBIDDEN,
            ApiResponse::<()>::forbidden("无权限访问"),
        ),
        _ => response,
    }
}

/// 将响应体重写为 `ApiResponse` JSON
fn rewrite_response(status: StatusCode, body: ApiResponse<()>) -> Response {
    let json = serde_json::to_string(&body).unwrap_or_else(|_| "{}".to_string());
    Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .body(Body::from(json))
        .unwrap()
}

/// 将错误处理中间件包装为 layer，供 `LayerConfigurator::add_router_layer` 使用
pub fn with_error_handler(router: Router) -> Router {
    router.layer(from_fn(error_handler_middleware))
}
