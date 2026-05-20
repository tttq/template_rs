use common::error::AppError;
use common::pagination::{PageQuery, PageResult};
use summer::plugin::service::Service;
use summer_sea_orm::DbConn;
use system_entity::{user, user_role};
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, PaginatorTrait};
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::*;

use super::dto::{CreateUserDto, UpdateUserDto, UserVo};

#[derive(Clone, Service)]
pub struct UserAppService {
    #[inject(component)]
    db: DbConn,
}

impl UserAppService {
    pub async fn list(&self, query: PageQuery) -> Result<PageResult<UserVo>, AppError> {
        let paginator = user::Entity::find()
            .paginate(&self.db, query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<user::Model> = paginator.fetch_page(query.page - 1).await?;

        let vos: Vec<UserVo> = items.into_iter().map(UserVo::from).collect();
        Ok(PageResult::new(vos, total, query.page, query.page_size))
    }

    pub async fn get_by_id(&self, id: i64) -> Result<UserVo, AppError> {
        let model: user::Model = user::Entity::find()
            .filter(user::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        let roles = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(id))
            .all(&self.db)
            .await?;

        let role_ids: Vec<i64> = roles.into_iter().map(|r| r.role_id).collect();

        let mut vo: UserVo = model.into();
        vo.role_ids = Some(role_ids);
        Ok(vo)
    }

    pub async fn create(&self, dto: CreateUserDto) -> Result<UserVo, AppError> {
        let existing: Option<user::Model> = user::Entity::find()
            .filter(user::Column::UserName.eq(&dto.user_name))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("用户名已存在".to_string()));
        }

        let role_ids = dto.role_ids.clone();
        let active_model = dto.into_active_model();
        let model = active_model.insert(&self.db).await?;

        if let Some(ids) = role_ids {
            for role_id in ids {
                user_role::ActiveModel {
                    user_id: Set(model.id),
                    role_id: Set(role_id),
                    ..Default::default()
                }
                .insert(&self.db)
                .await?;
            }
        }

        let mut vo: UserVo = model.into();
        vo.role_ids = None;
        Ok(vo)
    }

    pub async fn update(&self, dto: UpdateUserDto) -> Result<UserVo, AppError> {
        let role_ids = dto.role_ids.clone();
        let active_model = dto.into_active_model();
        let model = active_model.update(&self.db).await?;

        if let Some(ids) = role_ids {
            user_role::Entity::delete_many()
                .filter(user_role::Column::UserId.eq(model.id))
                .exec(&self.db)
                .await?;

            for role_id in ids {
                user_role::ActiveModel {
                    user_id: Set(model.id),
                    role_id: Set(role_id),
                    ..Default::default()
                }
                .insert(&self.db)
                .await?;
            }
        }

        let mut vo: UserVo = model.into();
        vo.role_ids = None;
        Ok(vo)
    }

    pub async fn delete(&self, id: i64) -> Result<(), AppError> {
        let model: user::Model = user::Entity::find()
            .filter(user::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        let mut am: user::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;

        user_role::Entity::delete_many()
            .filter(user_role::Column::UserId.eq(id))
            .exec(&self.db)
            .await?;

        Ok(())
    }

    pub async fn update_status(&self, id: i64, status: i32) -> Result<(), AppError> {
        let model: user::Model = user::Entity::find()
            .filter(user::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        let mut active: user::ActiveModel = model.into();
        active.status = Set(status);
        active.update(&self.db).await?;
        Ok(())
    }

    pub async fn assign_roles(&self, user_id: i64, role_ids: Vec<i64>) -> Result<(), AppError> {
        let _model: user::Model = user::Entity::find()
            .filter(user::Column::Id.eq(user_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        user_role::Entity::delete_many()
            .filter(user_role::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;

        for role_id in role_ids {
            user_role::ActiveModel {
                user_id: Set(user_id),
                role_id: Set(role_id),
                ..Default::default()
            }
            .insert(&self.db)
            .await?;
        }

        Ok(())
    }

    pub async fn get_role_ids(&self, user_id: i64) -> Result<Vec<i64>, AppError> {
        let roles = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(user_id))
            .all(&self.db)
            .await?;

        Ok(roles.into_iter().map(|r| r.role_id).collect())
    }
}
