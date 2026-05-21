use async_trait::async_trait;
use common::error::AppError;
use system_entity::role_menu;

#[async_trait]
pub trait RoleMenuRepository: Send + Sync {
    async fn find_by_role_id(&self, role_id: String) -> Result<Vec<role_menu::Model>, AppError>;
    async fn create(&self, model: role_menu::ActiveModel) -> Result<role_menu::Model, AppError>;
    async fn delete_by_role_id(&self, role_id: String) -> Result<(), AppError>;
}