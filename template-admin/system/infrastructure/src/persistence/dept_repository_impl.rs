use async_trait::async_trait;
use common::error::AppError;
use sea_orm::prelude::*;
use sea_orm::{QueryFilter, ColumnTrait, QueryOrder, ActiveValue::Set};
use sea_orm_ext::DbConn;
use system_entity::dept;
use system_domain::DeptRepository;

pub struct DeptRepositoryImpl {
    db: DbConn,
}

impl DeptRepositoryImpl {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

#[async_trait]
impl DeptRepository for DeptRepositoryImpl {
    async fn find_by_id(&self, id: String) -> Result<Option<dept::Model>, AppError> {
        Ok(dept::Entity::find()
            .filter(dept::Column::Id.eq(id))
            .one(&self.db)
            .await?)
    }

    async fn find_all(&self) -> Result<Vec<dept::Model>, AppError> {
        Ok(dept::Entity::find()
            .order_by_asc(dept::Column::DeptSort)
            .all(&self.db)
            .await?)
    }

    async fn find_by_parent_id(&self, parent_id: String) -> Result<Vec<dept::Model>, AppError> {
        Ok(dept::Entity::find()
            .filter(dept::Column::ParentId.eq(parent_id))
            .order_by_asc(dept::Column::DeptSort)
            .all(&self.db)
            .await?)
    }

    async fn create(&self, model: dept::ActiveModel) -> Result<dept::Model, AppError> {
        Ok(model.insert(&self.db).await?)
    }

    async fn update(&self, model: dept::ActiveModel) -> Result<dept::Model, AppError> {
        Ok(model.update(&self.db).await?)
    }

    async fn delete(&self, id: String) -> Result<(), AppError> {
        let model = dept::Entity::find()
            .filter(dept::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@dept_not_found".to_string()))?;
        let mut am: dept::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}