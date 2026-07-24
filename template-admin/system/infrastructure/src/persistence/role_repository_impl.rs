use async_trait::async_trait;
use common::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{QueryFilter, ColumnTrait, ActiveValue::Set};
use sea_orm_ext::DbConn;
use system_entity::role;
use system_domain::RoleRepository;

pub struct RoleRepositoryImpl {
    db: DbConn,
}

impl RoleRepositoryImpl {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RoleRepository for RoleRepositoryImpl {
    async fn find_by_id(&self, id: String) -> Result<Option<role::Model>, AppError> {
        Ok(role::Entity::find()
            .filter(role::Column::Id.eq(id))
            .one(&self.db)
            .await?)
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<role::Model>, AppError> {
        Ok(role::Entity::find()
            .filter(role::Column::RoleCode.eq(code))
            .one(&self.db)
            .await?)
    }

    async fn find_all(&self) -> Result<Vec<role::Model>, AppError> {
        Ok(role::Entity::find().all(&self.db).await?)
    }

    async fn create(&self, model: role::ActiveModel) -> Result<role::Model, AppError> {
        Ok(model.insert(&self.db).await?)
    }

    async fn update(&self, model: role::ActiveModel) -> Result<role::Model, AppError> {
        Ok(model.update(&self.db).await?)
    }

    async fn delete(&self, id: String) -> Result<(), AppError> {
        let model = role::Entity::find()
            .filter(role::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("角色不存在".to_string()))?;
        let mut am: role::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}