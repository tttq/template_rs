use super::dto::{
    ChangeEmailDto, ChangePasswordDto, CompleteProfileDto, LocateTenantDto, LocateTenantVo,
    LoginDto, RegisterDto, RegisterRoleVo, TokenVo, UpdateProfileDto, UserInfoVo, WechatQrVo,
};
use crate::menu::dto::MenuVo;
use common::error::AppError;
use sa_token_adapter::storage::SaStorage;
use sa_token_core::refresh::RefreshTokenManager;
use sa_token_core::StpUtil;
use sea_orm::prelude::*;
use sea_orm::{ActiveValue::Set, ColumnTrait, QueryFilter, QueryOrder, TransactionTrait};
use sea_orm_ext::{get_tenant_config, ignore_tenant, tenant_scope, TenantIgnoreGuard};
use sea_query::Value;
use std::sync::Arc;
use summer::plugin::service::Service;
use summer_redis::Redis;
use summer_sa_token::storage::SummerRedisStorage;
use summer_sa_token::{CoreConfig, SaTokenConfig};
use sea_orm_ext::DbConn;
use system_entity::{
    client, dict_item, dict_type, role, tenant, tenant_user, user, user_identity, user_role,
};

/// 默认客户端（管理后台）主键 ID：与 `auth_sys_role.client_id` 的 DDL 默认值一致。
///
/// 各业务角色都不显式带 `client_id`，落在这个客户端下；按 `role_code` 查角色必须限定它，
/// 否则将来其他客户端出现同名 `role_code` 会产生歧义。
const DEFAULT_CLIENT_ID: &str = "1876543210000000300";

/// 小程序端基础角色编码（wx-purchase 客户端）：完善资料时按端补授，
/// 否则小程序会话下菜单为空（该角色提供小程序菜单）。
const MINIAPP_BASE_ROLE_CODE: &str = "wx_user";

/// 注册身份字典编码（服务端写死，注册选项接口不接受任何路径/查询参数）。
///
/// 全端唯一一份身份列表：管理后台注册、门户 `/register`、小程序注册与完善资料页共用它，
/// 因此不再区分「采购身份 / 门户身份」（AI用户 与 采购员/工厂/采购管理员 平级）。
pub const DICT_REGISTER_ROLE: &str = "register_role";

/// Web 扫码 state 的 Redis 键前缀与有效期（秒，一次性消费）
const WX_QR_STATE_PREFIX: &str = "wx_qr_state:";
const WX_QR_STATE_TTL: i64 = 300;

#[derive(Clone, Service)]
pub struct AuthAppService {
    #[inject(component)]
    db: DbConn,
    #[inject(component)]
    redis: Redis,
    #[inject(config)]
    sa_token_config: SaTokenConfig,
}

impl AuthAppService {
    /// 密码（或三方授权）校验通过后的两步验证强制检查：
    /// 启用 TOTP 的账号未携带动态码 → `TwoFactorRequired`；携带则复验
    async fn enforce_two_factor(&self, user_id: &str, totp_code: Option<&str>) -> Result<(), AppError> {
        let Some(hook) = super::two_fa::hook() else {
            return Ok(());
        };
        if !hook.required(user_id).await {
            return Ok(());
        }
        match totp_code.map(str::trim).filter(|c| !c.is_empty()) {
            None => Err(AppError::TwoFactorRequired(
                "@twofa_code_required".to_string(),
            )),
            Some(code) => hook.verify_code(user_id, code).await,
        }
    }

    fn create_refresh_manager(&self) -> RefreshTokenManager {
        let storage: Arc<dyn SaStorage> = Arc::new(
            SummerRedisStorage::new(self.redis.clone())
        );
        RefreshTokenManager::new(storage, Arc::new(CoreConfig::from(self.sa_token_config.clone())))
    }

    /// sa-token 存储键前缀（索引键与 token 键共用，如 `{prefix}login:tokens:{id}`）
    fn storage_prefix(&self) -> &str {
        self.sa_token_config.storage_key_prefix.as_str()
    }

    /// 收敛该账号的 sa-token 残留索引（修复 Redis 无界膨胀）。
    ///
    /// 每次登录都会向 `login:tokens` / `session` / `refresh:user` 三类**永久键**追加条目，
    /// 而它们只在显式登出时才会缩减 —— 这里在登录成功后就地修剪失效条目并按并发上限收敛，
    /// 使这些键的规模始终有界。
    async fn prune_session_index(&self, login_id: &str) {
        let stat = super::session_cleanup::cleanup_user(
            &self.redis,
            self.storage_prefix(),
            login_id,
            self.sa_token_config.max_login_count,
        )
        .await;
        if !stat.is_empty() {
            log::info!(
                "sa-token 会话索引已收敛: login_id={}, 移除条目={}, 删除空键={}",
                login_id,
                stat.removed_entries,
                stat.deleted_keys
            );
        }
    }

    /// 全量清理 sa-token 残留索引（启动执行一次 + 周期兜底）。
    ///
    /// 返回清理统计，供启动/周期任务打日志观测。
    pub async fn sweep_session_index(&self) -> super::session_cleanup::CleanupStat {
        super::session_cleanup::sweep_all(
            &self.redis,
            self.storage_prefix(),
            self.sa_token_config.max_login_count,
        )
        .await
    }

