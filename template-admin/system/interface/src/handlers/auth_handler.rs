use summer_web::{post, get, nest, put};
use summer_web::extractor::{Component, Json, Path};
use summer_web::axum::body::Body;
use summer_web::axum::http::{HeaderMap, StatusCode};
use summer_web::axum::response::{IntoResponse, Response};
use summer_web::error::WebError;
use summer_sa_token::{sa_check_login, sa_ignore};
use common::response::ApiResponse;
use system_application::auth::code_service::AuthCodeService;
use system_application::auth::dto::{
    ChangeEmailDto, ChangePasswordDto, CompleteProfileDto, LocateTenantDto, LoginDto,
    RefreshTokenDto, RegisterDto, SendCodeDto, UpdateProfileDto,
};
use system_application::auth::service::AuthAppService;
use system_application::auth::provider::BindDto;
use system_application::auth::provider_service::AuthProviderService;

/// 从请求头解析客户端 IP（网关/反代下取 X-Forwarded-For 首段）。
///
/// 图形验证码与发码限速按 IP 收敛，取不到时退回 "unknown"（限速仍然生效）。
fn client_ip(headers: &HeaderMap) -> Option<String> {
    for name in ["x-forwarded-for", "x-real-ip"] {
        if let Some(v) = headers.get(name).and_then(|v| v.to_str().ok()) {
            let first = v.split(',').next().unwrap_or("").trim();
            if !first.is_empty() {
                return Some(first.to_string());
            }
        }
    }
    None
}

