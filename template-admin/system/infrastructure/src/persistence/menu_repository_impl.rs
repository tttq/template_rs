use async_trait::async_trait;
use common::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use summer_sea_orm::DbConn;
use system_entity::menu;
use system_domain::MenuRepository;

pub struct MenuRepositoryImpl {
    db: DbConn,
}

impl MenuRepositoryImpl {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

#[async_trait]
impl MenuRepository for MenuRepositoryImpl {
    async fn find_by_id(&self, id: i64) -> Result<Option<menu::Model>, AppError> {
        Ok(menu::Entity::find()
            .filter(menu::Column::Id.eq(id))
            .one(&self.db)
            .await?)
    }

    async fn find_all(&self) -> Result<Vec<menu::Model>, AppError> {
        Ok(menu::Entity::find()
            .order_by_asc(menu::Column::SortOrder)
            .all(&self.db)
            .await?)
    }

    async fn find_by_parent_id(&self, parent_id: i64) -> Result<Vec<menu::Model>, AppError> {
        Ok(menu::Entity::find()
            .filter(menu::Column::ParentId.eq(parent_id))
            .order_by_asc(menu::Column::SortOrder)
            .all(&self.db)
            .await?)
    }

    async fn create(&self, model: menu::ActiveModel) -> Result<menu::Model, AppError> {
        Ok(model.insert(&self.db).await?)
    }

    async fn update(&self, model: menu::ActiveModel) -> Result<menu::Model, AppError> {
        Ok(model.update(&self.db).await?)
    }

    async fn delete(&self, id: i64) -> Result<(), AppError> {
        let model = menu::Entity::find()
            .filter(menu::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("菜单不存在".to_string()))?;
        model.delete(&self.db).await?;
        Ok(())
    }
}