    #[ignore_tenant]
    pub async fn locate_tenant(&self, dto: LocateTenantDto) -> Result<LocateTenantVo, AppError> {
        let index = tenant_user::Entity::find()
            .filter(tenant_user::Column::UserName.eq(&dto.user_name))
            .filter(tenant_user::Column::Status.eq(1))
            .one(&self.db)
            .await?;

        let index = match index {
            Some(i) => i,
            None => {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                return Err(AppError::Unauthorized("@bad_credentials".to_string()));
            }
        };

        let tenant_model = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(&index.tenant_id))
            .filter(tenant::Column::Status.eq(1))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("@bad_credentials".to_string()))?;

        if let Some(expire) = tenant_model.expire_time
            && expire < chrono::Utc::now() {
                return Err(AppError::Unauthorized("@tenant_expired".to_string()));
            }

        Ok(LocateTenantVo {
            tenant_name: tenant_model.tenant_name,
            tenant_code: tenant_model.tenant_code,
            tenant_logo: None,
            user_name: index.user_name,
            login_type: Some(index.identity_type),
        })
    }

    pub async fn login(&self, dto: LoginDto) -> Result<TokenVo, AppError> {
        // 客户端识别（OAuth2 的 client_id/client_secret）：携带 clientCode 时按客户端收敛菜单与功能权限
        let client = self.resolve_login_client(&dto).await?;
        let login_type = dto.login_type.as_deref().unwrap_or("password");

        match login_type {
            "password" => {
                if dto.tenant_code.is_some() {
                    self.login_database_mode(dto, client.as_ref()).await
                } else {
                    self.login_auto_detect(dto, client.as_ref()).await
                }
            }
            "wechat" => {
                // Web 扫码登录：先校验并一次性消费 CSRF state，再走三方登录
                self.consume_qr_state(dto.qr_state.as_deref()).await?;
                self.login_third_party("wechat", dto.code.as_deref(), &dto.state, client.as_ref())
                    .await
            }
            // 微信小程序：code 来自 wx.login()，用 jscode2session 换取 openid
            "wechat_mp" => self.login_third_party("wechat_mp", dto.code.as_deref(), &dto.state, client.as_ref()).await,
            "dingtalk" => self.login_third_party("dingtalk", dto.code.as_deref(), &dto.state, client.as_ref()).await,
            _ => self.login_table_mode(dto, client.as_ref()).await,
        }
    }

    /// 解析登录请求携带的客户端：不存在/已停用/密钥错误直接拒绝登录。
    ///
    /// 客户端注册表是全局表（主库），登录发生在租户上下文之前，查询需忽略租户过滤。
    async fn resolve_login_client(&self, dto: &LoginDto) -> Result<Option<client::Model>, AppError> {
        let Some(code) = dto
            .client_code
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        else {
            return Ok(None);
        };

        let _guard = TenantIgnoreGuard::new();
        let model: client::Model = client::Entity::find()
            .filter(client::Column::ClientCode.eq(code))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("@client_not_found".to_string()))?;

        if model.status != 1 {
            return Err(AppError::Unauthorized("@client_disabled".to_string()));
        }
        if !model.client_secret.is_empty() {
            let provided = dto
                .client_secret
                .as_deref()
                .map(str::trim)
                .unwrap_or("");
            if provided.is_empty() {
                return Err(AppError::BadRequest("@client_secret_required".to_string()));
            }
            if provided != model.client_secret {
                return Err(AppError::Unauthorized("@client_secret_invalid".to_string()));
            }
        }
        Ok(Some(model))
    }

    async fn login_auto_detect(
        &self,
        dto: LoginDto,
        client: Option<&client::Model>,
    ) -> Result<TokenVo, AppError> {
        let user_name = dto.user_name.as_deref().unwrap_or("");
        let _guard = TenantIgnoreGuard::new();

        let index = tenant_user::Entity::find()
            .filter(tenant_user::Column::UserName.eq(user_name))
            .filter(tenant_user::Column::Status.eq(1))
            .one(&self.db)
            .await?;

        if let Some(idx) = index {
            let tenant_model = tenant::Entity::find()
                .filter(tenant::Column::Id.eq(&idx.tenant_id))
                .filter(tenant::Column::Status.eq(1))
                .one(&self.db)
                .await?
                .ok_or_else(|| AppError::Unauthorized("@bad_credentials".to_string()))?;

            if tenant_model.mode == "database" {
                drop(_guard);
                let mut db_dto = dto.clone();
                db_dto.tenant_code = Some(tenant_model.tenant_code.clone());
                return self.login_database_mode(db_dto, client).await;
            }
        }

        drop(_guard);
        self.login_table_mode(dto, client).await
    }

    #[ignore_tenant]
    async fn login_database_mode(
        &self,
        dto: LoginDto,
        client: Option<&client::Model>,
    ) -> Result<TokenVo, AppError> {
        let user_name = dto.user_name.as_deref()
            .ok_or_else(|| AppError::BadRequest("@username_required".to_string()))?;
        let tenant_code = dto.tenant_code.as_deref()
            .ok_or_else(|| AppError::BadRequest("@tenant_code_required".to_string()))?;
        let remember_me = dto.remember_me.unwrap_or(false);

        let tenant_model = tenant::Entity::find()
            .filter(tenant::Column::TenantCode.eq(tenant_code))
            .filter(tenant::Column::Status.eq(1))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("@bad_credentials".to_string()))?;

        if let Some(expire) = tenant_model.expire_time
            && expire < chrono::Utc::now() {
                return Err(AppError::Unauthorized("@bad_credentials".to_string()));
            }

        let tenant_db = self.get_tenant_database(&tenant_model.id).await?;

        let user_model = user::Entity::find()
            .filter(user::Column::UserName.eq(user_name))
            .one(&tenant_db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("@bad_credentials".to_string()))?;

        // 未完善账号（自动注册后 pass_word 为空）不能走密码登录：
        // verify_password 对空哈希会报内部错误，这里显式给出可读提示
        if user_model.pass_word.is_empty() {
            return Err(AppError::Unauthorized("@account_profile_incomplete".to_string()));
        }

        if !common::verify_password(dto.pass_word.as_deref().unwrap_or(""), &user_model.pass_word)? {
            return Err(AppError::Unauthorized("@bad_credentials".to_string()));
        }

        if user_model.status != 1 {
            return Err(AppError::Unauthorized("@account_disabled".to_string()));
        }

        self.do_login(&tenant_model, &user_model, &tenant_db, "database", remember_me, dto.totp_code.as_deref(), client).await
    }

    /// 第三方登录（微信小程序一键登录 / Web 扫码 / 钉钉）：
    /// 按第三方身份表定位账号；未绑定且满足条件时自动注册惰性账号（待完善资料）。
    #[ignore_tenant]
    async fn login_third_party(
        &self,
        provider: &str,
        code: Option<&str>,
        state: &Option<String>,
        client: Option<&client::Model>,
    ) -> Result<TokenVo, AppError> {
        let code = code.ok_or_else(|| AppError::BadRequest("@code_required".to_string()))?;
        let tenant_code = state.as_deref().unwrap_or("");

        // 微信类优先取 unionid（跨端识别依赖它），拿不到时回退 openid
        let identity = resolve_third_party_identity(provider, code).await?;

        let existing = self.find_identity(&identity).await?;

        let (tenant_model, user_id) = match existing {
            Some(row) => {
                let user_id = row.user_id.clone();
                // tenant_user 已收敛为纯「用户-租户索引」：按 user_id 反查租户
                let index = tenant_user::Entity::find()
                    .filter(tenant_user::Column::UserId.eq(&user_id))
                    .filter(tenant_user::Column::Status.eq(1))
                    .one(&self.db)
                    .await?
                    .ok_or_else(|| AppError::Unauthorized("@bad_credentials".to_string()))?;

                let tenant_model = tenant::Entity::find()
                    .filter(tenant::Column::Id.eq(&index.tenant_id))
                    .filter(tenant::Column::Status.eq(1))
                    .one(&self.db)
                    .await?
                    .ok_or_else(|| AppError::Unauthorized("@bad_credentials".to_string()))?;

                if !tenant_code.is_empty() && tenant_model.tenant_code != tenant_code {
                    return Err(AppError::Unauthorized("@bad_credentials".to_string()));
                }
                (tenant_model, user_id)
            }
            None => {
                // 未绑定：仅当「微信类 + 已解析出登录客户端」时才自动建号（安全边界）
                if !can_auto_register(provider, client) {
                    return Err(AppError::Unauthorized("@account_not_bound".to_string()));
                }
                self.auto_register_third_party(provider, &identity).await?
            }
        };

        let tenant_mode = tenant_model.mode.as_str();
        let tenant_db = if tenant_mode == "database" {
            Some(self.get_tenant_database(&tenant_model.id).await?)
        } else {
            None
        };
        let db = tenant_db.as_ref().unwrap_or(&self.db);

        let user_model = user::Entity::find()
            .filter(user::Column::Id.eq(&user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("@bad_credentials".to_string()))?;

        if user_model.status != 1 {
            return Err(AppError::Unauthorized("@account_disabled".to_string()));
        }

        self.do_login(&tenant_model, &user_model, db, tenant_mode, false, None, client)
            .await
    }

    /// 按第三方标识查身份行（详见 [`find_identity_row`]）。
    async fn find_identity(
        &self,
        identity: &ThirdPartyIdentity,
    ) -> Result<Option<user_identity::Model>, AppError> {
        find_identity_row(&self.db, identity).await
    }

    /// 自动注册惰性账号（第三方首登，未绑定任何账号时）。
    ///
    /// - 占位用户名 `wx_<sha256(identity_value) 前 16 hex>`（确定性命名，见 `common::user::placeholder_user_name`）
    /// - `pass_word = ''` 空串 = 未完善标记；**不授任何角色** → 权限为空，
    ///   除 `sa_check_login` 端点外一律 403，只能走「完善账户信息」
    /// - 写两处索引：`tenant_user`（用户-租户）+ `user_identity`（第三方身份）
    ///
    /// 返回 `(默认租户, 新用户 id)`。并发首登时唯一索引会让其中一方失败，
    /// 失败方按第三方标识重查并复用对方建的账号。
    #[ignore_tenant]
    async fn auto_register_third_party(
        &self,
        provider: &str,
        identity: &ThirdPartyIdentity,
    ) -> Result<(tenant::Model, String), AppError> {
        let default_tenant_id = get_tenant_config()
            .and_then(|c| c.default_tenant_id.clone())
            .and_then(|v| match v {
                Value::String(Some(s)) => Some(s),
                _ => None,
            })
            .ok_or_else(|| AppError::Internal("@default_tenant_not_configured".to_string()))?;
        let tenant_model = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(&default_tenant_id))
            .filter(tenant::Column::Status.eq(1))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Internal("@default_tenant_not_found".to_string()))?;

        let placeholder = common::user::placeholder_user_name(&identity.identity_value);

        let result = self
            .insert_lazy_account(
                provider,
                identity,
                &placeholder,
                &tenant_model,
            )
            .await;

        match result {
            Ok(user_id) => Ok((tenant_model, user_id)),
            Err(e) => {
                // 并发首登：另一请求先建成功，按第三方标识重查后复用其账号
                if let Some(row) = self.find_identity(identity).await? {
                    log::info!(
                        "第三方自动注册并发冲突，复用已有账号: provider={}, user_id={}",
                        provider,
                        row.user_id
                    );
                    return Ok((tenant_model, row.user_id));
                }
                log::error!("第三方自动注册失败: provider={}, err={}", provider, e);
                Err(AppError::Internal("@wechat_autoregister_failed".to_string()))
            }
        }
    }

    /// 惰性账号落库（user + tenant_user + user_identity）。
    ///
    /// table 模式下三张表同库，用事务保证原子；database 模式下租户库写 `user`、
    /// 主库写两张索引表，跨库非原子（与既有 `register()` 的限制一致）。
    #[ignore_tenant]
    async fn insert_lazy_account(
        &self,
        provider: &str,
        identity: &ThirdPartyIdentity,
        placeholder: &str,
        tenant_model: &tenant::Model,
    ) -> Result<String, AppError> {
        let tenant_db = if tenant_model.mode == "database" {
            Some(self.get_tenant_database(&tenant_model.id).await?)
        } else {
            None
        };

        let user_model = user::ActiveModel {
            user_name: Set(placeholder.to_string()),
            // 空密码 = 未完善标记（前端据此强制跳完善页；密码登录也被显式拦截）
            pass_word: Set(String::new()),
            nick_name: Set(None),
            email: Set(None),
            phone: Set(None),
            identity_type: Set(Some(provider.to_string())),
            identity_value: Set(Some(identity.identity_value.clone())),
            status: Set(1),
            admin_flag: Set(0),
            dept_id: Set(None),
            tenant_id: Set(Some(tenant_model.id.clone())),
            ..Default::default()
        };

        let user_id = if let Some(tenant_db) = &tenant_db {
            // database 模式：租户库写 user（跨库，无法与主库索引同事务）
            let tx = tenant_db.inner().begin().await?;
            let created = user_model.insert(&tx).await?;
            tx.commit().await?;
            created.id
        } else {
            // table 模式：user + tenant_user + user_identity 同一事务
            let tx = self.db.inner().begin().await?;
            let created = user_model.insert(&tx).await?;
            tenant_user::ActiveModel {
                user_name: Set(placeholder.to_string()),
                tenant_id: Set(tenant_model.id.clone()),
                tenant_code: Set(tenant_model.tenant_code.clone()),
                user_id: Set(created.id.clone()),
                identity_type: Set("username".to_string()),
                identity_value: Set(None),
                status: Set(1),
                ..Default::default()
            }
            .insert(&tx)
            .await?;
            user_identity::ActiveModel {
                user_id: Set(created.id.clone()),
                provider: Set(provider.to_string()),
                identity_value: Set(identity.identity_value.clone()),
                nick_name: Set(None),
                status: Set(1),
                tenant_id: Set(Some(tenant_model.id.clone())),
                ..Default::default()
            }
            .insert(&tx)
            .await?;
            tx.commit().await?;
            created.id
        };

        // database 模式：两张索引表在主库
        if tenant_db.is_some() {
            tenant_user::ActiveModel {
                user_name: Set(placeholder.to_string()),
                tenant_id: Set(tenant_model.id.clone()),
                tenant_code: Set(tenant_model.tenant_code.clone()),
                user_id: Set(user_id.clone()),
                identity_type: Set("username".to_string()),
                identity_value: Set(None),
                status: Set(1),
                ..Default::default()
            }
            .insert(&self.db)
            .await?;
            user_identity::ActiveModel {
                user_id: Set(user_id.clone()),
                provider: Set(provider.to_string()),
                identity_value: Set(identity.identity_value.clone()),
                nick_name: Set(None),
                status: Set(1),
                tenant_id: Set(Some(tenant_model.id.clone())),
                ..Default::default()
            }
            .insert(&self.db)
            .await?;
        }

        Ok(user_id)
    }

    #[ignore_tenant]
    async fn login_table_mode(
        &self,
        dto: LoginDto,
        client: Option<&client::Model>,
    ) -> Result<TokenVo, AppError> {
        let login_type = dto.login_type.as_deref().unwrap_or("username");
        let remember_me = dto.remember_me.unwrap_or(false);
        // 邮箱验证码登录：以验证码替代密码验证，密码字段可留空
        let is_email_code = login_type == "email_code";

        let user_model = match login_type {
            "email" => {
                let email = dto.email.as_deref().ok_or_else(|| {
                    AppError::BadRequest("@email_required".to_string())
                })?;
                user::Entity::find()
                    .filter(user::Column::Email.eq(email))
                    .one(&self.db)
                    .await?
                    .ok_or_else(|| AppError::Unauthorized("@bad_email_credentials".to_string()))?
            }
            "email_code" => {
                let email = dto.email.as_deref().ok_or_else(|| {
                    AppError::BadRequest("@email_required".to_string())
                })?;
                let code = dto.email_code.as_deref().ok_or_else(|| {
                    AppError::BadRequest("@email_code_required".to_string())
                })?;
                // 先校验并一次性消费验证码，再按邮箱取用户（不区分该邮箱是否注册，防枚举）
                self.verify_email_code(email, "login", code).await?;
                user::Entity::find()
                    .filter(user::Column::Email.eq(email))
                    .one(&self.db)
                    .await?
                    .ok_or_else(|| AppError::Unauthorized("@email_code_invalid".to_string()))?
            }
            _ => {
                let user_name = dto.user_name.as_deref().unwrap_or("");
                user::Entity::find()
                    .filter(user::Column::UserName.eq(user_name))
                    .one(&self.db)
                    .await?
                    .ok_or_else(|| AppError::Unauthorized("@bad_credentials".to_string()))?
            }
        };

        // 未完善账号（自动注册后 pass_word 为空）不能走密码登录：
        // verify_password 对空哈希会报内部错误，这里显式给出可读提示
        if !is_email_code && user_model.pass_word.is_empty() {
            return Err(AppError::Unauthorized("@account_profile_incomplete".to_string()));
        }

        if !is_email_code
            && !common::verify_password(
                dto.pass_word.as_deref().unwrap_or(""),
                &user_model.pass_word,
            )?
        {
            return Err(AppError::Unauthorized("@wrong_password".to_string()));
        }

        if user_model.status != 1 {
            return Err(AppError::Unauthorized("@account_disabled".to_string()));
        }

        self.enforce_two_factor(&user_model.id.to_string(), dto.totp_code.as_deref()).await?;

        let user_id_str = user_model.id.to_string();
        let client_id = client.map(|c| c.id.as_str());
        let grant = {
            let query =
                super::permission_sync::load_user_grant(&self.db, &user_id_str, client_id);

            if let Some(tid) = &user_model.tenant_id {
                let tid_value = Value::String(Some(tid.clone()));
                tenant_scope(tid_value, query).await?
            } else {
                query.await?
            }
        };
        let (role_codes, permissions) = (grant.role_codes, grant.permissions);

        // 客户端登录限制：非超管必须在该客户端下拥有角色，否则无权从该客户端登录。
        // 例外刻意收窄：待完善账号（自动注册后 pass_word 为空）放行 —— 它零角色、权限为空，
        // 除 sa_check_login 端点外一律 403，只能走「完善账户信息」，不存在越权风险。
        // 不要写成"无角色即放行"。
        if client.is_some() && user_model.admin_flag != 1 && role_codes.is_empty() && !user_model.pass_word.is_empty() {
            return Err(AppError::Unauthorized("@client_login_denied".to_string()));
        }

        let login_id = common::user::build_scoped_login_id(
            &user_id_str,
            client.map(|c| c.client_code.as_str()),
        );
        let extra = client_extra(
            serde_json::json!({
                "userId": user_id_str,
                "userName": user_model.user_name,
                "nickName": user_model.nick_name,
                "tenantId": user_model.tenant_id,
                "tenantMode": "table",
            }),
            client,
        );

        let token_value = StpUtil::login_with_extra(&login_id, extra.clone()).await.map_err(|e| {
            AppError::Internal(format!("@login_failed:{}", e))
        })?;

        StpUtil::set_roles(&login_id, role_codes).await.ok();
        StpUtil::set_permissions(&login_id, permissions).await.ok();

        let token = token_value.as_str().to_string();
        let token_info = StpUtil::get_token_info(&token_value).await
            .map_err(|e| AppError::Internal(format!("@token_info_failed:{}", e)))?;
        let expire_time = token_info.expire_time.map(|t| t.timestamp());

        // 复用 sa-token 在登录时已生成并落库的 refresh token：库在 `login_with_extra` 内
        // 已写入 `refresh:{rt}` 记录与 `refresh:user` 索引；若此处再 generate 一次，每次登录
        // 会产生 2 条 refresh 记录，其中库的那条永不返回给客户端（纯垃圾），使 refresh 键翻倍膨胀。
        let (refresh_token, refresh_expire_time) = if remember_me {
            let rt = match token_info.refresh_token.clone() {
                Some(existing) => existing,
                None => {
                    // 库未生成时（如未开启 refresh token）回退为自行生成，保持原行为
                    let refresh_mgr = self.create_refresh_manager();
                    // login_id 已按客户端隔离，refresh 索引必须与之一致，否则登出/刷新找不到记录
                    let generated = refresh_mgr.generate(&login_id);
                    refresh_mgr.store_with_extra(&generated, &token, &login_id, Some(&extra)).await
                        .map_err(|e| AppError::Internal(format!("@refresh_token_failed:{}", e)))?;
                    generated
                }
            };

            let refresh_exp = if self.sa_token_config.refresh_token_timeout > 0 {
                Some(chrono::Utc::now().timestamp() + self.sa_token_config.refresh_token_timeout)
            } else {
                None
            };

            (Some(rt), refresh_exp)
        } else {
            (None, None)
        };

        // 登录成功后收敛该账号的 sa-token 残留索引（修复 Redis 无界膨胀）
        self.prune_session_index(&login_id).await;

        Ok(TokenVo {
            token,
            token_name: self.sa_token_config.token_name.clone(),
            token_prefix: "Bearer ".to_string(),
            refresh_token,
            expire_time,
            refresh_expire_time,
        })
    }

    /// 校验并一次性消费邮箱验证码：与 relay 门户发送端共享 Redis key（`mail:code:{email}:{scene}`）。
    ///
    /// `scene` 与发码端白名单一致：
    /// - `login` —— 邮箱验证码登录
    /// - `register` —— 注册 / 完善账户信息时的邮箱所有权验证
    /// - `change_password` —— 修改密码（验证码发到账号**当前**邮箱）
    /// - `change_email` —— 更换绑定邮箱（验证码发到**新**邮箱）
    async fn verify_email_code(&self, email: &str, scene: &str, code: &str) -> Result<(), AppError> {
        if email.trim().is_empty() || code.trim().is_empty() {
            return Err(AppError::BadRequest("@email_or_code_required".to_string()));
        }
        let mut conn = self.redis.clone();
        let key = common::email_code_redis_key(email, scene);
        let saved: Option<String> = summer_redis::redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|_| AppError::Internal("@captcha_service_unavailable".to_string()))?;
        let matched = saved
            .as_deref()
            .map(|s| s == code.trim())
            .unwrap_or(false);
        // 一次性消费：无论对错均删除，避免在同一验证码上爆破重试
        let _ = summer_redis::redis::cmd("DEL").arg(&key).query_async::<i64>(&mut conn).await;
        if matched {
            Ok(())
        } else {
            Err(AppError::Unauthorized("@email_code_invalid".to_string()))
        }
    }

    /// 注册 / 完善资料时的邮箱必填校验：格式 → 查重 → 一次性消费验证码。
    ///
    /// `exclude_user_id`：完善资料时排除自己（注册时传 None）。
    /// 顺序刻意把**验证码校验放最后**：格式或查重失败时不消耗用户刚收到的验证码。
    async fn assert_email_verified(
        &self,
        email: &str,
        email_code: &str,
        exclude_user_id: Option<&str>,
    ) -> Result<(), AppError> {
        let email = email.trim();
        if email.is_empty() {
            return Err(AppError::BadRequest("@email_required".to_string()));
        }
        // 与发码端（relay send_code）同一口径
        if !email.contains('@') || email.len() > 100 {
            return Err(AppError::BadRequest("@email_invalid".to_string()));
        }
        if email_code.trim().is_empty() {
            return Err(AppError::BadRequest("@email_code_required".to_string()));
        }

        // 邮箱查重：auth_sys_user.email 无唯一索引，唯一性由应用层保证（与门户注册一致）
        let mut query = user::Entity::find().filter(user::Column::Email.eq(email));
        if let Some(uid) = exclude_user_id {
            query = query.filter(user::Column::Id.ne(uid));
        }
        if query.one(&self.db).await?.is_some() {
            return Err(AppError::BadRequest("@email_taken".to_string()));
        }

        self.verify_email_code(email, "register", email_code).await
    }

    async fn do_login(
        &self,
        tenant_model: &tenant::Model,
        user_model: &user::Model,
        db: &DbConn,
        tenant_mode: &str,
        remember_me: bool,
        totp_code: Option<&str>,
        client: Option<&client::Model>,
    ) -> Result<TokenVo, AppError> {
        self.enforce_two_factor(&user_model.id.to_string(), totp_code).await?;

        let user_id_str = user_model.id.to_string();
        let client_id = client.map(|c| c.id.as_str());
        // database 模式下 db 已是租户库，无需再叠加 tenant_scope
        let grant = super::permission_sync::load_user_grant(db, &user_id_str, client_id).await?;
        let (role_codes, permissions) = (grant.role_codes, grant.permissions);

        // 客户端登录限制：非超管必须在该客户端下拥有角色，否则无权从该客户端登录。
        // 例外刻意收窄：待完善账号（自动注册后 pass_word 为空）放行 —— 它零角色、权限为空，
        // 除 sa_check_login 端点外一律 403，只能走「完善账户信息」，不存在越权风险。
        if client.is_some() && user_model.admin_flag != 1 && role_codes.is_empty() && !user_model.pass_word.is_empty() {
            return Err(AppError::Unauthorized("@client_login_denied".to_string()));
        }

        let login_id = common::user::build_scoped_login_id(
            &user_id_str,
            client.map(|c| c.client_code.as_str()),
        );
        let extra = client_extra(
            serde_json::json!({
                "userId": user_id_str,
                "userName": user_model.user_name,
                "nickName": user_model.nick_name,
                "tenantId": tenant_model.id.clone(),
                "tenantCode": tenant_model.tenant_code.clone(),
                "tenantMode": tenant_mode,
            }),
            client,
        );

        let token_value = StpUtil::login_with_extra(&login_id, extra.clone()).await
            .map_err(|e| AppError::Internal(format!("@login_failed:{}", e)))?;

        StpUtil::set_roles(&login_id, role_codes).await.ok();
        StpUtil::set_permissions(&login_id, permissions).await.ok();

        let token = token_value.as_str().to_string();
        let token_info = StpUtil::get_token_info(&token_value).await
            .map_err(|e| AppError::Internal(format!("@token_info_failed:{}", e)))?;
        let expire_time = token_info.expire_time.map(|t| t.timestamp());

        // 复用 sa-token 在登录时已生成并落库的 refresh token：库在 `login_with_extra` 内
        // 已写入 `refresh:{rt}` 记录与 `refresh:user` 索引；若此处再 generate 一次，每次登录
        // 会产生 2 条 refresh 记录，其中库的那条永不返回给客户端（纯垃圾），使 refresh 键翻倍膨胀。
        let (refresh_token, refresh_expire_time) = if remember_me {
            let rt = match token_info.refresh_token.clone() {
                Some(existing) => existing,
                None => {
                    // 库未生成时（如未开启 refresh token）回退为自行生成，保持原行为
                    let refresh_mgr = self.create_refresh_manager();
                    let generated = refresh_mgr.generate(&login_id);
                    refresh_mgr.store_with_extra(&generated, &token, &login_id, Some(&extra)).await
                        .map_err(|e| AppError::Internal(format!("@refresh_token_failed:{}", e)))?;
                    generated
                }
            };

            let refresh_exp = if self.sa_token_config.refresh_token_timeout > 0 {
                Some(chrono::Utc::now().timestamp() + self.sa_token_config.refresh_token_timeout)
            } else {
                None
            };

            (Some(rt), refresh_exp)
        } else {
            (None, None)
        };

        // 登录成功后收敛该账号的 sa-token 残留索引（修复 Redis 无界膨胀）
        self.prune_session_index(&login_id).await;

        Ok(TokenVo {
            token,
            token_name: self.sa_token_config.token_name.clone(),
            token_prefix: "Bearer ".to_string(),
            refresh_token,
            expire_time,
            refresh_expire_time,
        })
    }

    async fn get_tenant_database(&self, tenant_id: &str) -> Result<DbConn, AppError> {
        // 未登录场景（登录/注册）下 provider 返回 None，自动路由不生效，
        // 需要手动按 tenant_id 选库。使用 unchecked 版本跳过 mode 检查
        // （因为全局配置可能是 table，但该 tenant 实际是 database 模式）。
        let tid_value = Value::String(Some(tenant_id.to_string()));
        let db = sea_orm_ext::get_database_for_tenant_unchecked(&tid_value)?
            .unwrap_or_else(|| self.db.inner().clone());
        Ok(DbConn::new(db))
    }

    /// 完善账户信息（第三方自动注册后的惰性账号首次进入系统前必填）。
    ///
    /// 校验：账号尚未完善 / 用户名全局唯一 / 手机号格式 / role_code 在启用字典项内且角色存在。
    /// 单事务：更新 `user` + 授端基础角色（小程序端补 `wx_user`）+ 授所选身份角色 + 更新 `tenant_user.user_name`。
    pub async fn complete_profile(&self, dto: CompleteProfileDto) -> Result<UserInfoVo, AppError> {
        let login_id = StpUtil::get_login_id_as_string()
            .await
            .map_err(|e| AppError::Unauthorized(format!("@not_logged_in:{}", e)))?;
        let user_id = common::user::base_user_id_from_login_id(&login_id);

        let user_model = user::Entity::find()
            .filter(user::Column::Id.eq(&user_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@user_not_found".to_string()))?;

        // 未完善判定：pass_word 为空串（自动注册时写入）
        if !user_model.pass_word.is_empty() {
            return Err(AppError::BadRequest("@profile_already_completed".to_string()));
        }

        let user_name = dto.user_name.trim().to_string();
        if !(3..=50).contains(&user_name.chars().count())
            || !user_name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return Err(AppError::BadRequest("@username_invalid".to_string()));
        }
        let pass_word = dto.pass_word.trim().to_string();
        if !(8..=64).contains(&pass_word.chars().count()) {
            return Err(AppError::BadRequest("@password_invalid".to_string()));
        }
        let phone = dto.phone.trim().to_string();
        if !is_valid_cn_mobile(&phone) {
            return Err(AppError::BadRequest("@phone_invalid".to_string()));
        }
        let nick_name = dto
            .nick_name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        if let Some(n) = &nick_name
            && n.chars().count() > 30
        {
            return Err(AppError::BadRequest("@nick_name_too_long".to_string()));
        }

        // 身份由字典驱动：绝不接受前端传 role_id
        let role_id = resolve_selectable_role(&self.db, &dto.role_code).await?;

        // 邮箱必填：格式 → 查重（排除自己）→ 一次性消费验证码
        self.assert_email_verified(&dto.email, &dto.email_code, Some(&user_id))
            .await?;
        let email = dto.email.trim().to_string();

        // 用户名查重：auth_sys_user.user_name 是全局 NOT NULL UNIQUE
        let taken = user::Entity::find()
            .filter(user::Column::UserName.eq(&user_name))
            .filter(user::Column::Id.ne(&user_id))
            .one(&self.db)
            .await?;
        if taken.is_some() {
            return Err(AppError::BadRequest("@username_taken".to_string()));
        }
        // tenant_user.user_name 上也有唯一索引（主库表，需忽略租户路由与过滤）
        {
            let _main_db_guard = TenantIgnoreGuard::new();
            let idx_taken = tenant_user::Entity::find()
                .filter(tenant_user::Column::UserName.eq(&user_name))
                .filter(tenant_user::Column::UserId.ne(&user_id))
                .one(&self.db)
                .await?;
            if idx_taken.is_some() {
                return Err(AppError::BadRequest("@username_taken".to_string()));
            }
        }

        // 端基础角色：小程序会话补 wx_user（仅作「小程序端用户」标记，**不携带菜单**），Web 端不补。
        // 小程序端可见功能由「身份角色」按 wx-purchase 客户端菜单收敛
        // （见 migration 的 initial.sql / purchase.sql / expense.sql 菜单授权段）
        let miniapp_role_id = self.miniapp_base_role_id().await?;

        let tenant_id = user_model.tenant_id.clone();
        let mut role_ids = vec![role_id];
        if let Some(rid) = miniapp_role_id {
            role_ids.push(rid);
        }

        // 已授角色去重（避免重复插入触发唯一冲突）
        let existing_role_ids: Vec<String> = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(&user_id))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|r| r.role_id)
            .collect();

        let tx = self.db.inner().begin().await?;

        let mut active: user::ActiveModel = user_model.clone().into();
        active.user_name = Set(user_name.clone());
        active.pass_word = Set(common::hash_password(&pass_word)?);
        active.phone = Set(Some(phone));
        active.email = Set(Some(email));
        active.nick_name = Set(nick_name);
        active.update(&tx).await?;

        for rid in role_ids {
            if existing_role_ids.contains(&rid) {
                continue;
            }
            user_role::ActiveModel {
                user_id: Set(user_id.clone()),
                role_id: Set(rid),
                tenant_id: Set(tenant_id.clone()),
                ..Default::default()
            }
            .insert(&tx)
            .await?;
        }
        tx.commit().await?;

        // 同步租户用户索引中的用户名：该表在**主库**，database 模式下与租户库写入
        // 无法同事务（与 register() 的既有约束一致），故放在提交后、走主库执行
        {
            let _main_db_guard = TenantIgnoreGuard::new();
            tenant_user::Entity::update_many()
                .col_expr(
                    tenant_user::Column::UserName,
                    sea_orm::sea_query::Expr::value(user_name.clone()),
                )
                .filter(tenant_user::Column::UserId.eq(&user_id))
                .exec(&self.db)
                .await?;
        }

        // 角色在提交后才产生，当前会话快照还是空的：立即重算并广播
        super::permission_sync::sync_and_notify(&self.db, &self.redis, &[user_id]).await;

        self.get_user_info().await
    }

    /// 修改个人信息（昵称 / 手机号）：只改当前登录人自己的记录。
    ///
    /// 用户名是登录凭据（全局唯一）不可改；邮箱与密码各有独立接口，均需邮箱验证码认证。
    pub async fn update_profile(&self, dto: UpdateProfileDto) -> Result<UserInfoVo, AppError> {
        let login_id = StpUtil::get_login_id_as_string()
            .await
            .map_err(|e| AppError::Unauthorized(format!("@not_logged_in:{}", e)))?;
        let user_id = common::user::base_user_id_from_login_id(&login_id);

        let user_model = user::Entity::find()
            .filter(user::Column::Id.eq(&user_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@user_not_found".to_string()))?;

        // 昵称：可选，最长 30 字符；空串视为清空
        let nick_name = dto
            .nick_name
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        if let Some(n) = &nick_name
            && n.chars().count() > 30
        {
            return Err(AppError::BadRequest("@nick_name_too_long".to_string()));
        }

        // 手机号：可选；填写须为合法手机号，空串表示清空
        let phone_raw = dto.phone.as_deref().map(str::trim).unwrap_or("");
        let phone = if phone_raw.is_empty() {
            None
        } else {
            if !is_valid_cn_mobile(phone_raw) {
                return Err(AppError::BadRequest("@phone_invalid".to_string()));
            }
            Some(phone_raw.to_string())
        };

        let mut active: user::ActiveModel = user_model.into();
        active.nick_name = Set(nick_name);
        active.phone = Set(phone);
        active.update(&self.db).await?;

        self.get_user_info().await
    }

    /// 修改密码：认证方式是账号**当前**绑定邮箱收到的一次性验证码（scene=change_password）。
    ///
    /// 邮箱取自登录人自己的记录（不接受前端传参），避免用他人邮箱的验证码改自己密码。
    pub async fn change_password(&self, dto: ChangePasswordDto) -> Result<(), AppError> {
        let login_id = StpUtil::get_login_id_as_string()
            .await
            .map_err(|e| AppError::Unauthorized(format!("@not_logged_in:{}", e)))?;
        let user_id = common::user::base_user_id_from_login_id(&login_id);

        let pass_word = dto.new_password.trim().to_string();
        if !(8..=64).contains(&pass_word.chars().count()) {
            return Err(AppError::BadRequest("@password_invalid".to_string()));
        }

        let user_model = user::Entity::find()
            .filter(user::Column::Id.eq(&user_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@user_not_found".to_string()))?;

        let email = user_model.email.clone().unwrap_or_default();
        if email.trim().is_empty() {
            return Err(AppError::BadRequest("@email_not_bound".to_string()));
        }
        self.verify_email_code(&email, "change_password", &dto.email_code)
            .await?;

        let mut active: user::ActiveModel = user_model.into();
        active.pass_word = Set(common::hash_password(&pass_word)?);
        active.update(&self.db).await?;
        Ok(())
    }

    /// 更换绑定邮箱：认证方式是**新**邮箱收到的一次性验证码（scene=change_email）。
    ///
    /// 顺序刻意把验证码校验放最后（格式/查重失败不消耗用户刚收到的验证码），与注册同一口径；
    /// 落库同时同步 `identity_value`（与注册写法一致）与主库 `tenant_user`。
    pub async fn change_email(&self, dto: ChangeEmailDto) -> Result<UserInfoVo, AppError> {
        let login_id = StpUtil::get_login_id_as_string()
            .await
            .map_err(|e| AppError::Unauthorized(format!("@not_logged_in:{}", e)))?;
        let user_id = common::user::base_user_id_from_login_id(&login_id);

        let new_email = dto.new_email.trim().to_string();
        if new_email.is_empty() {
            return Err(AppError::BadRequest("@email_required".to_string()));
        }
        if !new_email.contains('@') || new_email.len() > 100 {
            return Err(AppError::BadRequest("@email_invalid".to_string()));
        }
        if dto.email_code.trim().is_empty() {
            return Err(AppError::BadRequest("@email_code_required".to_string()));
        }

        let user_model = user::Entity::find()
            .filter(user::Column::Id.eq(&user_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@user_not_found".to_string()))?;

        if user_model.email.as_deref() == Some(new_email.as_str()) {
            return Err(AppError::BadRequest("@email_unchanged".to_string()));
        }

        // 查重（排除自己）：auth_sys_user.email 无唯一索引，唯一性由应用层保证
        let taken = user::Entity::find()
            .filter(user::Column::Email.eq(&new_email))
            .filter(user::Column::Id.ne(&user_id))
            .one(&self.db)
            .await?;
        if taken.is_some() {
            return Err(AppError::BadRequest("@email_taken".to_string()));
        }

        self.verify_email_code(&new_email, "change_email", &dto.email_code)
            .await?;

        let tx = self.db.inner().begin().await?;
        let mut active: user::ActiveModel = user_model.into();
        active.email = Set(Some(new_email.clone()));
        active.identity_value = Set(Some(new_email.clone()));
        active.update(&tx).await?;
        tx.commit().await?;

        // tenant_user 在主库：database 模式下与租户库写入无法同事务，提交后走主库同步
        {
            let _main_db_guard = TenantIgnoreGuard::new();
            tenant_user::Entity::update_many()
                .col_expr(
                    tenant_user::Column::IdentityValue,
                    sea_orm::sea_query::Expr::value(new_email.clone()),
                )
                .filter(tenant_user::Column::UserId.eq(&user_id))
                .exec(&self.db)
                .await?;
        }

        self.get_user_info().await
    }

    /// 当前会话若来自小程序客户端，返回该客户端的 `wx_user` 角色 ID（否则 None）。
    ///
    /// `wx_user` 是随 `initial.sql` 种在**主库**的客户端级角色（wx-purchase 客户端），
    /// 因此客户端与角色两次查询都在主库进行。
    /// 该角色**不绑定任何菜单**，仅标记「小程序端用户」：小程序端可见功能由身份角色逐条授予，
    /// 避免所有身份（如采购员）被基础角色带上记账/审核等越权入口。
    async fn miniapp_base_role_id(&self) -> Result<Option<String>, AppError> {
        let Some(client_code) = common::user::get_current_client_code() else {
            return Ok(None);
        };
        let _main_db_guard = TenantIgnoreGuard::new();
        let Some(client_model) = client::Entity::find()
            .filter(client::Column::ClientCode.eq(&client_code))
            .one(&self.db)
            .await?
        else {
            return Ok(None);
        };
        if client_model.client_type != "miniapp" {
            return Ok(None);
        }
        let role_model = role::Entity::find()
            .filter(role::Column::ClientId.eq(&client_model.id))
            .filter(role::Column::RoleCode.eq(MINIAPP_BASE_ROLE_CODE))
            .one(&self.db)
            .await?;
        if role_model.is_none() {
            log::warn!(
                "小程序客户端缺少基础角色 {}: client_id={}",
                MINIAPP_BASE_ROLE_CODE,
                client_model.id
            );
        }
        Ok(role_model.map(|r| r.id))
    }

    /// 校验并一次性消费 Web 扫码的 CSRF state（`GET /api/auth/wechat/qr-url` 生成）。
    pub(super) async fn consume_qr_state(&self, qr_state: Option<&str>) -> Result<(), AppError> {
        consume_qr_state(&self.redis, qr_state).await
    }

    /// 生成 Web 扫码登录二维码地址（微信开放平台「网站应用」qrconnect）。
    ///
    /// state 存 Redis（300s，一次性消费），供回调时校验 CSRF。
    pub async fn wechat_qr_url(&self) -> Result<WechatQrVo, AppError> {
        let appid = std::env::var("WECHAT_APPID").unwrap_or_default();
        let secret = std::env::var("WECHAT_SECRET").unwrap_or_default();
        let redirect_uri = std::env::var("WECHAT_OPEN_REDIRECT_URI").unwrap_or_default();
        if appid.trim().is_empty() || secret.trim().is_empty() || redirect_uri.trim().is_empty() {
            return Err(AppError::BadRequest(
                "@wechat_open_not_configured".to_string(),
            ));
        }

        // 复用已有 CSPRNG 工具生成随机串后再哈希成 hex（不新增依赖）
        let state = common::crypto::sha256_hex(&common::crypto::generate_client_secret());
        let mut conn = self.redis.clone();
        let _: Option<String> = summer_redis::redis::cmd("SET")
            .arg(format!("{WX_QR_STATE_PREFIX}{state}"))
            .arg(&state)
            .arg("EX")
            .arg(WX_QR_STATE_TTL)
            .query_async(&mut conn)
            .await
            .map_err(|_| AppError::Internal("@captcha_service_unavailable".to_string()))?;

        let url = format!(
            "https://open.weixin.qq.com/connect/qrconnect?appid={}&redirect_uri={}&response_type=code&scope=snsapi_login&state={}&self_redirect=true#wechat_redirect",
            appid,
            urlencode(redirect_uri.trim()),
            state
        );
        Ok(WechatQrVo { url, state })
    }

    /// 注册身份选项（免登录公开接口；全端共用同一份列表）。
    ///
    /// `#[ignore_tenant]`：免登录请求没有租户上下文，若走表隔离过滤会追加
    /// `WHERE tenant_id = NULL` 而恒为空集；字典与角色本就是全端共用的全局配置，直接放行。
    #[ignore_tenant]
    pub async fn register_roles(&self) -> Result<Vec<RegisterRoleVo>, AppError> {
        self.selectable_roles().await
    }

    /// 查询可选身份：只返回「启用字典项」且「能对上默认客户端角色」的交集（INNER JOIN 语义）。
    async fn selectable_roles(&self) -> Result<Vec<RegisterRoleVo>, AppError> {
        let type_ids: Vec<String> = dict_type::Entity::find()
            .filter(dict_type::Column::DictType.eq(DICT_REGISTER_ROLE))
            .filter(dict_type::Column::Status.eq(1))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|t| t.id)
            .collect();
        if type_ids.is_empty() {
            return Ok(Vec::new());
        }

        let items = dict_item::Entity::find()
            .filter(dict_item::Column::DictTypeId.is_in(type_ids))
            .filter(dict_item::Column::Status.eq(1))
            .order_by_asc(dict_item::Column::SortOrder)
            .all(&self.db)
            .await?;
        if items.is_empty() {
            return Ok(Vec::new());
        }

        // 与角色表内连接：字典里配了但没有对应角色的编码不暴露给用户
        let values: Vec<String> = items.iter().map(|i| i.dict_value.clone()).collect();
        let existing: Vec<String> = role::Entity::find()
            .filter(role::Column::ClientId.eq(DEFAULT_CLIENT_ID))
            .filter(role::Column::Status.eq(1))
            .filter(role::Column::RoleCode.is_in(values))
            .all(&self.db)
            .await?
            .into_iter()
            .map(|r| r.role_code)
            .collect();

        Ok(items
            .into_iter()
            .filter(|i| existing.contains(&i.dict_value))
            .map(|i| RegisterRoleVo {
                label: i.dict_label,
                value: i.dict_value,
            })
            .collect())
    }

    pub async fn refresh_token(&self, refresh_token_str: &str) -> Result<TokenVo, AppError> {
        let refresh_mgr = self.create_refresh_manager();

        let (new_token_value, _login_id) = refresh_mgr.refresh_access_token(refresh_token_str).await
            .map_err(|e| AppError::Unauthorized(format!("@refresh_token_invalid:{}", e)))?;

        let token_info = StpUtil::get_token_info(&new_token_value).await
            .map_err(|e| AppError::Internal(format!("@token_info_failed:{}", e)))?;

        let expire_time = token_info.expire_time.map(|t| t.timestamp());
        let refresh_expire_time = if self.sa_token_config.refresh_token_timeout > 0 {
            Some(chrono::Utc::now().timestamp() + self.sa_token_config.refresh_token_timeout)
        } else {
            None
        };

        Ok(TokenVo {
            token: new_token_value.as_str().to_string(),
            token_name: self.sa_token_config.token_name.clone(),
            token_prefix: "Bearer ".to_string(),
            refresh_token: Some(refresh_token_str.to_string()),
            expire_time,
            refresh_expire_time,
        })
    }

    #[ignore_tenant]
    pub async fn register(&self, dto: RegisterDto) -> Result<UserInfoVo, AppError> {
        if let Some(tenant_code) = &dto.tenant_code {
            let tenant_model = tenant::Entity::find()
                .filter(tenant::Column::TenantCode.eq(tenant_code))
                .filter(tenant::Column::Status.eq(1))
                .one(&self.db)
                .await?
                .ok_or_else(|| AppError::BadRequest("@tenant_not_found".to_string()))?;

            if tenant_model.mode == "database" {
                let tenant_db = self.get_tenant_database(&tenant_model.id).await?;

                let existing = user::Entity::find()
                    .filter(user::Column::UserName.eq(&dto.user_name))
                    .one(&tenant_db)
                    .await?;

                if existing.is_some() {
                    return Err(AppError::BadRequest("@username_taken".to_string()));
                }

                // 邮箱必填：格式 → 查重 → 一次性消费验证码（通过后账号即带着已验证的邮箱）
                self.assert_email_verified(&dto.email, &dto.email_code, None).await?;

                // 身份由下拉选择决定（字典驱动）：绝不接受前端传 role_id
                let role_id = resolve_selectable_role(&tenant_db, &dto.role_code).await?;

                let user_model = user::ActiveModel {
                    user_name: Set(dto.user_name.clone()),
                    pass_word: Set(common::hash_password(&dto.pass_word)?),
                    nick_name: Set(dto.nick_name.clone()),
                    email: Set(Some(dto.email.clone())),
                    phone: Set(dto.phone.clone()),
                    identity_type: Set(Some("username".to_string())),
                    identity_value: Set(Some(dto.email.clone())),
                    status: Set(1),
                    admin_flag: Set(0),
                    dept_id: Set(None),
                    ..Default::default()
                };
                // 用户 + 所选身份角色同一事务（同一租户库）：避免注册中途失败留下无角色账号
                let tx = tenant_db.inner().begin().await?;
                let result = user_model.insert(&tx).await?;

                user_role::ActiveModel {
                    user_id: Set(result.id.clone()),
                    role_id: Set(role_id),
                    tenant_id: Set(Some(tenant_model.id.clone())),
                    ..Default::default()
                }
                .insert(&tx)
                .await?;
                tx.commit().await?;

                // 租户用户索引在主库（跨库写入，无法与上面同事务）
                tenant_user::ActiveModel {
                    user_name: Set(result.user_name.clone()),
                    tenant_id: Set(tenant_model.id.clone()),
                    tenant_code: Set(tenant_model.tenant_code.clone()),
                    user_id: Set(result.id.clone()),
                    identity_type: Set("username".to_string()),
                    identity_value: Set(Some(dto.email.clone())),
                    status: Set(1),
                    ..Default::default()
                }.insert(&self.db).await?;

                return Ok(UserInfoVo {
                    id: result.id,
                    user_name: result.user_name,
                    nick_name: result.nick_name,
                    email: result.email,
                    phone: result.phone,
                    avatar: result.avatar,
                    roles: vec![],
                    permissions: vec![],
                    menus: vec![],
                    tenant_id: Some(tenant_model.id),
                    tenant_code: Some(tenant_model.tenant_code),
                    tenant_name: Some(tenant_model.tenant_name),
                    profile_completed: true,
                });
            }
        }

        // table 模式注册：根据 tenant_code 或默认配置填充 tenant_id
        // 由于外层 #[ignore_tenant] 跳过了框架自动填充，需手动 Set
        let (tenant_id, tenant_code, tenant_name) = if let Some(code) = &dto.tenant_code {
            let tenant_model = tenant::Entity::find()
                .filter(tenant::Column::TenantCode.eq(code))
                .filter(tenant::Column::Status.eq(1))
                .one(&self.db)
                .await?
                .ok_or_else(|| AppError::BadRequest("@tenant_not_found".to_string()))?;

            if tenant_model.mode == "database" {
                return Err(AppError::BadRequest("@tenant_database_mode_register".to_string()));
            }

            (
                tenant_model.id.clone(),
                tenant_model.tenant_code.clone(),
                tenant_model.tenant_name.clone(),
            )
        } else {
            // 使用全局配置的默认租户 ID
            let default_id = get_tenant_config()
                .and_then(|c| c.default_tenant_id.clone())
                .and_then(|v| match v {
                    Value::String(Some(s)) => Some(s),
                    _ => None,
                })
                .ok_or_else(|| AppError::Internal("@tenant_table_register_need_default".to_string()))?;

            // 查询默认租户的 code/name 用于索引同步
            let tenant_model = tenant::Entity::find()
                .filter(tenant::Column::Id.eq(&default_id))
                .one(&self.db)
                .await?
                .ok_or_else(|| AppError::Internal(format!("@default_tenant_not_found:{}", default_id)))?;

            (default_id, tenant_model.tenant_code.clone(), tenant_model.tenant_name.clone())
        };

        let existing = user::Entity::find()
            .filter(user::Column::UserName.eq(&dto.user_name))
            .filter(user::Column::TenantId.eq(&tenant_id))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("@username_taken".to_string()));
        }

        // 邮箱必填：格式 → 查重 → 一次性消费验证码（通过后账号即带着已验证的邮箱）
        self.assert_email_verified(&dto.email, &dto.email_code, None).await?;

        // 身份由下拉选择决定（字典驱动）：绝不接受前端传 role_id
        let role_id = resolve_selectable_role(&self.db, &dto.role_code).await?;

        let user_model = user::ActiveModel {
            user_name: Set(dto.user_name),
            pass_word: Set(common::hash_password(&dto.pass_word)?),
            nick_name: Set(dto.nick_name),
            email: Set(Some(dto.email.clone())),
            phone: Set(dto.phone),
            identity_type: Set(Some("username".to_string())),
            identity_value: Set(Some(dto.email.clone())),
            status: Set(1),
            admin_flag: Set(0),
            dept_id: Set(None),
            tenant_id: Set(Some(tenant_id.clone())),
            ..Default::default()
        };

        // 用户 + 所选身份角色 + 租户索引同一事务：避免注册中途失败留下无角色/无索引的半成品账号
        let tx = self.db.inner().begin().await?;
        let result = user_model.insert(&tx).await?;

        user_role::ActiveModel {
            user_id: Set(result.id.clone()),
            role_id: Set(role_id),
            tenant_id: Set(Some(tenant_id.clone())),
            ..Default::default()
        }
        .insert(&tx)
        .await?;

        // 同步 tenant_user 索引（主库）
        tenant_user::ActiveModel {
            user_name: Set(result.user_name.clone()),
            tenant_id: Set(tenant_id.clone()),
            tenant_code: Set(tenant_code.clone()),
            user_id: Set(result.id.clone()),
            identity_type: Set("username".to_string()),
            identity_value: Set(Some(dto.email.clone())),
            status: Set(1),
            ..Default::default()
        }.insert(&tx).await?;
        tx.commit().await?;

        Ok(UserInfoVo {
            id: result.id,
            user_name: result.user_name,
            nick_name: result.nick_name,
            email: result.email,
            phone: result.phone,
            avatar: result.avatar,
            roles: vec![],
            permissions: vec![],
            menus: vec![],
            tenant_id: Some(tenant_id),
            tenant_code: Some(tenant_code),
            tenant_name: Some(tenant_name),
            profile_completed: true,
        })
    }

    pub async fn get_user_info(&self) -> Result<UserInfoVo, AppError> {
        let login_id = StpUtil::get_login_id_as_string()
            .await
            .map_err(|e| AppError::Unauthorized(format!("@not_logged_in:{}", e)))?;
        // 客户端会话的 login_id 形如 `用户ID#客户端编码`，业务查询一律用真实用户ID
        let user_id = common::user::base_user_id_from_login_id(&login_id);

        let token_info = StpUtil::get_token_info_current()
            .map_err(|e| AppError::Internal(format!("@token_info_failed:{}", e)))?;

        let extra: serde_json::Value = token_info.extra_data.clone()
            .unwrap_or(serde_json::json!({}));

        let tenant_id: Option<String> = extra.get("tenantId")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let tenant_code: Option<String> = extra.get("tenantCode")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let tenant_mode: Option<String> = extra.get("tenantMode")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // 当前客户端：优先 token extra，extra 缺失（如刷新令牌后）回退解析 login_id 后缀
        let client_code: Option<String> = extra
            .get("clientCode")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .or_else(|| common::user::client_code_from_login_id(&login_id));

        // database 模式下 self.db 会自动路由到租户库（由 SaTokenTenantIdProvider 提供 tenantMode），
        // 无需手动切换数据库。但查询 auth_sys_tenant 全局表时需要临时走主库，
        // 否则会被路由到租户库导致找不到表。
        let tenant_name = if tenant_mode.as_deref() == Some("database") {
            if let Some(ref tid) = tenant_id {
                let _main_db_guard = TenantIgnoreGuard::new();
                tenant::Entity::find()
                    .filter(tenant::Column::Id.eq(tid))
                    .one(&self.db)
                    .await?
                    .map(|t| t.tenant_name)
            } else {
                None
            }
        } else {
            None
        };

        let client_id: Option<String> = match extra.get("clientId").and_then(|v| v.as_str()) {
            Some(id) => Some(id.to_string()),
            None => match client_code.as_deref() {
                Some(code) => {
                    let _main_db_guard = TenantIgnoreGuard::new();
                    client::Entity::find()
                        .filter(client::Column::ClientCode.eq(code))
                        .one(&self.db)
                        .await?
                        .map(|c| c.id)
                }
                None => None,
            },
        };

        let user_model = user::Entity::find()
            .filter(user::Column::Id.eq(&user_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@user_not_found".to_string()))?;

        // 按当前客户端收敛角色/权限（未携带客户端的会话保持历史行为）
        let grant = super::permission_sync::load_user_grant(&self.db, &user_id, client_id.as_deref())
            .await?;
        let (role_codes, permissions, menus) = (grant.role_codes, grant.permissions, grant.menus);

        let menu_vos: Vec<MenuVo> = menus
            .into_iter()
            .filter(|m| m.status == 1 && m.visible == 1 && m.menu_type != "button")
            .map(MenuVo::from)
            .collect();
        let menu_tree = Self::build_menu_tree(menu_vos);

        Ok(UserInfoVo {
            id: user_model.id,
            user_name: user_model.user_name,
            nick_name: user_model.nick_name,
            email: user_model.email,
            phone: user_model.phone,
            avatar: user_model.avatar,
            roles: role_codes,
            permissions,
            menus: menu_tree,
            tenant_id,
            tenant_code,
            tenant_name,
            profile_completed: !user_model.pass_word.is_empty(),
        })
    }

    fn build_menu_tree(menus: Vec<MenuVo>) -> Vec<MenuVo> {
        let mut map: std::collections::HashMap<String, Vec<MenuVo>> = std::collections::HashMap::new();

        for menu in &menus {
            map.entry(menu.parent_id.clone()).or_default();
        }

        for menu in menus {
            map.entry(menu.parent_id.clone()).or_default().push(menu);
        }

        let mut result = Vec::new();
        if let Some(roots) = map.remove("0") {
            for mut root in roots {
                root.children = Self::build_menu_children(root.id.clone(), &mut map);
                result.push(root);
            }
        }

        result.sort_by_key(|m| m.sort_order);
        result
    }

    fn build_menu_children(
        parent_id: String,
        map: &mut std::collections::HashMap<String, Vec<MenuVo>>,
    ) -> Option<Vec<MenuVo>> {
        if let Some(mut children) = map.remove(&parent_id) {
            children.sort_by_key(|m| m.sort_order);
            let mut result = Vec::new();
            for mut child in children {
                child.children = Self::build_menu_children(child.id.clone(), map);
                result.push(child);
            }
            Some(result)
        } else {
            None
        }
    }

    pub async fn logout(&self) -> Result<(), AppError> {
        // 登出前先取出当前 access token / login_id：库的 logout 只删 access token，
        // 不会回收 refresh token（否则登出后 7 天内仍可用它换取新的 access token）
        let current_token = StpUtil::get_token_value().ok().map(|t| t.as_str().to_string());
        let login_id = StpUtil::get_login_id_as_string().await.ok();

        StpUtil::logout_current()
            .await
            .map_err(|e| AppError::Internal(format!("@logout_failed:{}", e)))?;

        if let (Some(token), Some(uid)) = (current_token, login_id) {
            let revoked = super::session_cleanup::revoke_refresh_by_access_token(
                &self.redis,
                self.storage_prefix(),
                &uid,
                &token,
            )
            .await;
            // 回收后同步收敛该账号的残留索引（被删的 refresh 记录对应条目一并清理）
            self.prune_session_index(&uid).await;
            if revoked > 0 {
                log::info!("登出已回收 refresh token: login_id={}, 数量={}", uid, revoked);
            }
        }
        Ok(())
    }
}

/// 登录 token 的 extra 数据附加客户端信息（clientId/clientCode/clientName）。
///
/// 权限校验按 `login_id = 用户ID#客户端编码` 隔离，extra 里的客户端信息
/// 供 `get_user_info`、网关日志等场景读取（刷新令牌丢失 extra 时回退解析 login_id）。
fn client_extra(
    mut extra: serde_json::Value,
    client: Option<&client::Model>,
) -> serde_json::Value {
    if let Some(c) = client {
        if let Some(obj) = extra.as_object_mut() {
            obj.insert("clientId".to_string(), serde_json::json!(c.id));
            obj.insert("clientCode".to_string(), serde_json::json!(c.client_code));
            obj.insert("clientName".to_string(), serde_json::json!(c.client_name));
        }
    }
    extra
}

/// 是否微信类 provider（小程序 / 网站应用）。
///
/// 微信类在身份查找时**视为同一类**：unionid 才是"同一个微信"的全局标识，
/// 小程序绑过的微信在 Web 扫码时必须能被认出来。
pub(super) fn is_wechat_provider(provider: &str) -> bool {
    matches!(provider, "wechat" | "wechat_mp")
}

/// 是否允许自动注册惰性账号：仅「微信类」且「已解析出登录客户端」。
///
/// 保留「已解析出客户端」这一条是**安全边界**：来源不明的登录保持旧行为（报未绑定），不自动建号。
fn can_auto_register(provider: &str, client: Option<&client::Model>) -> bool {
    is_wechat_provider(provider) && client.is_some()
}

/// 校验并一次性消费 Web 扫码的 CSRF state（登录与绑定共用）。
pub(super) async fn consume_qr_state(
    redis: &Redis,
    qr_state: Option<&str>,
) -> Result<(), AppError> {
    let Some(state) = qr_state.map(str::trim).filter(|s| !s.is_empty()) else {
        return Err(AppError::BadRequest("@qr_state_invalid".to_string()));
    };
    let key = format!("{WX_QR_STATE_PREFIX}{state}");
    let mut conn = redis.clone();
    let saved: Option<String> = summer_redis::redis::cmd("GET")
        .arg(&key)
        .query_async(&mut conn)
        .await
        .map_err(|_| AppError::Internal("@captcha_service_unavailable".to_string()))?;
    // 一次性消费：无论校验结果如何都删除，避免同一 state 被重放
    let _ = summer_redis::redis::cmd("DEL")
        .arg(&key)
        .query_async::<i64>(&mut conn)
        .await;
    match saved {
        Some(v) if v == state => Ok(()),
        _ => Err(AppError::BadRequest("@qr_state_invalid".to_string())),
    }
}

/// 按第三方标识查身份行（登录与绑定共用）。
///
/// 微信类**不限 provider**：unionid 才是"同一个微信"的全局标识，
/// 小程序绑过的微信在 Web 扫码时必须能被认出来（不要写成 `provider = 'wechat'` 的精确匹配）。
/// 非微信类限定 provider。
///
/// 顺带做存量兼容：老数据存的是 openid，本次拿到 unionid 时按 openid 再查一次，
/// 命中则把该行升级为 unionid（并发冲突时保留原值并记 warn）。
pub(super) async fn find_identity_row(
    db: &DbConn,
    identity: &ThirdPartyIdentity,
) -> Result<Option<user_identity::Model>, AppError> {
    let mut query = user_identity::Entity::find()
        .filter(user_identity::Column::IdentityValue.eq(&identity.identity_value))
        .filter(user_identity::Column::Status.eq(1));
    if !is_wechat_provider(&identity.provider) {
        query = query.filter(user_identity::Column::Provider.eq(&identity.provider));
    }
    if let Some(row) = query.one(db).await? {
        return Ok(Some(row));
    }

    if !is_wechat_provider(&identity.provider) || !identity.unionid_available {
        return Ok(None);
    }

    // 存量升级：按 openid 找老行
    let legacy = user_identity::Entity::find()
        .filter(user_identity::Column::Provider.is_in(vec![
            "wechat_mp".to_string(),
            "wechat".to_string(),
        ]))
        .filter(user_identity::Column::IdentityValue.eq(&identity.openid))
        .filter(user_identity::Column::Status.eq(1))
        .one(db)
        .await?;
    let Some(row) = legacy else {
        return Ok(None);
    };

    let mut active: user_identity::ActiveModel = row.clone().into();
    active.identity_value = Set(identity.identity_value.clone());
    match active.update(db).await {
        Ok(updated) => Ok(Some(updated)),
        Err(e) => {
            // 唯一索引冲突（该 unionid 已被另一行占用）时保留原值，不阻断登录
            log::warn!(
                "存量微信身份升级 unionid 失败（保留 openid）: user_id={}, openid={}, err={}",
                row.user_id,
                identity.openid,
                e
            );
            Ok(Some(row))
        }
    }
}

/// 第三方授权码换来的身份标识。
pub(super) struct ThirdPartyIdentity {
    /// 主标识：微信类优先 unionid（拿不到时回退 openid）；其他平台为原生 id
    pub identity_value: String,
    /// 原始 openid（微信类存量数据升级用；其他平台同 `identity_value`）
    pub openid: String,
    /// `identity_value` 是否为 unionid（微信应用未绑开放平台时为 false）
    pub unionid_available: bool,
    /// 发起本次解析的 provider
    pub provider: String,
}

/// 解析第三方身份：微信类优先返回 unionid，拿不到时回退 openid 并记 warn。
///
/// 跨端识别完全依赖 unionid（开放平台绑定后两端返回相同 unionid）；
/// 若这里只取 openid，小程序与 Web 会认不出是同一个微信。
pub(super) async fn resolve_third_party_identity(
    provider: &str,
    code: &str,
) -> Result<ThirdPartyIdentity, AppError> {
    let (openid, unionid) = match provider {
        "wechat" => fetch_wechat_web_identity(code).await?,
        "wechat_mp" => fetch_wechat_mp_identity(code).await?,
        "dingtalk" => {
            return Err(AppError::Internal(
                "@dingtalk_not_implemented".to_string(),
            ));
        }
        _ => {
            return Err(AppError::BadRequest(
                "@third_party_login_unsupported".to_string(),
            ));
        }
    };

    let unionid_available = unionid.as_deref().is_some_and(|v| !v.trim().is_empty());
    if !unionid_available {
        log::warn!(
            "微信返回体未包含 unionid（provider={}）：跨端识别将退化为 openid，请确认应用已绑定开放平台账号",
            provider
        );
    }
    let identity_value = unionid
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| openid.clone());

    Ok(ThirdPartyIdentity {
        identity_value,
        openid,
        unionid_available,
        provider: provider.to_string(),
    })
}

/// 微信开放平台「网站应用」code → (openid, unionid)
async fn fetch_wechat_web_identity(
    code: &str,
) -> Result<(String, Option<String>), AppError> {
    let appid = std::env::var("WECHAT_APPID").unwrap_or_default();
    let secret = std::env::var("WECHAT_SECRET").unwrap_or_default();
    if appid.trim().is_empty() || secret.trim().is_empty() {
        return Err(AppError::Internal("@wechat_not_configured".to_string()));
    }

    let url = format!(
        "https://api.weixin.qq.com/sns/oauth2/access_token?appid={}&secret={}&code={}&grant_type=authorization_code",
        appid, secret, code
    );
    let resp: serde_json::Value = reqwest::get(&url)
        .await
        .map_err(|e| AppError::Internal(format!("@wxapp_api_failed:{}", e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("@wxapp_parse_failed:{}", e)))?;
    extract_wechat_identity(&resp)
}

/// 微信小程序 code → (openid, unionid)（sns/jscode2session）
async fn fetch_wechat_mp_identity(
    code: &str,
) -> Result<(String, Option<String>), AppError> {
    let appid = std::env::var("WECHAT_MP_APPID").unwrap_or_default();
    let secret = std::env::var("WECHAT_MP_SECRET").unwrap_or_default();
    if appid.trim().is_empty() || secret.trim().is_empty() {
        return Err(AppError::Internal("@wxapp_not_configured".to_string()));
    }

    let url = format!(
        "https://api.weixin.qq.com/sns/jscode2session?appid={}&secret={}&js_code={}&grant_type=authorization_code",
        appid, secret, code
    );
    let resp: serde_json::Value = reqwest::get(&url)
        .await
        .map_err(|e| AppError::Internal(format!("@wxapp_api_failed:{}", e)))?
        .json()
        .await
        .map_err(|e| AppError::Internal(format!("@wxapp_parse_failed:{}", e)))?;

    if let Some(errmsg) = resp.get("errmsg").and_then(|v: &serde_json::Value| v.as_str())
        && !errmsg.is_empty()
    {
        return Err(AppError::Internal(format!("@wxapp_login_failed:{}", errmsg)));
    }
    extract_wechat_identity(&resp)
}

/// 从微信响应体取 openid 与 unionid（unionid 仅在应用绑定开放平台账号后返回）
fn extract_wechat_identity(resp: &serde_json::Value) -> Result<(String, Option<String>), AppError> {
    let openid = resp
        .get("openid")
        .and_then(|v: &serde_json::Value| v.as_str())
        .map(|s: &str| s.to_string())
        .ok_or_else(|| AppError::Internal("@openid_failed".to_string()))?;
    let unionid = resp
        .get("unionid")
        .and_then(|v: &serde_json::Value| v.as_str())
        .map(|s: &str| s.to_string());
    Ok((openid, unionid))
}

/// 校验中国大陆手机号（`^1[3-9]\d{9}$`）
fn is_valid_cn_mobile(phone: &str) -> bool {
    let bytes = phone.as_bytes();
    bytes.len() == 11
        && bytes[0] == b'1'
        && (b'3'..=b'9').contains(&bytes[1])
        && bytes[2..].iter().all(u8::is_ascii_digit)
}

/// 极简 URL 百分号编码（仅用于拼接微信授权地址里的 redirect_uri）
fn urlencode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.as_bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*byte as char)
            }
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}

/// 校验并解析注册/完善资料时用户选择的身份，返回角色 ID。
///
/// 四道一致性保障中的第 3 道：绕过前端直接调接口也绑不上不存在的角色。
/// 字典固定为 [`DICT_REGISTER_ROLE`]（全端唯一一份身份列表），不接受调用方另传字典编码
/// —— 避免各处口径漂移。
///
/// 角色查找**显式限定默认客户端**（与 `auth_sys_role.client_id` 的 DDL 默认值一致），
/// 否则将来别的客户端出现同名 `role_code` 会产生歧义。
pub async fn resolve_selectable_role(
    db: &DbConn,
    role_code: &str,
) -> Result<String, AppError> {
    let role_code = role_code.trim();
    if role_code.is_empty() {
        return Err(AppError::BadRequest("@role_not_selectable".to_string()));
    }

    let type_ids: Vec<String> = dict_type::Entity::find()
        .filter(dict_type::Column::DictType.eq(DICT_REGISTER_ROLE))
        .filter(dict_type::Column::Status.eq(1))
        .all(db)
        .await?
        .into_iter()
        .map(|t| t.id)
        .collect();
    if type_ids.is_empty() {
        return Err(AppError::BadRequest("@role_not_selectable".to_string()));
    }

    let item = dict_item::Entity::find()
        .filter(dict_item::Column::DictTypeId.is_in(type_ids))
        .filter(dict_item::Column::DictValue.eq(role_code))
        .filter(dict_item::Column::Status.eq(1))
        .one(db)
        .await?;
    if item.is_none() {
        return Err(AppError::BadRequest("@role_not_selectable".to_string()));
    }

    let role_model = role::Entity::find()
        .filter(role::Column::ClientId.eq(DEFAULT_CLIENT_ID))
        .filter(role::Column::RoleCode.eq(role_code))
        .filter(role::Column::Status.eq(1))
        .one(db)
        .await?
        .ok_or_else(|| AppError::BadRequest("@role_not_selectable".to_string()))?;

    Ok(role_model.id)
}
