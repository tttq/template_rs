use common::error::AppError;
use common::pagination::{PageQuery, PageResult};
use common::tenant_db::get_effective_db;
use summer::plugin::service::Service;
use summer_sea_orm::DbConn;
use system_entity::config;
use sea_orm::{QueryFilter, ColumnTrait, PaginatorTrait, ActiveValue::Set};
use sea_orm::prelude::*;

use super::dto::{CreateConfigDto, UpdateConfigDto, ConfigVo};

#[derive(Clone, Service)]
pub struct ConfigAppService {
    #[inject(component)]
    db: DbConn,
}

impl ConfigAppService {
    pub async fn list(&self, query: PageQuery) -> Result<PageResult<ConfigVo>, AppError> {
        let db = get_effective_db(&self.db).await?;

        let paginator = config::Entity::find()
            .paginate(&db, query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<config::Model> = paginator.fetch_page(query.page - 1).await?;

        let vos: Vec<ConfigVo> = items.into_iter().map(ConfigVo::from).collect();
        Ok(PageResult::new(vos, total, query.page, query.page_size))
    }

    pub async fn get_by_id(&self, id: String) -> Result<ConfigVo, AppError> {
        let db = get_effective_db(&self.db).await?;

        let model: config::Model = config::Entity::find()
            .filter(config::Column::Id.eq(&id))
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("配置不存在".to_string()))?;

        Ok(model.into())
    }

    pub async fn get_by_key(&self, key: &str) -> Result<ConfigVo, AppError> {
        let db = get_effective_db(&self.db).await?;

        let model: config::Model = config::Entity::find()
            .filter(config::Column::ConfigKey.eq(key))
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("配置不存在".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateConfigDto) -> Result<ConfigVo, AppError> {
        let db = get_effective_db(&self.db).await?;

        let existing: Option<config::Model> = config::Entity::find()
            .filter(config::Column::ConfigKey.eq(&dto.config_key))
            .one(&db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("配置键已存在".to_string()));
        }

        let active_model = dto.into_active_model();
        let model = active_model.insert(&db).await?;
        Ok(model.into())
    }

    pub async fn update(&self, dto: UpdateConfigDto) -> Result<ConfigVo, AppError> {
        let db = get_effective_db(&self.db).await?;

        let active_model = dto.into_active_model();
        let model = active_model.update(&db).await?;
        Ok(model.into())
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {
        let db = get_effective_db(&self.db).await?;

        let model: config::Model = config::Entity::find()
            .filter(config::Column::Id.eq(&id))
            .one(&db)
            .await?
            .ok_or_else(|| AppError::NotFound("配置不存在".to_string()))?;

        let mut am: config::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&db).await?;
        Ok(())
    }
}