use common::error::AppError;
use common::pagination::PageResult;
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use system_entity::{client, menu};
use sea_orm::{QueryFilter, ColumnTrait, QueryOrder, PaginatorTrait, ActiveValue::Set};
use sea_orm::prelude::*;
use chrono::Utc;
use fast_excel::ExportTaskContext;
use summer::App;
use summer::plugin::ComponentRegistry;

use super::dto::{
    CreateMenuDto, MenuExportQuery, MenuQuery, MenuTreeQuery, MenuVo, UpdateMenuDto,
};

#[derive(Clone, Service)]
pub struct MenuAppService {
    #[inject(component)]
    db: DbConn,
}

impl MenuAppService {
    pub async fn list(&self, query: MenuQuery) -> Result<PageResult<MenuVo>, AppError> {
        let mut select = menu::Entity::find();

        if let Some(v) = &query.client_id {
            if !v.is_empty() {
                select = select.filter(menu::Column::ClientId.eq(v));
            }
        }
        if let Some(v) = &query.menu_name {
            if !v.is_empty() {
                select = select.filter(menu::Column::MenuName.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(menu::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(menu::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(menu::Column::CreateBy.contains(v));
            }
        }

        select = select.order_by_asc(menu::Column::SortOrder);

        let paginator = select.paginate(&self.db, query.page_query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<menu::Model> = paginator.fetch_page(query.page_query.page - 1).await?;

        let vos: Vec<MenuVo> = items.into_iter().map(MenuVo::from).collect();
        Ok(PageResult::new(vos, total, query.page_query.page, query.page_query.page_size))
    }

    pub async fn list_tree(&self, query: MenuTreeQuery) -> Result<Vec<MenuVo>, AppError> {
        let mut select = menu::Entity::find();
        if let Some(v) = query.client_id.filter(|v| !v.is_empty()) {
            select = select.filter(menu::Column::ClientId.eq(v));
        }
        let items: Vec<menu::Model> = select
            .order_by_asc(menu::Column::SortOrder)
            .all(&self.db)
            .await?;

        let vos: Vec<MenuVo> = items.into_iter().map(MenuVo::from).collect();
        Ok(Self::build_tree(vos))
    }

    /// 导出菜单列表
    pub async fn export(&self, query: MenuExportQuery) -> Result<Vec<MenuVo>, AppError> {
        let items: Vec<menu::Model> = Self::export_select(&query).all(&self.db).await?;
        Ok(items.into_iter().map(MenuVo::from).collect())
    }

    /// 可导出行数（同步/异步分流判定；与 `export` 同口径，只 COUNT 不拉数据）
    pub async fn export_count(&self, query: &MenuExportQuery) -> Result<u64, AppError> {
        Self::export_select(query).count(&self.db).await.map_err(AppError::from)
    }

    /// 导出筛选条件（列表导出与行数统计共用同一口径）
    fn export_select(query: &MenuExportQuery) -> sea_orm::Select<menu::Entity> {
        let mut select = menu::Entity::find();

        if let Some(v) = &query.client_id {
            if !v.is_empty() {
                select = select.filter(menu::Column::ClientId.eq(v));
            }
        }
        if let Some(v) = &query.menu_name {
            if !v.is_empty() {
                select = select.filter(menu::Column::MenuName.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(menu::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(menu::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(menu::Column::CreateBy.contains(v));
            }
        }
        if let Some(ids_str) = &query.ids {
            if !ids_str.is_empty() {
                let ids: Vec<String> = ids_str.split(',').map(|s| s.trim().to_string()).collect();
                select = select.filter(menu::Column::Id.is_in(ids));
            }
        }

        select = select.order_by_asc(menu::Column::SortOrder);

        select
    }

    fn build_tree(menus: Vec<MenuVo>) -> Vec<MenuVo> {
        let mut result = Vec::new();
        let mut map: std::collections::HashMap<String, Vec<MenuVo>> = std::collections::HashMap::new();

        for menu in menus {
            map.entry(menu.parent_id.clone()).or_default().push(menu);
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
        map: &mut std::collections::HashMap<String, Vec<MenuVo>>,
    ) -> Option<Vec<MenuVo>> {
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

    pub async fn get_by_id(&self, id: String) -> Result<MenuVo, AppError> {

        let model: menu::Model = menu::Entity::find()
            .filter(menu::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@menu_not_found".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateMenuDto) -> Result<MenuVo, AppError> {
        // 客户端是权限体系的顶级维度：菜单必须挂在已存在的客户端下
        let client_id = dto.client_id.clone().unwrap_or_default();
        let client_id = client_id.trim();
        if client_id.is_empty() {
            return Err(AppError::BadRequest("@client_required".to_string()));
        }
        let exists = client::Entity::find()
            .filter(client::Column::Id.eq(client_id))
            .one(&self.db)
            .await?;
        if exists.is_none() {
            return Err(AppError::BadRequest("@client_not_found".to_string()));
        }
        let active_model = dto.into_active_model();
        let model = active_model.insert(&self.db).await?;
        Ok(model.into())
    }

    pub async fn update(&self, dto: UpdateMenuDto) -> Result<MenuVo, AppError> {
        // 菜单不允许跨客户端迁移（角色/权限按客户端隔离，迁移会让已授权角色越界）
        if let Some(id) = dto.id.as_deref().filter(|v| !v.is_empty()) {
            if let Some(target) = dto.client_id.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
                let model: menu::Model = menu::Entity::find()
                    .filter(menu::Column::Id.eq(id))
                    .one(&self.db)
                    .await?
                    .ok_or_else(|| AppError::NotFound("@menu_not_found".to_string()))?;
                if model.client_id != target {
                    return Err(AppError::BadRequest("@client_immutable".to_string()));
                }
            }
        }
        let active_model = dto.into_active_model();
        let model = active_model.update(&self.db).await?;
        Ok(model.into())
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {

        let model: menu::Model = menu::Entity::find()
            .filter(menu::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@menu_not_found".to_string()))?;

        let children: Vec<menu::Model> = menu::Entity::find()
            .filter(menu::Column::ParentId.eq(&id))
            .all(&self.db)
            .await?;

        // 批量软删除子菜单（单条 SQL）
        if !children.is_empty() {
            let child_models: Vec<menu::ActiveModel> = children.into_iter().map(Into::into).collect();
            menu::Entity::delete_many_soft(child_models, &self.db).await?;
        }

        // 软删除父菜单：保留 update 审计字段填充（who/when deleted）
        let mut am: menu::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }
}

// ===================== 列表导出（同步表头/行映射 + 异步导出注册） =====================

/// 菜单导出表头
pub const MENU_HEADERS: &[&str] = &[
    "菜单名称", "菜单类型", "路由路径", "组件路径", "图标", "排序", "权限标识", "状态", "可见性",
    "创建时间", "创建人", "创建人ID", "修改时间", "修改人", "修改人ID",
];

fn ts(t: chrono::DateTime<chrono::Utc>) -> String {
    t.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 菜单列表 → 导出行
pub fn menu_rows(vos: &[MenuVo]) -> Vec<Vec<String>> {
    vos.iter()
        .map(|v| {
            let type_str = match v.menu_type.as_str() {
                "dir" => "目录",
                "menu" => "菜单",
                "button" => "按钮",
                other => other,
            };
            vec![
                v.menu_name.clone(),
                type_str.to_string(),
                v.path.clone().unwrap_or_default(),
                v.component.clone().unwrap_or_default(),
                v.icon.clone().unwrap_or_default(),
                v.sort_order.to_string(),
                v.permission.clone().unwrap_or_default(),
                if v.status == 1 { "启用" } else { "禁用" }.to_string(),
                if v.visible == 1 { "显示" } else { "隐藏" }.to_string(),
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

/// 异步导出（> 10 万条）：导出中心按 `task_type = "menu"` 直接调用。
async fn menu_export_rows(ctx: ExportTaskContext) -> Result<Vec<Vec<String>>, AppError> {
    let service = App::global()
        .try_get_component::<MenuAppService>()
        .map_err(|e| AppError::Internal(format!("菜单服务组件未就绪：{e}")))?;
    let query: MenuExportQuery = serde_json::from_value(ctx.query.clone())
        .map_err(|e| AppError::BadRequest(format!("导出参数不合法: {e}")))?;
    let vos = service.export(query).await?;
    Ok(menu_rows(&vos))
}

// 注册即完成：链接期自动登记，无执行器、无 install()、无需 main.rs 聚合。
fast_excel::export_task! {
    task_type = "menu",
    sheet_name = "菜单数据",
    headers = MENU_HEADERS,
    rows = menu_export_rows,
}
