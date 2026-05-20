use async_trait::async_trait;
use common::error::AppError;
use system_entity::config;

#[async_trait]
pub trait ConfigRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<Option<config::Model>, AppError>;
    async fn find_by_key(&self, key: &str) -> Result<Option<config::Model>, AppError>;
    async fn find_all(&self) -> Result<Vec<config::Model>, AppError>;
    async fn create(&self, model: config::ActiveModel) -> Result<config::Model, AppError>;
    async fn update(&self, model: config::ActiveModel) -> Result<config::Model, AppError>;
    async fn delete(&self, id: i64) -> Result<(), AppError>;
}