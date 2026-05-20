use async_trait::async_trait;
use common::error::AppError;
use system_entity::dict_type;

#[async_trait]
pub trait DictTypeRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<Option<dict_type::Model>, AppError>;
    async fn find_by_type(&self, dict_type: &str) -> Result<Option<dict_type::Model>, AppError>;
    async fn find_all(&self) -> Result<Vec<dict_type::Model>, AppError>;
    async fn create(&self, model: dict_type::ActiveModel) -> Result<dict_type::Model, AppError>;
    async fn update(&self, model: dict_type::ActiveModel) -> Result<dict_type::Model, AppError>;
    async fn delete(&self, id: i64) -> Result<(), AppError>;
}