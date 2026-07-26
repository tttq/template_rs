use common::error::AppError;
use common::pagination::{PageQuery, PageResult};
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use system_entity::dict_type;
use sea_orm::{QueryFilter, ColumnTrait, PaginatorTrait, ActiveValue::Set};
use sea_orm::prelude::*;

use super::dto::{CreateDictTypeDto, UpdateDictTypeDto, DictTypeVo};

#[derive(Clone, Service)]
pub struct DictTypeAppService {
    #[inject(component)]
    db: DbConn,
}

impl DictTypeAppService {
    pub async fn list(&self, query: PageQuery) -> Result<PageResult<DictTypeVo>, AppError> {

        let paginator = dict_type::Entity::find()
            .paginate(&self.db, query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<dict_type::Model> = paginator.fetch_page(query.page - 1).await?;

        let vos: Vec<DictTypeVo> = items.into_iter().map(DictTypeVo::from).collect();
        Ok(PageResult::new(vos, total, query.page, query.page_size))
    }

    pub async fn get_by_id(&self, id: String) -> Result<DictTypeVo, AppError> {

        let model: dict_type::Model = dict_type::Entity::find()
            .filter(dict_type::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("字典类型不存在".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateDictTypeDto) -> Result<DictTypeVo, AppError> {

        let existing: Option<dict_type::Model> = dict_type::Entity::find()
            .filter(dict_type::Column::DictType.eq(&dto.dict_type))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("字典类型已存在".to_string()));
        }

        let active_model = dto.into_active_model();
        let model = active_model.insert(&self.db).await?;
        Ok(model.into())
    }

    pub async fn update(&self, dto: UpdateDictTypeDto) -> Result<DictTypeVo, AppError> {

        let active_model = dto.into_active_model();
        let model = active_model.update(&self.db).await?;
        Ok(model.into())
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {

        let model: dict_type::Model = dict_type::Entity::find()
            .filter(dict_type::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("字典类型不存在".to_string()))?;

        let mut am: dict_type::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}