use super::dto::{LocateTenantDto, LocateTenantVo, LoginDto, RegisterDto, TokenVo, UserInfoVo};
use crate::menu::dto::MenuVo;
use common::error::AppError;
use common::tenant_db::get_effective_db_by_tenant_id;
use sa_token_adapter::storage::SaStorage;
use sa_token_core::refresh::RefreshTokenManager;
use sa_token_core::StpUtil;
use sea_orm::prelude::*;
use sea_orm::{ActiveValue::Set, ColumnTrait, QueryFilter, QueryOrder};
use sea_orm_ext::{tenant_scope, TenantIgnoreGuard};
use sea_query::Value;
use std::sync::Arc;
use summer::plugin::service::Service;
use summer_redis::Redis;
use summer_sa_token::storage::SummerRedisStorage;
use summer_sa_token::{CoreConfig, SaTokenConfig};
use summer_sea_orm::DbConn;
use system_entity::{menu, role, role_menu, tenant, tenant_user, user, user_role};

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
    fn create_refresh_manager(&self) -> RefreshTokenManager {
        let storage: Arc<dyn SaStorage> = Arc::new(
            SummerRedisStorage::new(
                self.redis.clone(),
                self.sa_token_config.storage_prefix.clone(),
                self.sa_token_config.rewrite_storage_prefix,
            )
        );
        RefreshTokenManager::new(storage, Arc::new(CoreConfig::from(self.sa_token_config.clone())))
    }

    pub async fn locate_tenant(&self, dto: LocateTenantDto) -> Result<LocateTenantVo, AppError> {
        let _guard = TenantIgnoreGuard::new();

        let index = tenant_user::Entity::find()
            .filter(tenant_user::Column::UserName.eq(&dto.user_name))
            .filter(tenant_user::Column::Status.eq(1))
            .one(&self.db)
            .await?;

        let index = match index {
            Some(i) => i,
            None => {
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
            }
        };

        let tenant_model = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(&index.tenant_id))
            .filter(tenant::Column::Status.eq(1))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

        if let Some(expire) = tenant_model.expire_time {
            if expire < chrono::Utc::now() {
                return Err(AppError::Unauthorized("租户已过期".to_string()));
            }
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
        let login_type = dto.login_type.as_deref().unwrap_or("password");

        match login_type {
            "password" => {
                if dto.tenant_code.is_some() {
                    self.login_database_mode(dto).await
                } else {
                    self.login_auto_detect(dto).await
                }
            }
            "wechat" => self.login_third_party("wechat", dto.code.as_deref(), &dto.state).await,
            "dingtalk" => self.login_third_party("dingtalk", dto.code.as_deref(), &dto.state).await,
            _ => self.login_table_mode(dto).await,
        }
    }

    async fn login_auto_detect(&self, dto: LoginDto) -> Result<TokenVo, AppError> {
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
                .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

            if tenant_model.mode == "database" {
                drop(_guard);
                let mut db_dto = dto.clone();
                db_dto.tenant_code = Some(tenant_model.tenant_code.clone());
                return self.login_database_mode(db_dto).await;
            }
        }

        drop(_guard);
        self.login_table_mode(dto).await
    }

    async fn login_database_mode(&self, dto: LoginDto) -> Result<TokenVo, AppError> {
        let user_name = dto.user_name.as_deref()
            .ok_or_else(|| AppError::BadRequest("用户名不能为空".to_string()))?;
        let tenant_code = dto.tenant_code.as_deref()
            .ok_or_else(|| AppError::BadRequest("租户编码不能为空".to_string()))?;
        let remember_me = dto.remember_me.unwrap_or(false);

        let _guard = TenantIgnoreGuard::new();

        let tenant_model = tenant::Entity::find()
            .filter(tenant::Column::TenantCode.eq(tenant_code))
            .filter(tenant::Column::Status.eq(1))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

        if let Some(expire) = tenant_model.expire_time {
            if expire < chrono::Utc::now() {
                return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
            }
        }

        let tenant_db = self.get_tenant_database(&tenant_model.id).await?;

        let user_model = user::Entity::find()
            .filter(user::Column::UserName.eq(user_name))
            .one(&tenant_db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

        if user_model.pass_word != dto.pass_word {
            return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
        }

        if user_model.status != 1 {
            return Err(AppError::Unauthorized("账号已被禁用".to_string()));
        }

        self.do_login(&tenant_model, &user_model, &tenant_db, "database", remember_me).await
    }

    async fn login_third_party(
        &self,
        provider: &str,
        code: Option<&str>,
        state: &Option<String>,
    ) -> Result<TokenVo, AppError> {
        let code = code.ok_or_else(|| AppError::BadRequest("授权码不能为空".to_string()))?;
        let tenant_code = state.as_deref().unwrap_or("");

        let openid = self.get_third_party_openid(provider, code).await?;

        let _guard = TenantIgnoreGuard::new();

        let index = tenant_user::Entity::find()
            .filter(tenant_user::Column::IdentityType.eq(provider))
            .filter(tenant_user::Column::IdentityValue.eq(&openid))
            .filter(tenant_user::Column::Status.eq(1))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("该账号未绑定".to_string()))?;

        let tenant_model = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(&index.tenant_id))
            .filter(tenant::Column::Status.eq(1))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

        if !tenant_code.is_empty() && tenant_model.tenant_code != tenant_code {
            return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
        }

        let tenant_mode = tenant_model.mode.as_str();
        let tenant_db = if tenant_mode == "database" {
            Some(self.get_tenant_database(&tenant_model.id).await?)
        } else {
            None
        };

        let db = tenant_db.as_ref().unwrap_or(&self.db);

        let user_model = user::Entity::find()
            .filter(user::Column::Id.eq(&index.user_id))
            .one(db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

        if user_model.status != 1 {
            return Err(AppError::Unauthorized("账号已被禁用".to_string()));
        }

        self.do_login(&tenant_model, &user_model, db, tenant_mode, false).await
    }

    async fn login_table_mode(&self, dto: LoginDto) -> Result<TokenVo, AppError> {
        let login_type = dto.login_type.as_deref().unwrap_or("username");
        let remember_me = dto.remember_me.unwrap_or(false);
        let _guard = TenantIgnoreGuard::new();

        let user_model = match login_type {
            "email" => {
                let email = dto.email.as_deref().ok_or_else(|| {
                    AppError::BadRequest("邮箱不能为空".to_string())
                })?;
                user::Entity::find()
                    .filter(user::Column::Email.eq(email))
                    .one(&self.db)
                    .await?
                    .ok_or_else(|| AppError::Unauthorized("邮箱或密码错误".to_string()))?
            }
            _ => {
                let user_name = dto.user_name.as_deref().unwrap_or("");
                user::Entity::find()
                    .filter(user::Column::UserName.eq(user_name))
                    .one(&self.db)
                    .await?
                    .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?
            }
        };

        if user_model.pass_word != dto.pass_word {
            return Err(AppError::Unauthorized("密码错误".to_string()));
        }

        if user_model.status != 1 {
            return Err(AppError::Unauthorized("账号已被禁用".to_string()));
        }

        let user_id_str = user_model.id.to_string();
        let extra = serde_json::json!({
            "userId": user_id_str,
            "userName": user_model.user_name,
            "nickName": user_model.nick_name,
            "tenantId": user_model.tenant_id,
            "tenantMode": "table",
        });

        let token_value = StpUtil::login_with_extra(&user_id_str, extra.clone()).await.map_err(|e| {
            AppError::Internal(format!("登录失败: {}", e))
        })?;

        let (role_codes, permissions) = {
            let query = async {
                let user_roles = user_role::Entity::find()
                    .filter(user_role::Column::UserId.eq(&user_model.id))
                    .all(&self.db)
                    .await?;

                let role_ids: Vec<String> = user_roles.iter().map(|r| r.role_id.clone()).collect();
                let roles = role::Entity::find()
                    .filter(role::Column::Id.is_in(role_ids.clone()))
                    .all(&self.db)
                    .await?;

                let role_codes: Vec<String> = roles.iter().map(|r| r.role_code.clone()).collect();

                let role_menus = role_menu::Entity::find()
                    .filter(role_menu::Column::RoleId.is_in(role_ids))
                    .all(&self.db)
                    .await?;

                let menu_ids: Vec<String> = role_menus.iter().map(|rm| rm.menu_id.clone()).collect();
                let menus = menu::Entity::find()
                    .filter(menu::Column::Id.is_in(menu_ids))
                    .all(&self.db)
                    .await?;

                let permissions: Vec<String> = menus
                    .iter()
                    .filter_map(|m| m.permission.clone())
                    .collect();

                Ok::<_, AppError>((role_codes, permissions))
            };

            if let Some(tid) = &user_model.tenant_id {
                let tid_value = Value::String(Some(tid.clone()));
                tenant_scope(tid_value, query).await?
            } else {
                query.await?
            }
        };

        StpUtil::set_roles(&user_id_str, role_codes).await.ok();
        StpUtil::set_permissions(&user_id_str, permissions).await.ok();

        let token = token_value.as_str().to_string();
        let token_info = StpUtil::get_token_info(&token_value).await
            .map_err(|e| AppError::Internal(format!("获取令牌信息失败: {}", e)))?;
        let expire_time = token_info.expire_time.map(|t| t.timestamp());

        let (refresh_token, refresh_expire_time) = if remember_me {
            let refresh_mgr = self.create_refresh_manager();
            let rt = refresh_mgr.generate(&user_id_str);
            refresh_mgr.store_with_extra(&rt, &token, &user_id_str, Some(&extra)).await
                .map_err(|e| AppError::Internal(format!("生成刷新令牌失败: {}", e)))?;

            let refresh_exp = if self.sa_token_config.refresh_token_timeout > 0 {
                Some(chrono::Utc::now().timestamp() + self.sa_token_config.refresh_token_timeout as i64)
            } else {
                None
            };

            (Some(rt), refresh_exp)
        } else {
            (None, None)
        };

        Ok(TokenVo {
            token,
            token_name: self.sa_token_config.token_name.clone(),
            token_prefix: self.sa_token_config.token_prefix.clone().unwrap_or_default(),
            refresh_token,
            expire_time,
            refresh_expire_time,
        })
    }

    async fn do_login(
        &self,
        tenant_model: &tenant::Model,
        user_model: &user::Model,
        db: &DatabaseConnection,
        tenant_mode: &str,
        remember_me: bool,
    ) -> Result<TokenVo, AppError> {
        let (role_codes, permissions) = self.load_user_permissions(db, &user_model.id).await?;

        let user_id_str = user_model.id.to_string();
        let extra = serde_json::json!({
            "userId": user_id_str,
            "userName": user_model.user_name,
            "nickName": user_model.nick_name,
            "tenantId": tenant_model.id.clone(),
            "tenantCode": tenant_model.tenant_code.clone(),
            "tenantMode": tenant_mode,
        });

        let token_value = StpUtil::login_with_extra(&user_id_str, extra.clone()).await
            .map_err(|e| AppError::Internal(format!("登录失败: {}", e)))?;

        StpUtil::set_roles(&user_id_str, role_codes).await.ok();
        StpUtil::set_permissions(&user_id_str, permissions).await.ok();

        let token = token_value.as_str().to_string();
        let token_info = StpUtil::get_token_info(&token_value).await
            .map_err(|e| AppError::Internal(format!("获取令牌信息失败: {}", e)))?;
        let expire_time = token_info.expire_time.map(|t| t.timestamp());

        let (refresh_token, refresh_expire_time) = if remember_me {
            let refresh_mgr = self.create_refresh_manager();
            let rt = refresh_mgr.generate(&user_id_str);
            refresh_mgr.store_with_extra(&rt, &token, &user_id_str, Some(&extra)).await
                .map_err(|e| AppError::Internal(format!("生成刷新令牌失败: {}", e)))?;

            let refresh_exp = if self.sa_token_config.refresh_token_timeout > 0 {
                Some(chrono::Utc::now().timestamp() + self.sa_token_config.refresh_token_timeout as i64)
            } else {
                None
            };

            (Some(rt), refresh_exp)
        } else {
            (None, None)
        };

        Ok(TokenVo {
            token,
            token_name: self.sa_token_config.token_name.clone(),
            token_prefix: self.sa_token_config.token_prefix.clone().unwrap_or_default(),
            refresh_token,
            expire_time,
            refresh_expire_time,
        })
    }

    async fn get_tenant_database(&self, tenant_id: &str) -> Result<DatabaseConnection, AppError> {
        get_effective_db_by_tenant_id(&self.db,Some(tenant_id.to_string())).await
    }

    async fn load_user_permissions(
        &self,
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<(Vec<String>, Vec<String>), AppError> {
        let user_roles = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        let role_ids: Vec<String> = user_roles.iter().map(|r| r.role_id.clone()).collect();

        let roles = role::Entity::find()
            .filter(role::Column::Id.is_in(role_ids.clone()))
            .all(db)
            .await?;

        let role_codes: Vec<String> = roles.iter().map(|r| r.role_code.clone()).collect();

        let role_menus = role_menu::Entity::find()
            .filter(role_menu::Column::RoleId.is_in(role_ids))
            .all(db)
            .await?;

        let menu_ids: Vec<String> = role_menus.iter().map(|rm| rm.menu_id.clone()).collect();

        let menus = menu::Entity::find()
            .filter(menu::Column::Id.is_in(menu_ids))
            .filter(menu::Column::Status.eq(1))
            .all(db)
            .await?;

        let permissions: Vec<String> = menus.iter()
            .filter_map(|m| m.permission.clone())
            .collect();

        Ok((role_codes, permissions))
    }

    async fn get_third_party_openid(&self, provider: &str, code: &str) -> Result<String, AppError> {
        match provider {
            "wechat" => self.get_wechat_openid(code).await,
            "dingtalk" => self.get_dingtalk_openid(code).await,
            _ => Err(AppError::BadRequest("不支持的第三方登录".to_string())),
        }
    }

    async fn get_wechat_openid(&self, code: &str) -> Result<String, AppError> {
        let appid = std::env::var("WECHAT_APPID").unwrap_or_default();
        let secret = std::env::var("WECHAT_SECRET").unwrap_or_default();

        if appid.is_empty() || secret.is_empty() {
            return Err(AppError::Internal("微信配置未设置".to_string()));
        }

        let url = format!(
            "https://api.weixin.qq.com/sns/oauth2/access_token?appid={}&secret={}&code={}&grant_type=authorization_code",
            appid, secret, code
        );

        let resp: serde_json::Value = reqwest::get(&url).await
            .map_err(|e| AppError::Internal(format!("微信接口调用失败: {}", e)))?
            .json()
            .await
            .map_err(|e| AppError::Internal(format!("解析微信响应失败: {}", e)))?;

        resp.get("openid")
            .and_then(|v: &serde_json::Value| v.as_str())
            .map(|s: &str| s.to_string())
            .ok_or_else(|| AppError::Internal("获取openid失败".to_string()))
    }

    async fn get_dingtalk_openid(&self, _code: &str) -> Result<String, AppError> {
        Err(AppError::Internal("钉钉登录尚未实现".to_string()))
    }

    pub async fn refresh_token(&self, refresh_token_str: &str) -> Result<TokenVo, AppError> {
        let refresh_mgr = self.create_refresh_manager();

        let (new_token_value, _login_id) = refresh_mgr.refresh_access_token(refresh_token_str).await
            .map_err(|e| AppError::Unauthorized(format!("刷新令牌无效或已过期: {}", e)))?;

        let token_info = StpUtil::get_token_info(&new_token_value).await
            .map_err(|e| AppError::Internal(format!("获取令牌信息失败: {}", e)))?;

        let expire_time = token_info.expire_time.map(|t| t.timestamp());
        let refresh_expire_time = if self.sa_token_config.refresh_token_timeout > 0 {
            Some(chrono::Utc::now().timestamp() + self.sa_token_config.refresh_token_timeout as i64)
        } else {
            None
        };

        Ok(TokenVo {
            token: new_token_value.as_str().to_string(),
            token_name: self.sa_token_config.token_name.clone(),
            token_prefix: self.sa_token_config.token_prefix.clone().unwrap_or_default(),
            refresh_token: Some(refresh_token_str.to_string()),
            expire_time,
            refresh_expire_time,
        })
    }

    pub async fn register(&self, dto: RegisterDto) -> Result<UserInfoVo, AppError> {
        let _guard = TenantIgnoreGuard::new();

        if let Some(tenant_code) = &dto.tenant_code {
            let tenant_model = tenant::Entity::find()
                .filter(tenant::Column::TenantCode.eq(tenant_code))
                .filter(tenant::Column::Status.eq(1))
                .one(&self.db)
                .await?
                .ok_or_else(|| AppError::BadRequest("租户不存在".to_string()))?;

            if tenant_model.mode == "database" {
                let tenant_db = self.get_tenant_database(&tenant_model.id).await?;

                let existing = user::Entity::find()
                    .filter(user::Column::UserName.eq(&dto.user_name))
                    .one(&tenant_db)
                    .await?;

                if existing.is_some() {
                    return Err(AppError::BadRequest("用户名已存在".to_string()));
                }

                let user_model = user::ActiveModel {
                    user_name: Set(dto.user_name.clone()),
                    pass_word: Set(dto.pass_word.clone()),
                    nick_name: Set(dto.nick_name.clone()),
                    email: Set(dto.email.clone()),
                    phone: Set(dto.phone.clone()),
                    identity_type: Set(Some("username".to_string())),
                    identity_value: Set(dto.email.clone().or(dto.phone.clone())),
                    status: Set(1),
                    admin_flag: Set(0),
                    dept_id: Set(None),
                    ..Default::default()
                };
                let result = user_model.insert(&tenant_db).await?;

                tenant_user::ActiveModel {
                    user_name: Set(result.user_name.clone()),
                    tenant_id: Set(tenant_model.id.clone()),
                    tenant_code: Set(tenant_model.tenant_code.clone()),
                    user_id: Set(result.id.clone()),
                    identity_type: Set("username".to_string()),
                    identity_value: Set(result.email.clone().or(result.phone.clone())),
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
                });
            }
        }

        let existing = user::Entity::find()
            .filter(user::Column::UserName.eq(&dto.user_name))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("用户名已存在".to_string()));
        }

        let user_model = user::ActiveModel {
            user_name: Set(dto.user_name),
            pass_word: Set(dto.pass_word),
            nick_name: Set(dto.nick_name),
            email: Set(dto.email),
            phone: Set(dto.phone),
            identity_type: Set(Some("username".to_string())),
            status: Set(1),
            admin_flag: Set(0),
            dept_id: Set(None),
            ..Default::default()
        };

        let result = user_model.insert(&self.db).await?;

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
            tenant_id: result.tenant_id,
            tenant_code: None,
            tenant_name: None,
        })
    }

    pub async fn get_user_info(&self) -> Result<UserInfoVo, AppError> {
        let user_id = StpUtil::get_login_id_as_string()
            .await
            .map_err(|e| AppError::Unauthorized(format!("未登录: {}", e)))?;

        let token_info = StpUtil::get_token_info_current()
            .map_err(|e| AppError::Internal(format!("获取令牌信息失败: {}", e)))?;

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

        let (db, tenant_name) = if tenant_mode == Some("database".to_string()) && tenant_id.is_some() {
            let tenant_db = self.get_tenant_database(tenant_id.as_ref().unwrap()).await?;

            let tenant_model = tenant::Entity::find()
                .filter(tenant::Column::Id.eq(tenant_id.as_ref().unwrap()))
                .one(&self.db)
                .await?;

            (tenant_db, tenant_model.map(|t| t.tenant_name))
        } else {
            (self.db.clone(), None)
        };

        let user_model = user::Entity::find()
            .filter(user::Column::Id.eq(&user_id))
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        let (role_codes, permissions, menu_tree) = {
            let user_roles = user_role::Entity::find()
                .filter(user_role::Column::UserId.eq(&user_id))
                .all(&db)
                .await?;

            let role_ids: Vec<String> = user_roles.iter().map(|r| r.role_id.clone()).collect();
            let roles = role::Entity::find()
                .filter(role::Column::Id.is_in(role_ids.clone()))
                .all(&db)
                .await?;

            let role_codes: Vec<String> = roles.iter().map(|r| r.role_code.clone()).collect();

            let role_menus = role_menu::Entity::find()
                .filter(role_menu::Column::RoleId.is_in(role_ids))
                .all(&db)
                .await?;

            let menu_ids: Vec<String> = role_menus.iter().map(|rm| rm.menu_id.clone()).collect();
            let menus = menu::Entity::find()
                .filter(menu::Column::Id.is_in(menu_ids))
                .filter(menu::Column::Status.eq(1))
                .filter(menu::Column::Visible.eq(1))
                .order_by_asc(menu::Column::SortOrder)
                .all(&db)
                .await?;

            let permissions: Vec<String> = menus
                .iter()
                .filter_map(|m| m.permission.clone())
                .collect();

            let menu_vos: Vec<MenuVo> = menus
                .into_iter()
                .filter(|m| m.menu_type != "button")
                .map(MenuVo::from)
                .collect();
            let menu_tree = Self::build_menu_tree(menu_vos);

            (role_codes, permissions, menu_tree)
        };

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
        StpUtil::logout_current()
            .await
            .map_err(|e| AppError::Internal(format!("登出失败: {}", e)))?;
        Ok(())
    }
}