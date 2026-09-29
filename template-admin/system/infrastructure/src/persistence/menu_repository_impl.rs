use async_trait::async_trait;
use common::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{QueryFilter, ColumnTrait, QueryOrder, ActiveValue::Set};
use sea_orm_ext::DbConn;
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
    async fn find_by_id(&self, id: String) -> Result<Option<menu::Model>, AppError> {
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

    async fn find_by_parent_id(&self, parent_id: String) -> Result<Vec<menu::Model>, AppError> {
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

    async fn delete(&self, id: String) -> Result<(), AppError> {
        let model = menu::Entity::find()
            .filter(menu::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@menu_not_found".to_string()))?;
        let mut am: menu::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}