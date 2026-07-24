use common::error::AppError;
use common::pagination::{PageQuery, PageResult};
use common::tenant_db::get_effective_db;
use common::user::get_current_tenant_mode;
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use system_entity::{user, user_role, tenant_user, tenant};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait};
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::*;
use sea_orm_ext::{ignore_tenant, TenantIgnoreGuard};

use super::dto::{CreateUserDto, UpdateUserDto, UserVo};

#[derive(Clone, Service)]
pub struct UserAppService {
    #[inject(component)]
    db: DbConn,
}

impl UserAppService {
    pub async fn list(&self, query: PageQuery) -> Result<PageResult<UserVo>, AppError> {
        let db = get_effective_db(&self.db).await?;

        let paginator = user::Entity::find()
            .paginate(&db, query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<user::Model> = paginator.fetch_page(query.page - 1).await?;

        let vos: Vec<UserVo> = items.into_iter().map(UserVo::from).collect();
        Ok(PageResult::new(vos, total, query.page, query.page_size))
    }

    pub async fn get_by_id(&self, id: String) -> Result<UserVo, AppError> {
        let db = get_effective_db(&self.db).await?;

        let model: user::Model = user::Entity::find()
            .filter(user::Column::Id.eq(&id))
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        let roles = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(&id))
            .all(&db)
            .await?;

        let role_ids: Vec<String> = roles.into_iter().map(|r| r.role_id).collect();

        let mut vo: UserVo = model.into();
        vo.role_ids = Some(role_ids);
        Ok(vo)
    }

    pub async fn create(&self, dto: CreateUserDto) -> Result<UserVo, AppError> {
        let db = get_effective_db(&self.db).await?;

        let existing: Option<user::Model> = user::Entity::find()
            .filter(user::Column::UserName.eq(&dto.user_name))
            .one(&db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("用户名已存在".to_string()));
        }

        let role_ids = dto.role_ids.clone();
        let active_model = dto.into_active_model()?;
        let model = active_model.insert(&db).await?;

        if let Some(ids) = role_ids {
            let models: Vec<user_role::ActiveModel> = ids.into_iter().map(|role_id| user_role::ActiveModel {
                user_id: Set(model.id.clone()),
                role_id: Set(role_id),
                ..Default::default()
            }).collect();
            user_role::Entity::insert_many_with_fill(models, &db).await?;
        }

        self.sync_tenant_user_index(&model).await?;

