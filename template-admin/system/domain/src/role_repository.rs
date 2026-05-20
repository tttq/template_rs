use async_trait::async_trait;
use common::error::AppError;
use system_entity::role;

#[async_trait]
pub trait RoleRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<Option<role::Model>, AppError>;
    async fn find_by_code(&self, code: &str) -> Result<Option<role::Model>, AppError>;
    async fn find_all(&self) -> Result<Vec<role::Model>, AppError>;
    async fn create(&self, model: role::ActiveModel) -> Result<role::Model, AppError>;
    async fn update(&self, model: role::ActiveModel) -> Result<role::Model, AppError>;
    async fn delete(&self, id: i64) -> Result<(), AppError>;
}