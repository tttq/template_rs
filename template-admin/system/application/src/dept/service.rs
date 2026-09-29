use common::error::AppError;
use common::pagination::PageResult;
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use system_entity::dept;
use sea_orm::{QueryFilter, ColumnTrait, QueryOrder, PaginatorTrait, ActiveValue::Set};
use sea_orm::prelude::*;
use chrono::Utc;
use fast_excel::ExportTaskContext;
use summer::App;
use summer::plugin::ComponentRegistry;

use super::dto::{CreateDeptDto, UpdateDeptDto, DeptVo, DeptQuery, DeptExportQuery};

#[derive(Clone, Service)]
pub struct DeptAppService {
    #[inject(component)]
    db: DbConn,
}

impl DeptAppService {
    pub async fn list(&self, query: DeptQuery) -> Result<PageResult<DeptVo>, AppError> {
        let mut select = dept::Entity::find();

        if let Some(v) = &query.dept_name {
            if !v.is_empty() {
                select = select.filter(dept::Column::DeptName.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(dept::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(dept::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(dept::Column::CreateBy.contains(v));
            }
        }

        select = select.order_by_asc(dept::Column::DeptSort);

        let paginator = select.paginate(&self.db, query.page_query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<dept::Model> = paginator.fetch_page(query.page_query.page - 1).await?;

        let vos: Vec<DeptVo> = items.into_iter().map(DeptVo::from).collect();
        Ok(PageResult::new(vos, total, query.page_query.page, query.page_query.page_size))
    }

    pub async fn list_tree(&self) -> Result<Vec<DeptVo>, AppError> {

        let items: Vec<dept::Model> = dept::Entity::find()
            .order_by_asc(dept::Column::DeptSort)
            .all(&self.db)
            .await?;

        let vos: Vec<DeptVo> = items.into_iter().map(DeptVo::from).collect();
        Ok(Self::build_tree(vos))
    }

    /// 导出部门列表
    pub async fn export(&self, query: DeptExportQuery) -> Result<Vec<DeptVo>, AppError> {
        let items: Vec<dept::Model> = Self::export_select(&query).all(&self.db).await?;
        Ok(items.into_iter().map(DeptVo::from).collect())
    }

    /// 可导出行数（同步/异步分流判定；与 `export` 同口径，只 COUNT 不拉数据）
    pub async fn export_count(&self, query: &DeptExportQuery) -> Result<u64, AppError> {
        Self::export_select(query).count(&self.db).await.map_err(AppError::from)
    }

    /// 导出筛选条件（列表导出与行数统计共用同一口径）
    fn export_select(query: &DeptExportQuery) -> sea_orm::Select<dept::Entity> {
        let mut select = dept::Entity::find();

        if let Some(v) = &query.dept_name {
            if !v.is_empty() {
                select = select.filter(dept::Column::DeptName.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(dept::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(dept::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(dept::Column::CreateBy.contains(v));
            }
        }
        if let Some(ids_str) = &query.ids {
            if !ids_str.is_empty() {
                let ids: Vec<String> = ids_str.split(',').map(|s| s.trim().to_string()).collect();
                select = select.filter(dept::Column::Id.is_in(ids));
            }
        }

        select = select.order_by_asc(dept::Column::DeptSort);

        select
    }

    fn build_tree(depts: Vec<DeptVo>) -> Vec<DeptVo> {
        let mut result = Vec::new();
        let mut map: std::collections::HashMap<String, Vec<DeptVo>> = std::collections::HashMap::new();

        for dept in depts {
            map.entry(dept.parent_id.clone()).or_default().push(dept);
        }

        if let Some(roots) = map.remove("0") {
            for mut root in roots {
                root.children = Self::build_children(root.id.clone(), &mut map);
                result.push(root);
            }
        }

        result
    }

    fn build_children(
        parent_id: String,
        map: &mut std::collections::HashMap<String, Vec<DeptVo>>,
    ) -> Option<Vec<DeptVo>> {
        if let Some(children) = map.remove(&parent_id) {
            let mut result = Vec::new();
            for mut child in children {
                child.children = Self::build_children(child.id.clone(), map);
                result.push(child);
            }
            Some(result)
        } else {
            None
        }
    }

    pub async fn get_by_id(&self, id: String) -> Result<DeptVo, AppError> {

        let model: dept::Model = dept::Entity::find()
            .filter(dept::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@dept_not_found".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateDeptDto) -> Result<DeptVo, AppError> {

        let active_model = dto.into_active_model();
        let model = active_model.insert(&self.db).await?;
        Ok(model.into())
    }

    pub async fn update(&self, dto: UpdateDeptDto) -> Result<DeptVo, AppError> {

        let active_model = dto.into_active_model();
        let model = active_model.update(&self.db).await?;
        Ok(model.into())
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {

        let model: dept::Model = dept::Entity::find()
            .filter(dept::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@dept_not_found".to_string()))?;

        let children: Vec<dept::Model> = dept::Entity::find()
            .filter(dept::Column::ParentId.eq(&id))
            .all(&self.db)
            .await?;

        // 批量软删除子部门（单条 SQL）
        if !children.is_empty() {
            let child_models: Vec<dept::ActiveModel> = children.into_iter().map(Into::into).collect();
            dept::Entity::delete_many_soft(child_models, &self.db).await?;
        }

        // 软删除父部门：保留 update 审计字段填充（who/when deleted）
        let mut am: dept::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}

// ===================== 列表导出（同步表头/行映射 + 异步导出注册） =====================

/// 部门导出表头
pub const DEPT_HEADERS: &[&str] = &[
    "部门名称", "上级部门", "排序", "负责人", "电话", "邮箱", "状态", "创建时间", "创建人",
    "创建人ID", "修改时间", "修改人", "修改人ID",
];

fn ts(t: chrono::DateTime<chrono::Utc>) -> String {
    t.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 部门列表 → 导出行
pub fn dept_rows(vos: &[DeptVo]) -> Vec<Vec<String>> {
    vos.iter()
        .map(|v| {
            vec![
                v.dept_name.clone(),
                v.parent_id.clone(),
                v.dept_sort.to_string(),
                v.leader.clone().unwrap_or_default(),
                v.phone.clone().unwrap_or_default(),
                v.email.clone().unwrap_or_default(),
                if v.status == 1 { "启用" } else { "禁用" }.to_string(),
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

/// 异步导出（> 10 万条）：导出中心按 `task_type = "dept"` 直接调用。
async fn dept_export_rows(ctx: ExportTaskContext) -> Result<Vec<Vec<String>>, AppError> {
    let service = App::global()
        .try_get_component::<DeptAppService>()
        .map_err(|e| AppError::Internal(format!("部门服务组件未就绪：{e}")))?;
    let query: DeptExportQuery = serde_json::from_value(ctx.query.clone())
        .map_err(|e| AppError::BadRequest(format!("导出参数不合法: {e}")))?;
    let vos = service.export(query).await?;
    Ok(dept_rows(&vos))
}

// 注册即完成：链接期自动登记，无执行器、无 install()、无需 main.rs 聚合。
fast_excel::export_task! {
    task_type = "dept",
    sheet_name = "部门数据",
    headers = DEPT_HEADERS,
    rows = dept_export_rows,
}
