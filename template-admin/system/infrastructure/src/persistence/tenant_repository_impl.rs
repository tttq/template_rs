use async_trait::async_trait;
use common::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait};
use summer_sea_orm::DbConn;
use system_entity::tenant;
use system_domain::TenantRepository;

pub struct TenantRepositoryImpl {
    db: DbConn,
}

impl TenantRepositoryImpl {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

#[async_trait]
impl TenantRepository for TenantRepositoryImpl {
    async fn find_by_id(&self, id: i64) -> Result<Option<tenant::Model>, AppError> {
        Ok(tenant::Entity::find()
            .filter(tenant::Column::Id.eq(id))
            .one(&self.db)
            .await?)
    }

    async fn find_by_code(&self, code: &str) -> Result<Option<tenant::Model>, AppError> {
        Ok(tenant::Entity::find()
            .filter(tenant::Column::TenantCode.eq(code))
            .one(&self.db)
            .await?)
    }

    async fn find_all(&self) -> Result<Vec<tenant::Model>, AppError> {
        Ok(tenant::Entity::find().all(&self.db).await?)
    }

    async fn create(&self, model: tenant::ActiveModel) -> Result<tenant::Model, AppError> {
        Ok(model.insert(&self.db).await?)
    }

    async fn update(&self, model: tenant::ActiveModel) -> Result<tenant::Model, AppError> {
        Ok(model.update(&self.db).await?)
    }

    async fn delete(&self, id: i64) -> Result<(), AppError> {
        let model = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("租户不存在".to_string()))?;
        model.delete(&self.db).await?;
        Ok(())
    }
}