use async_trait::async_trait;
use common::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, DeleteMany};
use summer_sea_orm::DbConn;
use system_entity::user_role;
use system_domain::UserRoleRepository;

pub struct UserRoleRepositoryImpl {
    db: DbConn,
}

impl UserRoleRepositoryImpl {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRoleRepository for UserRoleRepositoryImpl {
    async fn find_by_user_id(&self, user_id: String) -> Result<Vec<user_role::Model>, AppError> {
        Ok(user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(user_id))
            .all(&self.db)
            .await?)
    }

    async fn find_by_role_id(&self, role_id: String) -> Result<Vec<user_role::Model>, AppError> {
        Ok(user_role::Entity::find()
            .filter(user_role::Column::RoleId.eq(role_id))
            .all(&self.db)
            .await?)
    }

    async fn create(&self, model: user_role::ActiveModel) -> Result<user_role::Model, AppError> {
        Ok(model.insert(&self.db).await?)
    }

    async fn delete_by_user_id(&self, user_id: String) -> Result<(), AppError> {
        user_role::Entity::delete_many()
            .filter(user_role::Column::UserId.eq(user_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }
}