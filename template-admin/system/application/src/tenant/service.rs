use common::error::AppError;
use common::pagination::{PageQuery, PageResult};
use summer::plugin::service::Service;
use summer_sea_orm::DbConn;
use system_entity::tenant;
use sea_orm::{QueryFilter, ColumnTrait, PaginatorTrait, ActiveValue::Set};
use sea_orm::prelude::*;

use super::dto::{CreateTenantDto, UpdateTenantDto, TenantVo};

#[derive(Clone, Service)]
pub struct TenantAppService {
    #[inject(component)]
    db: DbConn,
}

impl TenantAppService {
    pub async fn list(&self, query: PageQuery) -> Result<PageResult<TenantVo>, AppError> {
        let paginator = tenant::Entity::find()
            .paginate(&self.db, query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<tenant::Model> = paginator.fetch_page(query.page - 1).await?;

        let vos: Vec<TenantVo> = items.into_iter().map(TenantVo::from).collect();
        Ok(PageResult::new(vos, total, query.page, query.page_size))
    }

    pub async fn get_by_id(&self, id: i64) -> Result<TenantVo, AppError> {
        let model: tenant::Model = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("租户不存在".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateTenantDto) -> Result<TenantVo, AppError> {
        let existing: Option<tenant::Model> = tenant::Entity::find()
            .filter(tenant::Column::TenantCode.eq(&dto.tenant_code))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("租户编码已存在".to_string()));
        }

        let active_model = dto.into_active_model();
        let model = active_model.insert(&self.db).await?;
        Ok(model.into())
    }

    pub async fn update(&self, dto: UpdateTenantDto) -> Result<TenantVo, AppError> {
        let active_model = dto.into_active_model();
        let model = active_model.update(&self.db).await?;
        Ok(model.into())
    }

    pub async fn delete(&self, id: i64) -> Result<(), AppError> {
        let model: tenant::Model = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("租户不存在".to_string()))?;

        let mut am: tenant::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}
