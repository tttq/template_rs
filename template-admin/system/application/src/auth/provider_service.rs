use common::error::AppError;
use summer::plugin::service::Service;
use summer_sea_orm::DbConn;
use sa_token_core::StpUtil;

use super::provider::{BindDto, ProviderVo};

#[derive(Clone, Service)]
pub struct AuthProviderService {
    #[inject(component)]
    #[allow(dead_code)]
    db: DbConn,
}

impl AuthProviderService {
    pub async fn get_providers(&self) -> Result<Vec<ProviderVo>, AppError> {
        let providers = vec![ProviderVo::new(
            "github",
            "GitHub",
            Some("github"),
            "/oauth2/github/authorize",
        )];
        Ok(providers)
    }

    pub async fn bind(&self, dto: BindDto) -> Result<(), AppError> {
        let _user_id = StpUtil::get_login_id_as_string()
            .await
            .map_err(|e| AppError::Unauthorized(format!("未登录: {}", e)))?;

        if dto.code.is_empty() {
            return Err(AppError::BadRequest("授权码不能为空".to_string()));
        }

        Ok(())
    }
}