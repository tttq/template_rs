use async_trait::async_trait;
use common::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{QueryFilter, ColumnTrait};
use sea_orm_ext::DbConn;
use system_entity::role_menu;
use system_domain::RoleMenuRepository;

pub struct RoleMenuRepositoryImpl {
    db: DbConn,
}

impl RoleMenuRepositoryImpl {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

#[async_trait]
impl RoleMenuRepository for RoleMenuRepositoryImpl {
    async fn find_by_role_id(&self, role_id: String) -> Result<Vec<role_menu::Model>, AppError> {
        Ok(role_menu::Entity::find()
            .filter(role_menu::Column::RoleId.eq(role_id))
            .all(&self.db)
            .await?)
    }

    async fn create(&self, model: role_menu::ActiveModel) -> Result<role_menu::Model, AppError> {
        Ok(model.insert(&self.db).await?)
    }

    async fn delete_by_role_id(&self, role_id: String) -> Result<(), AppError> {
        role_menu::Entity::delete_many()
            .filter(role_menu::Column::RoleId.eq(role_id))
            .exec(&self.db)
            .await?;
        Ok(())
    }
}