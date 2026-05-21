use async_trait::async_trait;
use common::error::AppError;
use system_entity::dept;

#[async_trait]
pub trait DeptRepository: Send + Sync {
    async fn find_by_id(&self, id: String) -> Result<Option<dept::Model>, AppError>;
    async fn find_all(&self) -> Result<Vec<dept::Model>, AppError>;
    async fn find_by_parent_id(&self, parent_id: String) -> Result<Vec<dept::Model>, AppError>;
    async fn create(&self, model: dept::ActiveModel) -> Result<dept::Model, AppError>;
    async fn update(&self, model: dept::ActiveModel) -> Result<dept::Model, AppError>;
    async fn delete(&self, id: String) -> Result<(), AppError>;
}