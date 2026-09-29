use common::error::AppError;
use common::pagination::PageResult;
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use system_entity::config;
use sea_orm::{QueryFilter, ColumnTrait, PaginatorTrait, ActiveValue::Set};
use sea_orm::prelude::*;
use chrono::Utc;
use fast_excel::ExportTaskContext;
use summer::App;
use summer::plugin::ComponentRegistry;

use super::dto::{CreateConfigDto, UpdateConfigDto, ConfigVo, ConfigQuery, ConfigExportQuery};

#[derive(Clone, Service)]
pub struct ConfigAppService {
    #[inject(component)]
    db: DbConn,
}

impl ConfigAppService {
    pub async fn list(&self, query: ConfigQuery) -> Result<PageResult<ConfigVo>, AppError> {
        let mut select = config::Entity::find();

        if let Some(v) = &query.config_name {
            if !v.is_empty() {
                select = select.filter(config::Column::ConfigName.contains(v));
            }
        }
        if let Some(v) = &query.config_key {
            if !v.is_empty() {
                select = select.filter(config::Column::ConfigKey.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(config::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(config::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(config::Column::CreateBy.contains(v));
            }
        }

        let paginator = select.paginate(&self.db, query.page_query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<config::Model> = paginator.fetch_page(query.page_query.page - 1).await?;

        let vos: Vec<ConfigVo> = items.into_iter().map(ConfigVo::from).collect();
        Ok(PageResult::new(vos, total, query.page_query.page, query.page_query.page_size))
    }

    /// 导出配置列表
    pub async fn export(&self, query: ConfigExportQuery) -> Result<Vec<ConfigVo>, AppError> {
        let items: Vec<config::Model> = Self::export_select(&query).all(&self.db).await?;
        Ok(items.into_iter().map(ConfigVo::from).collect())
    }

    /// 可导出行数（同步/异步分流判定；与 `export` 同口径，只 COUNT 不拉数据）
    pub async fn export_count(&self, query: &ConfigExportQuery) -> Result<u64, AppError> {
        Self::export_select(query).count(&self.db).await.map_err(AppError::from)
    }

    /// 导出筛选条件（列表导出与行数统计共用同一口径）
    fn export_select(query: &ConfigExportQuery) -> sea_orm::Select<config::Entity> {
        let mut select = config::Entity::find();

        if let Some(v) = &query.config_name {
            if !v.is_empty() {
                select = select.filter(config::Column::ConfigName.contains(v));
            }
        }
        if let Some(v) = &query.config_key {
            if !v.is_empty() {
                select = select.filter(config::Column::ConfigKey.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(config::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(config::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(config::Column::CreateBy.contains(v));
            }
        }
        if let Some(ids_str) = &query.ids {
            if !ids_str.is_empty() {
                let ids: Vec<String> = ids_str.split(',').map(|s| s.trim().to_string()).collect();
                select = select.filter(config::Column::Id.is_in(ids));
            }
        }

        select
    }

    pub async fn get_by_id(&self, id: String) -> Result<ConfigVo, AppError> {

        let model: config::Model = config::Entity::find()
            .filter(config::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@config_not_found".to_string()))?;

        Ok(model.into())
    }

    pub async fn get_by_key(&self, key: &str) -> Result<ConfigVo, AppError> {

        let model: config::Model = config::Entity::find()
            .filter(config::Column::ConfigKey.eq(key))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@config_not_found".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateConfigDto) -> Result<ConfigVo, AppError> {

        let existing: Option<config::Model> = config::Entity::find()
            .filter(config::Column::ConfigKey.eq(&dto.config_key))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("@config_key_exists".to_string()));
        }

        let active_model = dto.into_active_model();
        let model = active_model.insert(&self.db).await?;
        Ok(model.into())
    }

    pub async fn update(&self, dto: UpdateConfigDto) -> Result<ConfigVo, AppError> {

        let active_model = dto.into_active_model();
        let model = active_model.update(&self.db).await?;
        Ok(model.into())
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {

        let model: config::Model = config::Entity::find()
            .filter(config::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@config_not_found".to_string()))?;

        let mut am: config::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}

// ===================== 列表导出（同步表头/行映射 + 异步导出注册） =====================

/// 参数配置导出表头
pub const CONFIG_HEADERS: &[&str] = &[
    "配置名称", "配置键", "配置值", "配置类型", "备注", "创建时间", "创建人", "创建人ID",
    "修改时间", "修改人", "修改人ID",
];

fn ts(t: chrono::DateTime<chrono::Utc>) -> String {
    t.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 参数配置列表 → 导出行
pub fn config_rows(vos: &[ConfigVo]) -> Vec<Vec<String>> {
    vos.iter()
        .map(|v| {
            vec![
                v.config_name.clone(),
                v.config_key.clone(),
                v.config_value.clone(),
                v.config_type.clone(),
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

/// 异步导出（> 10 万条）：导出中心按 `task_type = "config"` 直接调用。
async fn config_export_rows(ctx: ExportTaskContext) -> Result<Vec<Vec<String>>, AppError> {
    let service = App::global()
        .try_get_component::<ConfigAppService>()
        .map_err(|e| AppError::Internal(format!("参数配置服务组件未就绪：{e}")))?;
    let query: ConfigExportQuery = serde_json::from_value(ctx.query.clone())
        .map_err(|e| AppError::BadRequest(format!("导出参数不合法: {e}")))?;
    let vos = service.export(query).await?;
    Ok(config_rows(&vos))
}

// 注册即完成：链接期自动登记，无执行器、无 install()、无需 main.rs 聚合。
fast_excel::export_task! {
    task_type = "config",
    sheet_name = "参数配置",
    headers = CONFIG_HEADERS,
    rows = config_export_rows,
}
