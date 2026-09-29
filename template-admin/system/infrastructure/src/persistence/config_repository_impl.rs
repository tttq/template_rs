use async_trait::async_trait;
use common::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{QueryFilter, ColumnTrait, ActiveValue::Set};
use sea_orm_ext::DbConn;
use system_entity::config;
use system_domain::ConfigRepository;

pub struct ConfigRepositoryImpl {
    db: DbConn,
}

impl ConfigRepositoryImpl {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ConfigRepository for ConfigRepositoryImpl {
    async fn find_by_id(&self, id: String) -> Result<Option<config::Model>, AppError> {
        Ok(config::Entity::find()
            .filter(config::Column::Id.eq(id))
            .one(&self.db)
            .await?)
    }

    async fn find_by_key(&self, key: &str) -> Result<Option<config::Model>, AppError> {
        Ok(config::Entity::find()
            .filter(config::Column::ConfigKey.eq(key))
            .one(&self.db)
            .await?)
    }

    async fn find_all(&self) -> Result<Vec<config::Model>, AppError> {
        Ok(config::Entity::find().all(&self.db).await?)
    }

    async fn create(&self, model: config::ActiveModel) -> Result<config::Model, AppError> {
        Ok(model.insert(&self.db).await?)
    }

    async fn update(&self, model: config::ActiveModel) -> Result<config::Model, AppError> {
        Ok(model.update(&self.db).await?)
    }

    async fn delete(&self, id: String) -> Result<(), AppError> {
        let model = config::Entity::find()
            .filter(config::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@config_not_found".to_string()))?;
        let mut am: config::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}