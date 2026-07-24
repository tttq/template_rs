use async_trait::async_trait;
use common::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{QueryFilter, ColumnTrait, ActiveValue::Set};
use sea_orm_ext::DbConn;
use system_entity::user;
use system_domain::UserRepository;

pub struct UserRepositoryImpl {
    db: DbConn,
}

impl UserRepositoryImpl {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_by_id(&self, id: String) -> Result<Option<user::Model>, AppError> {
        Ok(user::Entity::find()
            .filter(user::Column::Id.eq(id))
            .one(&self.db)
            .await?)
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<user::Model>, AppError> {
        Ok(user::Entity::find()
            .filter(user::Column::UserName.eq(name))
            .one(&self.db)
            .await?)
    }

    async fn find_all(&self) -> Result<Vec<user::Model>, AppError> {
        Ok(user::Entity::find().all(&self.db).await?)
    }

    async fn create(&self, model: user::ActiveModel) -> Result<user::Model, AppError> {
        Ok(model.insert(&self.db).await?)
    }

    async fn update(&self, model: user::ActiveModel) -> Result<user::Model, AppError> {
        Ok(model.update(&self.db).await?)
    }

    async fn delete(&self, id: String) -> Result<(), AppError> {
        let model = user::Entity::find()
            .filter(user::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;
        let mut am: user::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}