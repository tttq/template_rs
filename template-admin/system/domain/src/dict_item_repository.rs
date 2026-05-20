use async_trait::async_trait;
use common::error::AppError;
use system_entity::dict_item;

#[async_trait]
pub trait DictItemRepository: Send + Sync {
    async fn find_by_id(&self, id: i64) -> Result<Option<dict_item::Model>, AppError>;
    async fn find_by_dict_type_id(&self, dict_type_id: i64) -> Result<Vec<dict_item::Model>, AppError>;
    async fn find_all(&self) -> Result<Vec<dict_item::Model>, AppError>;
    async fn create(&self, model: dict_item::ActiveModel) -> Result<dict_item::Model, AppError>;
    async fn update(&self, model: dict_item::ActiveModel) -> Result<dict_item::Model, AppError>;
    async fn delete(&self, id: i64) -> Result<(), AppError>;
}