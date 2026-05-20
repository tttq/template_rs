use async_trait::async_trait;
use common::error::AppError;
use system_entity::tenant;

#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<Option<tenant::Model>, AppError>;
    async fn find_by_code(&self, code: &str) -> Result<Option<tenant::Model>, AppError>;
    async fn find_all(&self) -> Result<Vec<tenant::Model>, AppError>;
    async fn create(&self, model: tenant::ActiveModel) -> Result<tenant::Model, AppError>;
    async fn update(&self, model: tenant::ActiveModel) -> Result<tenant::Model, AppError>;
    async fn delete(&self, id: i64) -> Result<(), AppError>;
}