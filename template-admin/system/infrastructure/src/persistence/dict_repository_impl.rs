use async_trait::async_trait;
use common::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use summer_sea_orm::DbConn;
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
    async fn find_by_id(&self, id: i64) -> Result<Option<dict_type::Model>, AppError> {
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

    async fn delete(&self, id: i64) -> Result<(), AppError> {
        let model = dict_type::Entity::find()
            .filter(dict_type::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("字典类型不存在".to_string()))?;
        model.delete(&self.db).await?;
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
    async fn find_by_id(&self, id: i64) -> Result<Option<dict_item::Model>, AppError> {
        Ok(dict_item::Entity::find()
            .filter(dict_item::Column::Id.eq(id))
            .one(&self.db)
            .await?)
    }

    async fn find_by_dict_type_id(&self, dict_type_id: i64) -> Result<Vec<dict_item::Model>, AppError> {
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

    async fn delete(&self, id: i64) -> Result<(), AppError> {
        let model = dict_item::Entity::find()
            .filter(dict_item::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("字典项不存在".to_string()))?;
        model.delete(&self.db).await?;
        Ok(())
    }
}