#[nest("/auth")]
mod controller {
    use super::*;

#[post("/login")]
#[sa_ignore]
async fn do_login(
    Component(service): Component<AuthAppService>,
    Json(dto): Json<LoginDto>,
) -> impl IntoResponse {
    match service.login(dto).await {
        Ok(token) => Json(ApiResponse::success(token)),
        // 428 Precondition Required：密码正确但需要两步验证码，前端据此弹出动态码输入
        Err(common::error::AppError::TwoFactorRequired(msg)) => {
            Json(ApiResponse::error(428, &msg))
        }
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

#[post("/register")]
#[sa_ignore]
async fn do_register(
    Component(service): Component<AuthAppService>,
    Json(dto): Json<RegisterDto>,
) -> impl IntoResponse {
    match service.register(dto).await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

#[get("/user-info")]
#[sa_check_login]
async fn do_get_user_info(
    Component(service): Component<AuthAppService>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.get_user_info().await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(401, &e.to_string())),
    })
}

#[post("/logout")]
#[sa_check_login]
async fn do_logout(
    Component(service): Component<AuthAppService>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.logout().await {
        Ok(()) => Json(ApiResponse::success("@logout_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/refresh-token")]
#[sa_ignore]
async fn do_refresh_token(
    Component(service): Component<AuthAppService>,
    Json(dto): Json<RefreshTokenDto>,
) -> impl IntoResponse {
    match service.refresh_token(&dto.refresh_token).await {
        Ok(token) => Json(ApiResponse::success(token)),
        Err(e) => Json(ApiResponse::error(401, &e.to_string())),
    }
}

#[get("/providers")]
#[sa_ignore]
async fn get_providers(
    Component(service): Component<AuthProviderService>,
) -> impl IntoResponse {
    match service.get_providers().await {
        Ok(providers) => Json(ApiResponse::success(providers)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

#[post("/bind")]
#[sa_check_login]
async fn bind_provider(
    Component(service): Component<AuthProviderService>,
    Json(dto): Json<BindDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.bind(dto).await {
        Ok(()) => Json(ApiResponse::success("@bind_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 当前账号已绑定的第三方身份（如 wechat_mp）
#[get("/bindings")]
#[sa_check_login]
async fn list_bindings(
    Component(service): Component<AuthProviderService>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.list_bindings().await {
        Ok(v) => Json(ApiResponse::success(v)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 解绑第三方身份（body { provider }，仅当前登录人自己的绑定）
#[post("/unbind")]
#[sa_check_login]
async fn unbind_provider(
    Component(service): Component<AuthProviderService>,
    Json(dto): Json<BindDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.unbind(dto).await {
        Ok(()) => Json(ApiResponse::success("@unbind_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

#[post("/locate")]
#[sa_ignore]
async fn locate_tenant(
    Component(service): Component<AuthAppService>,
    Json(dto): Json<LocateTenantDto>,
) -> impl IntoResponse {
    match service.locate_tenant(dto).await {
        Ok(vo) => Json(ApiResponse::success(vo)),
        Err(e) => Json(ApiResponse::error(401, &e.to_string())),
    }
}

/// 完善账户信息（第三方自动注册后的惰性账号首次进入系统前必填）
#[post("/complete-profile")]
#[sa_check_login]
async fn complete_profile(
    Component(service): Component<AuthAppService>,
    Json(dto): Json<CompleteProfileDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.complete_profile(dto).await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 注册身份选项（免登录；全端共用同一份字典，编码服务端写死，不接受任何参数）
#[get("/register-roles")]
#[sa_ignore]
async fn register_roles(
    Component(service): Component<AuthAppService>,
) -> impl IntoResponse {
    match service.register_roles().await {
        Ok(v) => Json(ApiResponse::success(v)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// Web 微信扫码登录：返回 qrconnect 地址与一次性 CSRF state（免登录）
#[get("/wechat/qr-url")]
#[sa_ignore]
async fn wechat_qr_url(
    Component(service): Component<AuthAppService>,
) -> impl IntoResponse {
    match service.wechat_qr_url().await {
        Ok(vo) => Json(ApiResponse::success(vo)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 修改个人信息（昵称 / 手机号）
#[put("/profile")]
#[sa_check_login]
async fn update_profile(
    Component(service): Component<AuthAppService>,
    Json(dto): Json<UpdateProfileDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.update_profile(dto).await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 修改密码（需账号当前邮箱的验证码，scene=change_password）
#[post("/change-password")]
#[sa_check_login]
async fn change_password(
    Component(service): Component<AuthAppService>,
    Json(dto): Json<ChangePasswordDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.change_password(dto).await {
        Ok(()) => Json(ApiResponse::success("@password_changed")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 更换绑定邮箱（需新邮箱的验证码，scene=change_email）
#[post("/change-email")]
#[sa_check_login]
async fn change_email(
    Component(service): Component<AuthAppService>,
    Json(dto): Json<ChangeEmailDto>,
) -> Result<impl IntoResponse, WebError> {
    Ok(match service.change_email(dto).await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}

/// 获取图形验证码（免登录）：返回 captchaId + PNG data URI，供注册/邮箱验证码登录/发码前置校验
#[get("/captcha")]
#[sa_ignore]
async fn get_captcha(
    Component(service): Component<AuthCodeService>,
    headers: HeaderMap,
) -> impl IntoResponse {
    match service.generate_captcha(client_ip(&headers).as_deref()).await {
        Ok(vo) => Json(ApiResponse::success(vo)),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}

/// 图形验证码原图（免登录）：小程序 <image> 按 URL 直接加载
#[get("/captcha-image/{id}")]
#[sa_ignore]
async fn get_captcha_image(
    Component(service): Component<AuthCodeService>,
    Path(id): Path<String>,
) -> Response {
    match service.captcha_png(&id).await {
        Ok(png) => (
            StatusCode::OK,
            [("content-type", "image/png")],
            Body::from(png),
        )
            .into_response(),
        Err(e) => (
            StatusCode::NOT_FOUND,
            [("content-type", "application/json")],
            Body::from(
                serde_json::to_string(&ApiResponse::<()>::error(404, &e.to_string()))
                    .unwrap_or_default(),
            ),
        )
            .into_response(),
    }
}

/// 发送邮箱验证码（免登录）：先校验图形验证码，按通知模板投递邮件
#[post("/send-code")]
#[sa_ignore]
async fn send_code(
    Component(service): Component<AuthCodeService>,
    headers: HeaderMap,
    Json(dto): Json<SendCodeDto>,
) -> impl IntoResponse {
    match service.send_code(dto, client_ip(&headers).as_deref()).await {
        Ok(()) => Json(ApiResponse::success("@captcha_sent_ok")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}
}