        let mut vo: UserVo = model.into();
        vo.role_ids = None;
        Ok(vo)
    }

    pub async fn update(&self, dto: UpdateUserDto) -> Result<UserVo, AppError> {
        let db = get_effective_db(&self.db).await?;

        let role_ids = dto.role_ids.clone();
        let active_model = dto.into_active_model();
        let model = active_model.update(&db).await?;

        if let Some(ids) = role_ids {
            user_role::Entity::delete_many()
                .filter(user_role::Column::UserId.eq(&model.id))
                .exec(&db)
                .await?;

            let models: Vec<user_role::ActiveModel> = ids.into_iter().map(|role_id| user_role::ActiveModel {
                user_id: Set(model.id.clone()),
                role_id: Set(role_id),
                ..Default::default()
            }).collect();
            user_role::Entity::insert_many_with_fill(models, &db).await?;
        }

        self.sync_tenant_user_index(&model).await?;

        let mut vo: UserVo = model.into();
        vo.role_ids = None;
        Ok(vo)
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {
        let db = get_effective_db(&self.db).await?;

        let model: user::Model = user::Entity::find()
            .filter(user::Column::Id.eq(&id))
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        let mut am: user::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&db).await?;

        user_role::Entity::delete_many()
            .filter(user_role::Column::UserId.eq(&id))
            .exec(&db)
            .await?;

        self.remove_tenant_user_index(&id).await?;

        Ok(())
    }

    pub async fn update_status(&self, id: String, status: i32) -> Result<(), AppError> {
        let db = get_effective_db(&self.db).await?;

        let model: user::Model = user::Entity::find()
            .filter(user::Column::Id.eq(&id))
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        let mut active: user::ActiveModel = model.into();
        active.status = Set(status);
        active.update(&db).await?;

        if let Some(idx) = self.find_tenant_user_index(&id).await? {
            let mut idx_am: tenant_user::ActiveModel = idx.into();
            idx_am.status = Set(status);
            let _guard = TenantIgnoreGuard::new();
            idx_am.update(&self.db).await?;
        }

        Ok(())
    }

    pub async fn assign_roles(&self, user_id: String, role_ids: Vec<String>) -> Result<(), AppError> {
        let db = get_effective_db(&self.db).await?;

        let _model: user::Model = user::Entity::find()
            .filter(user::Column::Id.eq(&user_id))
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        user_role::Entity::delete_many()
            .filter(user_role::Column::UserId.eq(&user_id))
            .exec(&db)
            .await?;

        let models: Vec<user_role::ActiveModel> = role_ids.into_iter().map(|role_id| user_role::ActiveModel {
            user_id: Set(user_id.clone()),
            role_id: Set(role_id),
            ..Default::default()
        }).collect();
        user_role::Entity::insert_many_with_fill(models, &db).await?;

        Ok(())
    }

    pub async fn get_role_ids(&self, user_id: String) -> Result<Vec<String>, AppError> {
        let db = get_effective_db(&self.db).await?;

        let roles = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(&user_id))
            .all(&db)
            .await?;

        Ok(roles.into_iter().map(|r| r.role_id).collect())
    }

    #[ignore_tenant]
    async fn sync_tenant_user_index(&self, user_model: &user::Model) -> Result<(), AppError> {
        let tenant_mode = get_current_tenant_mode();
        if tenant_mode.as_deref() != Some("database") {
            return Ok(());
        }

        let tenant_id = match common::user::get_current_tenant_id() {
            Some(id) => id,
            None => return Ok(()),
        };

        let tenant_model = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(&tenant_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Internal("租户不存在".to_string()))?;

        let existing = tenant_user::Entity::find()
            .filter(tenant_user::Column::UserId.eq(&user_model.id))
            .filter(tenant_user::Column::TenantId.eq(&tenant_id))
            .one(&self.db)
            .await?;

        if let Some(idx) = existing {
            let mut am: tenant_user::ActiveModel = idx.into();
            am.user_name = Set(user_model.user_name.clone());
            am.identity_type = Set(user_model.identity_type.clone().unwrap_or_else(|| "username".to_string()));
            am.identity_value = Set(user_model.email.clone().or(user_model.phone.clone()));
            am.status = Set(user_model.status);
            am.update(&self.db).await?;
        } else {
            tenant_user::ActiveModel {
                user_name: Set(user_model.user_name.clone()),
                tenant_id: Set(tenant_id.clone()),
                tenant_code: Set(tenant_model.tenant_code.clone()),
                user_id: Set(user_model.id.clone()),
                identity_type: Set(user_model.identity_type.clone().unwrap_or_else(|| "username".to_string())),
                identity_value: Set(user_model.email.clone().or(user_model.phone.clone())),
                status: Set(user_model.status),
                ..Default::default()
            }.insert(&self.db).await?;
        }

        Ok(())
    }

    #[ignore_tenant]
    async fn remove_tenant_user_index(&self, user_id: &str) -> Result<(), AppError> {
        let tenant_mode = get_current_tenant_mode();
        if tenant_mode.as_deref() != Some("database") {
            return Ok(());
        }

        let tenant_id = match common::user::get_current_tenant_id() {
            Some(id) => id,
            None => return Ok(()),
        };

        if let Some(idx) = tenant_user::Entity::find()
            .filter(tenant_user::Column::UserId.eq(user_id))
            .filter(tenant_user::Column::TenantId.eq(&tenant_id))
            .one(&self.db)
            .await?
        {
            let mut am: tenant_user::ActiveModel = idx.into();
            am.status = Set(0);
            am.update(&self.db).await?;
        }

        Ok(())
    }

    #[ignore_tenant]
    async fn find_tenant_user_index(&self, user_id: &str) -> Result<Option<tenant_user::Model>, AppError> {
        Ok(tenant_user::Entity::find()
            .filter(tenant_user::Column::UserId.eq(user_id))
            .one(&self.db)
            .await?)
    }
}