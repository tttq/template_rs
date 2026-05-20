use common::error::AppError;
use summer::plugin::service::Service;
use summer_sea_orm::DbConn;
use sa_token_core::StpUtil;
use system_entity::{user, user_role, role, role_menu, menu};
use sea_orm::{QueryFilter, ColumnTrait, ActiveValue::Set, QueryOrder};
use sea_orm::prelude::*;

use super::dto::{LoginDto, RegisterDto, TokenVo, UserInfoVo};
use crate::menu::dto::MenuVo;

#[derive(Clone, Service)]
pub struct AuthAppService {
    #[inject(component)]
    db: DbConn,
}

impl AuthAppService {
    pub async fn login(&self, dto: LoginDto) -> Result<TokenVo, AppError> {
        let login_type = dto.login_type.as_deref().unwrap_or("username");

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
                let user_name = dto.user_name.as_deref().ok_or_else(|| {
                    AppError::BadRequest("用户名不能为空".to_string())
                })?;
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
        });
        let token_value = StpUtil::login_with_extra(&user_id_str, extra).await.map_err(|e| {
            AppError::Internal(format!("登录失败: {}", e))
        })?;

        let user_roles = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(user_model.id))
            .all(&self.db)
            .await?;

        let role_ids: Vec<i64> = user_roles.iter().map(|r| r.role_id).collect();
        let roles = role::Entity::find()
            .filter(role::Column::Id.is_in(role_ids.clone()))
            .all(&self.db)
            .await?;

        let role_codes: Vec<String> = roles.iter().map(|r| r.role_code.clone()).collect();

        let role_menus = role_menu::Entity::find()
            .filter(role_menu::Column::RoleId.is_in(role_ids))
            .all(&self.db)
            .await?;

        let menu_ids: Vec<i64> = role_menus.iter().map(|rm| rm.menu_id).collect();
        let menus = menu::Entity::find()
            .filter(menu::Column::Id.is_in(menu_ids))
            .all(&self.db)
            .await?;

        let permissions: Vec<String> = menus
            .iter()
            .filter_map(|m| m.permission.clone())
            .collect();

        StpUtil::set_roles(&user_id_str, role_codes).await.ok();
        StpUtil::set_permissions(&user_id_str, permissions).await.ok();

        let token = token_value.as_str().to_string();

        Ok(TokenVo {
            token,
            token_name: "Authorization".to_string(),
            token_prefix: "Bearer ".to_string(),
        })
    }

    pub async fn register(&self, dto: RegisterDto) -> Result<UserInfoVo, AppError> {
        let register_type = dto.register_type.as_deref().unwrap_or("username");

        let existing = user::Entity::find()
            .filter(user::Column::UserName.eq(&dto.user_name))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("用户名已存在".to_string()));
        }

        if register_type == "email" {
            if dto.email.is_none() || dto.email.as_deref().unwrap_or("").is_empty() {
                return Err(AppError::BadRequest("邮箱不能为空".to_string()));
            }
            let email_dup = user::Entity::find()
                .filter(user::Column::Email.eq(dto.email.as_deref().unwrap()))
                .one(&self.db)
                .await?;
            if email_dup.is_some() {
                return Err(AppError::BadRequest("邮箱已被注册".to_string()));
            }
        }

        let user_model = user::ActiveModel {
            user_name: Set(dto.user_name),
            pass_word: Set(dto.pass_word),
            nick_name: Set(dto.nick_name),
            email: Set(dto.email),
            phone: Set(dto.phone),
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
        })
    }

    pub async fn get_user_info(&self) -> Result<UserInfoVo, AppError> {
        let user_id_str = StpUtil::get_login_id_as_string()
            .await
            .map_err(|e| AppError::Unauthorized(format!("未登录: {}", e)))?;

        let user_id: i64 = user_id_str.parse().map_err(|_| {
            AppError::BadRequest("无效的用户ID".to_string())
        })?;

        let user_model = user::Entity::find()
            .filter(user::Column::Id.eq(user_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        let user_roles = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(user_id))
            .all(&self.db)
            .await?;

        let role_ids: Vec<i64> = user_roles.iter().map(|r| r.role_id).collect();
        let roles = role::Entity::find()
            .filter(role::Column::Id.is_in(role_ids.clone()))
            .all(&self.db)
            .await?;

        let role_codes: Vec<String> = roles.iter().map(|r| r.role_code.clone()).collect();

        let role_menus = role_menu::Entity::find()
            .filter(role_menu::Column::RoleId.is_in(role_ids))
            .all(&self.db)
            .await?;

        let menu_ids: Vec<i64> = role_menus.iter().map(|rm| rm.menu_id).collect();
        let menus = menu::Entity::find()
            .filter(menu::Column::Id.is_in(menu_ids))
            .filter(menu::Column::Status.eq(1))
            .filter(menu::Column::Visible.eq(1))
            .order_by_asc(menu::Column::SortOrder)
            .all(&self.db)
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
        })
    }

    fn build_menu_tree(menus: Vec<MenuVo>) -> Vec<MenuVo> {
        let mut map: std::collections::HashMap<i64, Vec<MenuVo>> = std::collections::HashMap::new();

        for menu in &menus {
            map.entry(menu.parent_id).or_default();
        }

        for menu in menus {
            map.entry(menu.parent_id).or_default().push(menu);
        }

        let mut result = Vec::new();
        if let Some(roots) = map.remove(&0) {
            for mut root in roots {
                root.children = Self::build_menu_children(root.id, &mut map);
                result.push(root);
            }
        }

        result.sort_by_key(|m| m.sort_order);
        result
    }

    fn build_menu_children(
        parent_id: i64,
        map: &mut std::collections::HashMap<i64, Vec<MenuVo>>,
    ) -> Option<Vec<MenuVo>> {
        if let Some(mut children) = map.remove(&parent_id) {
            children.sort_by_key(|m| m.sort_order);
            let mut result = Vec::new();
            for mut child in children {
                child.children = Self::build_menu_children(child.id, map);
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
