use common::error::AppError;
use common::pagination::{PageQuery, PageResult};
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use system_entity::{dict_item, dict_type};
use sea_orm::{QueryFilter, ColumnTrait, PaginatorTrait, QueryOrder, ActiveValue::Set};
use sea_orm::prelude::*;

use super::dto::{CreateDictItemDto, DictItemVo, DictOptionVo, UpdateDictItemDto};

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

    pub async fn get_by_dict_type_id(&self, dict_type_id: String) -> Result<Vec<DictItemVo>, AppError> {

        let items: Vec<dict_item::Model> = dict_item::Entity::find()
            .filter(dict_item::Column::DictTypeId.eq(&dict_type_id))
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
            .ok_or_else(|| AppError::NotFound(format!("@dict_type_not_found_code:{}", dict_type_code)))?;

        let items: Vec<dict_item::Model> = dict_item::Entity::find()
            .filter(dict_item::Column::DictTypeId.eq(&dict_type_model.id))
            .order_by_asc(dict_item::Column::SortOrder)
            .all(&self.db)
            .await?;

        let vos: Vec<DictItemVo> = items.into_iter().map(DictItemVo::from).collect();
        Ok(vos)
    }

    /// 按字典类型编码取**启用中**的字典项选项（仅 label/value）
    ///
    /// 与 `get_by_dict_type_code` 的区别：
    /// - handler 侧仅需登录（`#[sa_check_login]`），业务页面无需 `dict:list` 权限
    /// - 只返回启用项与展示所需字段，不暴露字典配置细节
    /// - 字典类型缺失或已停用时返回空列表而非报错，避免业务页面因未配置字典而整体失败
    pub async fn options_by_type_code(
        &self,
        dict_type_code: &str,
    ) -> Result<Vec<DictOptionVo>, AppError> {
        let Some(dict_type_model) = dict_type::Entity::find()
            .filter(dict_type::Column::DictType.eq(dict_type_code))
            .filter(dict_type::Column::Status.eq(1))
            .one(&self.db)
            .await?
        else {
            return Ok(Vec::new());
        };

        let items: Vec<dict_item::Model> = dict_item::Entity::find()
            .filter(dict_item::Column::DictTypeId.eq(&dict_type_model.id))
            .filter(dict_item::Column::Status.eq(1))
            .order_by_asc(dict_item::Column::SortOrder)
            .all(&self.db)
            .await?;

        Ok(items
            .into_iter()
            .map(|m| DictOptionVo {
                label: m.dict_label,
                value: m.dict_value,
            })
            .collect())
    }

    pub async fn get_by_id(&self, id: String) -> Result<DictItemVo, AppError> {

        let model: dict_item::Model = dict_item::Entity::find()
            .filter(dict_item::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@dict_item_not_found".to_string()))?;

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

    pub async fn delete(&self, id: String) -> Result<(), AppError> {

        let model: dict_item::Model = dict_item::Entity::find()
            .filter(dict_item::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@dict_item_not_found".to_string()))?;

        let mut am: dict_item::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}