use async_trait::async_trait;
use common::error::AppError;
use system_entity::menu;

#[async_trait]
pub trait MenuRepository: Send + Sync {
    async fn find_by_id(&self, id: String) -> Result<Option<menu::Model>, AppError>;
    async fn find_all(&self) -> Result<Vec<menu::Model>, AppError>;
    async fn find_by_parent_id(&self, parent_id: String) -> Result<Vec<menu::Model>, AppError>;
    async fn create(&self, model: menu::ActiveModel) -> Result<menu::Model, AppError>;
    async fn update(&self, model: menu::ActiveModel) -> Result<menu::Model, AppError>;
    async fn delete(&self, id: String) -> Result<(), AppError>;
}