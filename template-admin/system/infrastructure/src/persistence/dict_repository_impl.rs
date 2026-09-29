use async_trait::async_trait;
use common::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{QueryFilter, ColumnTrait, QueryOrder, ActiveValue::Set};
use sea_orm_ext::DbConn;
use system_entity::{dict_type, dict_item};
use system_domain::{DictTypeRepository, DictItemRepository};

pub struct DictTypeRepositoryImpl {
    db: DbConn,
}

impl DictTypeRepositoryImpl {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

#[async_trait]
impl DictTypeRepository for DictTypeRepositoryImpl {
    async fn find_by_id(&self, id: String) -> Result<Option<dict_type::Model>, AppError> {
        Ok(dict_type::Entity::find()
            .filter(dict_type::Column::Id.eq(id))
            .one(&self.db)
            .await?)
    }

    async fn find_by_type(&self, dict_type_str: &str) -> Result<Option<dict_type::Model>, AppError> {
        Ok(dict_type::Entity::find()
            .filter(dict_type::Column::DictType.eq(dict_type_str))
            .one(&self.db)
            .await?)
    }

    async fn find_all(&self) -> Result<Vec<dict_type::Model>, AppError> {
        Ok(dict_type::Entity::find().all(&self.db).await?)
    }

    async fn create(&self, model: dict_type::ActiveModel) -> Result<dict_type::Model, AppError> {
        Ok(model.insert(&self.db).await?)
    }

    async fn update(&self, model: dict_type::ActiveModel) -> Result<dict_type::Model, AppError> {
        Ok(model.update(&self.db).await?)
    }

    async fn delete(&self, id: String) -> Result<(), AppError> {
        let model = dict_type::Entity::find()
            .filter(dict_type::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@dict_type_not_found".to_string()))?;
        let mut am: dict_type::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}

pub struct DictItemRepositoryImpl {
    db: DbConn,
}

impl DictItemRepositoryImpl {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

#[async_trait]
impl DictItemRepository for DictItemRepositoryImpl {
    async fn find_by_id(&self, id: String) -> Result<Option<dict_item::Model>, AppError> {
        Ok(dict_item::Entity::find()
            .filter(dict_item::Column::Id.eq(id))
            .one(&self.db)
            .await?)
    }

    async fn find_by_dict_type_id(&self, dict_type_id: String) -> Result<Vec<dict_item::Model>, AppError> {
        Ok(dict_item::Entity::find()
            .filter(dict_item::Column::DictTypeId.eq(dict_type_id))
            .order_by_asc(dict_item::Column::SortOrder)
            .all(&self.db)
            .await?)
    }

    async fn find_all(&self) -> Result<Vec<dict_item::Model>, AppError> {
        Ok(dict_item::Entity::find().all(&self.db).await?)
    }

    async fn create(&self, model: dict_item::ActiveModel) -> Result<dict_item::Model, AppError> {
        Ok(model.insert(&self.db).await?)
    }

    async fn update(&self, model: dict_item::ActiveModel) -> Result<dict_item::Model, AppError> {
        Ok(model.update(&self.db).await?)
    }

    async fn delete(&self, id: String) -> Result<(), AppError> {
        let model = dict_item::Entity::find()
            .filter(dict_item::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@dict_item_not_found".to_string()))?;
        let mut am: dict_item::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}