use async_trait::async_trait;
use common::error::AppError;
use sea_orm::prelude::*;
use system_entity::user;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: String) -> Result<Option<user::Model>, AppError>;
    async fn find_by_name(&self, name: &str) -> Result<Option<user::Model>, AppError>;
    async fn find_all(&self) -> Result<Vec<user::Model>, AppError>;
    async fn create(&self, model: user::ActiveModel) -> Result<user::Model, AppError>;
    async fn update(&self, model: user::ActiveModel) -> Result<user::Model, AppError>;
    async fn delete(&self, id: String) -> Result<(), AppError>;
}