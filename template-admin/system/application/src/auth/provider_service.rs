use common::error::AppError;
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::*;
use sea_orm::{ColumnTrait, QueryFilter};
use sea_orm_ext::{ignore_tenant, DbConn};
use sa_token_core::StpUtil;
use summer::plugin::service::Service;
use summer_redis::Redis;
use system_entity::user_identity;

use super::provider::{BindDto, ProviderBindingVo, ProviderVo};
use super::service::{
    consume_qr_state, find_identity_row, is_wechat_provider, resolve_third_party_identity,
};

#[derive(Clone, Service)]
pub struct AuthProviderService {
    #[inject(component)]
    db: DbConn,
    #[inject(component)]
    redis: Redis,
}

impl AuthProviderService {
    pub async fn get_providers(&self) -> Result<Vec<ProviderVo>, AppError> {
        let providers = vec![
            ProviderVo::new("wechat_mp", "微信小程序", Some("wechat"), ""),
            ProviderVo::new("wechat", "微信扫码", Some("wechat"), ""),
            ProviderVo::new("github", "GitHub", Some("github"), "/oauth2/github/authorize"),
        ];
        Ok(providers)
    }

    /// 绑定第三方身份到当前登录账号。
    ///
    /// 流程（`wechat` / `wechat_mp` 对称）：
    /// 1. 换 `identity_value`（微信类取 unionid，拿不到回退 openid）；
    ///    `provider = wechat`（Web 扫码）还需先校验并消费 `qr_state`
    /// 2. 按 `identity_value` 查身份表（微信类不限 provider）：
    ///    命中且属于当前用户 → 幂等成功；命中且属于他人 → 「该微信已绑定其他账号」
    /// 3. 未命中 → 检查当前账号是否**已绑过微信**：有则「当前账号已绑定微信，请先解绑」
    ///    —— **一个账号只能绑一个微信**，是硬约束
    /// 4. 插入新行（数据库层的两条部分唯一索引是最后一道保险）
    #[ignore_tenant]
    pub async fn bind(&self, dto: BindDto) -> Result<(), AppError> {
        let login_id = StpUtil::get_login_id_as_string()
            .await
            .map_err(|e| AppError::Unauthorized(format!("@not_logged_in:{}", e)))?;
        // 客户端会话的 login_id 形如 `用户ID#客户端编码`；身份表存真实用户ID
        let user_id = common::user::base_user_id_from_login_id(&login_id);

        if dto.code.trim().is_empty() {
            return Err(AppError::BadRequest("@code_required".to_string()));
        }
        // Web 扫码绑定：先校验并一次性消费 CSRF state
        if dto.provider == "wechat" {
            consume_qr_state(&self.redis, dto.qr_state.as_deref()).await?;
        }

        let identity = resolve_third_party_identity(&dto.provider, dto.code.trim()).await?;
        let wechat = is_wechat_provider(&dto.provider);

        // 该第三方身份已绑定的账号
        if let Some(row) = find_identity_row(&self.db, &identity).await? {
            if row.user_id == user_id {
                return Ok(()); // 幂等
            }
            return Err(AppError::BadRequest("@wechat_bound_other".to_string()));
        }

        // 一个账号只能绑一个微信：当前账号已有微信行时拒绝
        if wechat {
            let existing = user_identity::Entity::find()
                .filter(user_identity::Column::UserId.eq(&user_id))
                .filter(user_identity::Column::Status.eq(1))
                .filter(user_identity::Column::Provider.is_in(vec![
                    "wechat_mp".to_string(),
                    "wechat".to_string(),
                ]))
                .one(&self.db)
                .await?;
            if existing.is_some() {
                return Err(AppError::BadRequest("@wechat_already_bound".to_string()));
            }
        }

        user_identity::ActiveModel {
            user_id: Set(user_id),
            provider: Set(dto.provider),
            identity_value: Set(identity.identity_value),
            nick_name: Set(None),
            status: Set(1),
            ..Default::default()
        }
        .insert(&self.db)
        .await?;

        Ok(())
    }

    /// 当前登录账号已绑定的第三方身份列表（供前端展示绑定状态）。
    #[ignore_tenant]
    pub async fn list_bindings(&self) -> Result<Vec<ProviderBindingVo>, AppError> {
        let login_id = StpUtil::get_login_id_as_string()
            .await
            .map_err(|e| AppError::Unauthorized(format!("@not_logged_in:{}", e)))?;
        let user_id = common::user::base_user_id_from_login_id(&login_id);

        let rows = user_identity::Entity::find()
            .filter(user_identity::Column::UserId.eq(&user_id))
            .filter(user_identity::Column::Status.eq(1))
            .all(&self.db)
            .await?;
        Ok(rows
            .into_iter()
            .map(|r| ProviderBindingVo {
                provider: r.provider,
            })
            .collect())
    }

    /// 解绑第三方身份（仅当前登录人自己的绑定）。
    ///
    /// 微信类一次清掉该账号的**所有**微信行（正常只有一行；用 `provider ∈ 微信类` 删除更稳），
    /// 且不再误删 `tenant_user` 的租户索引行（历史 bug：解绑会把用户的租户索引一起删掉，
    /// 导致密码登录 401）。
    #[ignore_tenant]
    pub async fn unbind(&self, dto: BindDto) -> Result<(), AppError> {
        let login_id = StpUtil::get_login_id_as_string()
            .await
            .map_err(|e| AppError::Unauthorized(format!("@not_logged_in:{}", e)))?;
        let user_id = common::user::base_user_id_from_login_id(&login_id);

        let mut delete = user_identity::Entity::delete_many()
            .filter(user_identity::Column::UserId.eq(&user_id));
        if is_wechat_provider(&dto.provider) {
            delete = delete.filter(user_identity::Column::Provider.is_in(vec![
                "wechat_mp".to_string(),
                "wechat".to_string(),
            ]));
        } else {
            delete = delete.filter(user_identity::Column::Provider.eq(&dto.provider));
        }

        let result = delete
            .exec(&self.db)
            .await
            .map_err(|e| AppError::Internal(format!("@unbind_failed:{}", e)))?;
        if result.rows_affected == 0 {
            return Err(AppError::BadRequest("@not_bound".to_string()));
        }
        Ok(())
    }
}
