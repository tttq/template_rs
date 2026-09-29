use common::error::AppError;
use common::pagination::PageResult;
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use system_entity::dict_type;
use sea_orm::{QueryFilter, ColumnTrait, PaginatorTrait, ActiveValue::Set};
use sea_orm::prelude::*;
use chrono::Utc;
use fast_excel::ExportTaskContext;
use summer::App;
use summer::plugin::ComponentRegistry;

use super::dto::{CreateDictTypeDto, UpdateDictTypeDto, DictTypeVo, DictTypeQuery, DictTypeExportQuery};

#[derive(Clone, Service)]
pub struct DictTypeAppService {
    #[inject(component)]
    db: DbConn,
}

impl DictTypeAppService {
    pub async fn list(&self, query: DictTypeQuery) -> Result<PageResult<DictTypeVo>, AppError> {
        let mut select = dict_type::Entity::find();

        if let Some(v) = &query.dict_name {
            if !v.is_empty() {
                select = select.filter(dict_type::Column::DictName.contains(v));
            }
        }
        if let Some(v) = &query.dict_type {
            if !v.is_empty() {
                select = select.filter(dict_type::Column::DictType.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(dict_type::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(dict_type::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(dict_type::Column::CreateBy.contains(v));
            }
        }

        let paginator = select.paginate(&self.db, query.page_query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<dict_type::Model> = paginator.fetch_page(query.page_query.page - 1).await?;

        let vos: Vec<DictTypeVo> = items.into_iter().map(DictTypeVo::from).collect();
        Ok(PageResult::new(vos, total, query.page_query.page, query.page_query.page_size))
    }

    /// 导出字典类型列表
    pub async fn export(&self, query: DictTypeExportQuery) -> Result<Vec<DictTypeVo>, AppError> {
        let items: Vec<dict_type::Model> = Self::export_select(&query).all(&self.db).await?;
        Ok(items.into_iter().map(DictTypeVo::from).collect())
    }

    /// 可导出行数（同步/异步分流判定；与 `export` 同口径，只 COUNT 不拉数据）
    pub async fn export_count(&self, query: &DictTypeExportQuery) -> Result<u64, AppError> {
        Self::export_select(query).count(&self.db).await.map_err(AppError::from)
    }

    /// 导出筛选条件（列表导出与行数统计共用同一口径）
    fn export_select(query: &DictTypeExportQuery) -> sea_orm::Select<dict_type::Entity> {
        let mut select = dict_type::Entity::find();

        if let Some(v) = &query.dict_name {
            if !v.is_empty() {
                select = select.filter(dict_type::Column::DictName.contains(v));
            }
        }
        if let Some(v) = &query.dict_type {
            if !v.is_empty() {
                select = select.filter(dict_type::Column::DictType.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(dict_type::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(dict_type::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(dict_type::Column::CreateBy.contains(v));
            }
        }
        if let Some(ids_str) = &query.ids {
            if !ids_str.is_empty() {
                let ids: Vec<String> = ids_str.split(',').map(|s| s.trim().to_string()).collect();
                select = select.filter(dict_type::Column::Id.is_in(ids));
            }
        }

        select
    }

    pub async fn get_by_id(&self, id: String) -> Result<DictTypeVo, AppError> {

        let model: dict_type::Model = dict_type::Entity::find()
            .filter(dict_type::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@dict_type_not_found".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateDictTypeDto) -> Result<DictTypeVo, AppError> {

        let existing: Option<dict_type::Model> = dict_type::Entity::find()
            .filter(dict_type::Column::DictType.eq(&dto.dict_type))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("@dict_type_exists".to_string()));
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
            .ok_or_else(|| AppError::NotFound("@dict_type_not_found".to_string()))?;

        let mut am: dict_type::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}

// ===================== 列表导出（同步表头/行映射 + 异步导出注册） =====================

/// 字典类型导出表头
pub const DICT_HEADERS: &[&str] = &[
    "字典名称", "字典类型", "状态", "备注", "创建时间", "创建人", "创建人ID", "修改时间", "修改人",
    "修改人ID",
];

fn ts(t: chrono::DateTime<chrono::Utc>) -> String {
    t.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 字典类型列表 → 导出行
pub fn dict_rows(vos: &[DictTypeVo]) -> Vec<Vec<String>> {
    vos.iter()
        .map(|v| {
            vec![
                v.dict_name.clone(),
                v.dict_type.clone(),
                if v.status == 1 { "启用" } else { "禁用" }.to_string(),
                v.remark.clone().unwrap_or_default(),
                ts(v.create_time),
                v.create_by.clone().unwrap_or_default(),
                v.create_id.clone().unwrap_or_default(),
                ts(v.update_time),
                v.update_by.clone().unwrap_or_default(),
                v.update_id.clone().unwrap_or_default(),
            ]
        })
        .collect()
}

/// 异步导出（> 10 万条）：导出中心按 `task_type = "dict"` 直接调用。
async fn dict_export_rows(ctx: ExportTaskContext) -> Result<Vec<Vec<String>>, AppError> {
    let service = App::global()
        .try_get_component::<DictTypeAppService>()
        .map_err(|e| AppError::Internal(format!("字典类型服务组件未就绪：{e}")))?;
    let query: DictTypeExportQuery = serde_json::from_value(ctx.query.clone())
        .map_err(|e| AppError::BadRequest(format!("导出参数不合法: {e}")))?;
    let vos = service.export(query).await?;
    Ok(dict_rows(&vos))
}

// 注册即完成：链接期自动登记，无执行器、无 install()、无需 main.rs 聚合。
fast_excel::export_task! {
    task_type = "dict",
    sheet_name = "字典类型",
    headers = DICT_HEADERS,
    rows = dict_export_rows,
}
