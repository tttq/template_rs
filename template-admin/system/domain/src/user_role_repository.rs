use async_trait::async_trait;
use common::error::AppError;
use system_entity::user_role;

#[async_trait]
pub trait UserRoleRepository: Send + Sync {
    async fn find_by_user_id(&self, user_id: String) -> Result<Vec<user_role::Model>, AppError>;
    async fn find_by_role_id(&self, role_id: String) -> Result<Vec<user_role::Model>, AppError>;
    async fn create(&self, model: user_role::ActiveModel) -> Result<user_role::Model, AppError>;
    async fn delete_by_user_id(&self, user_id: String) -> Result<(), AppError>;
}