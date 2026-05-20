use common::error::AppError;
use common::pagination::{PageQuery, PageResult};
use summer::plugin::service::Service;
use summer_sea_orm::DbConn;
use system_entity::{dict_item, dict_type};
use sea_orm::{QueryFilter, ColumnTrait, PaginatorTrait, QueryOrder, ActiveValue::Set};
use sea_orm::prelude::*;

use super::dto::{CreateDictItemDto, UpdateDictItemDto, DictItemVo};

#[derive(Clone, Service)]
pub struct DictItemAppService {
    #[inject(component)]
    db: DbConn,
}

impl DictItemAppService {
    pub async fn list(&self, query: PageQuery) -> Result<PageResult<DictItemVo>, AppError> {
        let paginator = dict_item::Entity::find()
            .paginate(&self.db, query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<dict_item::Model> = paginator.fetch_page(query.page - 1).await?;

        let vos: Vec<DictItemVo> = items.into_iter().map(DictItemVo::from).collect();
        Ok(PageResult::new(vos, total, query.page, query.page_size))
    }

    pub async fn get_by_dict_type_id(&self, dict_type_id: i64) -> Result<Vec<DictItemVo>, AppError> {
        let items: Vec<dict_item::Model> = dict_item::Entity::find()
            .filter(dict_item::Column::DictTypeId.eq(dict_type_id))
            .order_by_asc(dict_item::Column::SortOrder)
            .all(&self.db)
            .await?;

        let vos: Vec<DictItemVo> = items.into_iter().map(DictItemVo::from).collect();
        Ok(vos)
    }

    pub async fn get_by_dict_type_code(&self, dict_type_code: String) -> Result<Vec<DictItemVo>, AppError> {
        let dict_type_model: dict_type::Model = dict_type::Entity::find()
            .filter(dict_type::Column::DictType.eq(&dict_type_code))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("字典类型 {} 不存在", dict_type_code)))?;

        let items: Vec<dict_item::Model> = dict_item::Entity::find()
            .filter(dict_item::Column::DictTypeId.eq(dict_type_model.id))
            .order_by_asc(dict_item::Column::SortOrder)
            .all(&self.db)
            .await?;

        let vos: Vec<DictItemVo> = items.into_iter().map(DictItemVo::from).collect();
        Ok(vos)
    }

    pub async fn get_by_id(&self, id: i64) -> Result<DictItemVo, AppError> {
        let model: dict_item::Model = dict_item::Entity::find()
            .filter(dict_item::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("字典项不存在".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateDictItemDto) -> Result<DictItemVo, AppError> {
        let active_model = dto.into_active_model();
        let model = active_model.insert(&self.db).await?;
        Ok(model.into())
    }

    pub async fn update(&self, dto: UpdateDictItemDto) -> Result<DictItemVo, AppError> {
        let active_model = dto.into_active_model();
        let model = active_model.update(&self.db).await?;
        Ok(model.into())
    }

    pub async fn delete(&self, id: i64) -> Result<(), AppError> {
        let model: dict_item::Model = dict_item::Entity::find()
            .filter(dict_item::Column::Id.eq(id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("字典项不存在".to_string()))?;

        let mut am: dict_item::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}
