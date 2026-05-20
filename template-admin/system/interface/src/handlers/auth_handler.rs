use summer_web::{post, get, Router};
use summer_web::extractor::{Component, Json};
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_web::handler::TypeRouter;
use summer_sa_token::{sa_check_login, sa_ignore};
use common::response::ApiResponse;
use system_application::auth::dto::{LoginDto, RegisterDto};
use system_application::auth::service::AuthAppService;
use system_application::auth::provider::BindDto;
use system_application::auth::provider_service::AuthProviderService;

pub fn routes() -> Router {
    Router::new()
        .typed_route(do_login)
        .typed_route(do_register)
        .typed_route(do_get_user_info)
        .typed_route(do_logout)
        .typed_route(get_providers)
        .typed_route(bind_provider)
}

#[post("/login")]
#[sa_ignore]
async fn do_login(
    Component(service): Component<AuthAppService>,
    Json(dto): Json<LoginDto>,
) -> impl IntoResponse {
    match service.login(dto).await {
        Ok(token) => Json(ApiResponse::success(token)),
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
        Ok(()) => Json(ApiResponse::success("登出成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
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
        Ok(()) => Json(ApiResponse::success("绑定成功")),
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    })
}
