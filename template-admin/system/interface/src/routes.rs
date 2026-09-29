use summer_web::Router;
use summer_web::axum::body::{Body, to_bytes};
use summer_web::axum::extract::Request;
use summer_web::axum::http::{HeaderValue, StatusCode, header};
use summer_web::axum::middleware::{from_fn, Next};
use summer_web::axum::response::Response;

use common::i18n::{Language, resolve};
use common::response::ApiResponse;

/// 全局错误响应中间件
///
/// 拦截 `summer-sa-token` 权限校验失败（401/403）产生的英文错误响应，
/// 替换为项目统一的 `ApiResponse` JSON 格式与中文友好提示
/// （消息以 `@key` 形式给出，由外层 i18n 中间件按请求语言翻译）。
///
/// - 401 → `{ code: 401, message: "@unauthorized", data: null }`
/// - 403 → `{ code: 403, message: "@forbidden", data: null }`
///
/// 注意：业务错误统一以 HTTP 200 + `code:500` 返回，不会触发本中间件。
pub async fn error_handler_middleware(req: Request, next: Next) -> Response {
    let response = next.run(req).await;
    let status = response.status();

    match status {
        StatusCode::UNAUTHORIZED => rewrite_response(
            StatusCode::UNAUTHORIZED,
            ApiResponse::<()>::unauthorized("@unauthorized"),
        ),
        StatusCode::FORBIDDEN => rewrite_response(
            StatusCode::FORBIDDEN,
            ApiResponse::<()>::forbidden("@forbidden"),
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

/// 业务消息翻译中间件
///
/// 把响应 JSON 中 `message` 字段里以 `@` 开头的消息按请求语言
/// （Accept-Language，兼容 x-locale）翻译为目标语言文本。
///
/// - 非 JSON 响应（文件直读/导出/流等）原样透传；
/// - 非 `@` 开头消息与未知 key 原样透传（兼容存量中文与 OpenAI 网关英文）。
pub async fn i18n_middleware(req: Request, next: Next) -> Response {
    let header_val: Option<&HeaderValue> = req
        .headers()
        .get(header::ACCEPT_LANGUAGE)
        .or_else(|| req.headers().get("x-locale"));
    let lang = Language::from_accept_language(header_val.and_then(|v| v.to_str().ok()));
    let response = next.run(req).await;
    translate_response(response, lang).await
}

async fn translate_response(response: Response, lang: Language) -> Response {
    let is_json = response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|t| t.contains("application/json"))
        .unwrap_or(false);
    if !is_json {
        return response;
    }

    let (parts, body) = response.into_parts();
    let bytes = match to_bytes(body, usize::MAX).await {
        Ok(b) => b,
        // 读 body 失败不应吞掉响应：重建一个空 JSON 已不可能，返回原 status 的空体
        Err(_) => return Response::from_parts(parts, Body::empty()),
    };

    // 只有 JSON 对象且 message 以 @ 开头才需要翻译，快速短路避免大对象全量序列化
    if !bytes.starts_with(b"{") {
        return Response::from_parts(parts, Body::from(bytes));
    }
    let mut value: serde_json::Value = match serde_json::from_slice(&bytes) {
        Ok(v) => v,
        Err(_) => return Response::from_parts(parts, Body::from(bytes)),
    };
    let needs_translate = value
        .get("message")
        .and_then(|m| m.as_str())
        .map(|m| m.starts_with('@'))
        .unwrap_or(false);
    if !needs_translate {
        return Response::from_parts(parts, Body::from(bytes));
    }

    let msg = value
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or_default();
    let translated = resolve(msg, lang);
    if let Some(obj) = value.as_object_mut() {
        obj.insert("message".to_string(), serde_json::Value::String(translated));
    }
    let json = serde_json::to_vec(&value).unwrap_or_else(|_| bytes.to_vec());
    // headers / status 不变，仅替换 body
    Response::from_parts(parts, Body::from(json))
}

/// 将错误处理中间件包装为 layer，供 `LayerConfigurator::add_router_layer` 使用
pub fn with_error_handler(router: Router) -> Router {
    router.layer(from_fn(error_handler_middleware))
}

/// 将业务消息翻译中间件包装为 layer；
/// 注册在 with_error_handler 外层（后注册），以便同时翻译 401/403 与业务 code!=200 的 message
pub fn with_i18n(router: Router) -> Router {
    router.layer(from_fn(i18n_middleware))
}