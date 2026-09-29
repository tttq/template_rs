use common::error::AppError;
use common::pagination::PageResult;
use summer::plugin::service::Service;
use summer_redis::Redis;
use sea_orm_ext::DbConn;
use system_entity::{client, menu, role, role_menu, user_role};
use sea_orm::{QueryFilter, ColumnTrait, PaginatorTrait, QueryOrder, TransactionTrait};
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::*;
use chrono::Utc;
use fast_excel::ExportTaskContext;
use summer::App;
use summer::plugin::ComponentRegistry;

use super::dto::{
    CreateRoleDto, RoleExportQuery, RoleQuery, RoleTreeQuery, RoleVo, UpdateRoleDto,
};
use crate::auth::permission_sync;

/// 去重 id 列表（保留首次出现顺序）。
///
/// 前端菜单树可能把同一 id 提交多次，重复绑定会触发唯一约束错误
/// （而角色菜单绑定是「先清空再插入」，一旦报错角色会丢光全部菜单）。
fn dedup_ids(ids: Vec<String>) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    ids.into_iter().filter(|id| seen.insert(id.clone())).collect()
}

#[derive(Clone, Service)]
pub struct RoleAppService {
    #[inject(component)]
    db: DbConn,
    #[inject(component)]
    redis: Redis,
}

impl RoleAppService {
    pub async fn list(&self, query: RoleQuery) -> Result<PageResult<RoleVo>, AppError> {
        let mut select = role::Entity::find();

        if let Some(v) = &query.client_id {
            if !v.is_empty() {
                select = select.filter(role::Column::ClientId.eq(v));
            }
        }
        if let Some(v) = &query.role_name {
            if !v.is_empty() {
                select = select.filter(role::Column::RoleName.contains(v));
            }
        }
        if let Some(v) = &query.role_code {
            if !v.is_empty() {
                select = select.filter(role::Column::RoleCode.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(role::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(role::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(role::Column::CreateBy.contains(v));
            }
        }

        let paginator = select.paginate(&self.db, query.page_query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<role::Model> = paginator.fetch_page(query.page_query.page - 1).await?;

        let vos: Vec<RoleVo> = items.into_iter().map(RoleVo::from).collect();
        Ok(PageResult::new(vos, total, query.page_query.page, query.page_query.page_size))
    }

    /// 导出角色列表
    pub async fn export(&self, query: RoleExportQuery) -> Result<Vec<RoleVo>, AppError> {
        let items: Vec<role::Model> = Self::export_select(&query).all(&self.db).await?;
        Ok(items.into_iter().map(RoleVo::from).collect())
    }

    /// 可导出行数（同步/异步分流判定；与 `export` 同口径，只 COUNT 不拉数据）
    pub async fn export_count(&self, query: &RoleExportQuery) -> Result<u64, AppError> {
        Self::export_select(query).count(&self.db).await.map_err(AppError::from)
    }

    /// 导出筛选条件（列表导出与行数统计共用同一口径）
    fn export_select(query: &RoleExportQuery) -> sea_orm::Select<role::Entity> {
        let mut select = role::Entity::find();

        if let Some(v) = &query.client_id {
            if !v.is_empty() {
                select = select.filter(role::Column::ClientId.eq(v));
            }
        }
        if let Some(v) = &query.role_name {
            if !v.is_empty() {
                select = select.filter(role::Column::RoleName.contains(v));
            }
        }
        if let Some(v) = &query.role_code {
            if !v.is_empty() {
                select = select.filter(role::Column::RoleCode.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(role::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(role::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(role::Column::CreateBy.contains(v));
            }
        }
        if let Some(ids_str) = &query.ids {
            if !ids_str.is_empty() {
                let ids: Vec<String> = ids_str.split(',').map(|s| s.trim().to_string()).collect();
                select = select.filter(role::Column::Id.is_in(ids));
            }
        }

        select
    }

    pub async fn list_tree(&self, query: RoleTreeQuery) -> Result<Vec<RoleVo>, AppError> {
        let mut select = role::Entity::find();
        if let Some(v) = query.client_id.filter(|v| !v.is_empty()) {
            select = select.filter(role::Column::ClientId.eq(v));
        }
        let items: Vec<role::Model> = select
            .order_by_asc(role::Column::RoleSort)
            .all(&self.db)
            .await?;

        let vos: Vec<RoleVo> = items.into_iter().map(RoleVo::from).collect();
        Ok(Self::build_tree(vos))
    }

    pub async fn list_all(&self, query: RoleTreeQuery) -> Result<Vec<RoleVo>, AppError> {
        let mut select = role::Entity::find();
        if let Some(v) = query.client_id.filter(|v| !v.is_empty()) {
            select = select.filter(role::Column::ClientId.eq(v));
        }
        let items: Vec<role::Model> = select
            .order_by_asc(role::Column::RoleSort)
            .all(&self.db)
            .await?;

        let vos: Vec<RoleVo> = items.into_iter().map(RoleVo::from).collect();
        Ok(vos)
    }

    fn build_tree(roles: Vec<RoleVo>) -> Vec<RoleVo> {
        let mut map: std::collections::HashMap<String, Vec<RoleVo>> = std::collections::HashMap::new();

        for role in &roles {
            map.entry(role.parent_id.clone()).or_default();
        }

        for role in roles {
            map.entry(role.parent_id.clone()).or_default().push(role);
        }

        let mut result = Vec::new();
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
        map: &mut std::collections::HashMap<String, Vec<RoleVo>>,
    ) -> Option<Vec<RoleVo>> {
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

    /// 校验菜单存在性：一个角色可跨客户端绑定任意客户端的菜单（单角色多端，
    /// 由运行期 `load_user_grant` 按登录客户端从这些菜单中收敛各自端的功能）。
    /// 此处仅防止绑定不存在的菜单 id，不再限制菜单必须属于角色所属客户端。
    async fn assert_menus_exist<C>(
        &self,
        db: &C,
        menu_ids: &[String],
    ) -> Result<(), AppError>
    where
        C: sea_orm::ConnectionTrait,
    {
        let ids = dedup_ids(menu_ids.to_vec());
        if ids.is_empty() {
            return Ok(());
        }
        let count = menu::Entity::find()
            .filter(menu::Column::Id.is_in(ids.clone()))
            .count(db)
            .await?;
        if count as usize != ids.len() {
            return Err(AppError::BadRequest("@menu_not_found".to_string()));
        }
        Ok(())
    }

    pub async fn get_by_id(&self, id: String) -> Result<RoleVo, AppError> {

        let model: role::Model = role::Entity::find()
            .filter(role::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@role_not_found".to_string()))?;

        let menus = role_menu::Entity::find()
            .filter(role_menu::Column::RoleId.eq(&id))
            .all(&self.db)
            .await?;

        let menu_ids: Vec<String> = menus.into_iter().map(|m| m.menu_id).collect();

        let mut vo: RoleVo = model.into();
        vo.menu_ids = Some(menu_ids);
        Ok(vo)
    }

    pub async fn create(&self, dto: CreateRoleDto) -> Result<RoleVo, AppError> {
        // 客户端是权限体系的顶级维度：角色必须挂在已存在的客户端下
        let client_id = dto.client_id.clone().unwrap_or_default();
        let client_id = client_id.trim().to_string();
        if client_id.is_empty() {
            return Err(AppError::BadRequest("@client_required".to_string()));
        }
        let client_exists = client::Entity::find()
            .filter(client::Column::Id.eq(&client_id))
            .one(&self.db)
            .await?;
        if client_exists.is_none() {
            return Err(AppError::BadRequest("@client_not_found".to_string()));
        }

        let existing: Option<role::Model> = role::Entity::find()
            .filter(role::Column::RoleCode.eq(&dto.role_code))
            .filter(role::Column::ClientId.eq(&client_id))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("@role_code_exists".to_string()));
        }

        let menu_ids = dto.menu_ids.clone();
        if let Some(ids) = &menu_ids {
            self.assert_menus_exist(&self.db, ids).await?;
        }
        let active_model = dto.into_active_model();
        // 角色与其菜单绑定必须同一事务：避免角色插入成功、绑定失败留下"无权限角色"
        let tx = self.db.inner().begin().await?;
        let model = active_model.insert(&tx).await?;

        if let Some(ids) = menu_ids {
            let models: Vec<role_menu::ActiveModel> = dedup_ids(ids).into_iter().map(|menu_id| role_menu::ActiveModel {
                role_id: Set(model.id.clone()),
                menu_id: Set(menu_id),
                ..Default::default()
            }).collect();
            role_menu::Entity::insert_many_with_fill(models, &tx).await?;
        }
        tx.commit().await?;

        let mut vo: RoleVo = model.into();
        vo.menu_ids = None;
        Ok(vo)
    }

    pub async fn update(&self, dto: UpdateRoleDto) -> Result<RoleVo, AppError> {
        let id = dto
            .id
            .clone()
            .filter(|v| !v.is_empty())
            .ok_or_else(|| AppError::BadRequest("@role_id_required".to_string()))?;
        let existing: role::Model = role::Entity::find()
            .filter(role::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@role_not_found".to_string()))?;

        // 角色不允许跨客户端迁移，否则已绑定菜单/用户会越界
        if let Some(target) = dto
            .client_id
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            if target != existing.client_id {
                return Err(AppError::BadRequest("@client_immutable".to_string()));
            }
        }

        if let Some(code) = dto.role_code.as_deref().map(str::trim).filter(|v| !v.is_empty()) {
            let dup: Option<role::Model> = role::Entity::find()
                .filter(role::Column::RoleCode.eq(code))
                .filter(role::Column::ClientId.eq(&existing.client_id))
                .filter(role::Column::Id.ne(&existing.id))
                .one(&self.db)
                .await?;
            if dup.is_some() {
                return Err(AppError::BadRequest("@role_code_exists".to_string()));
            }
        }

        let menu_ids = dto.menu_ids.clone();
        if let Some(ids) = &menu_ids {
            self.assert_menus_exist(&self.db, ids).await?;
        }
        let menus_changed = menu_ids.is_some();
        let active_model = dto.into_active_model();
        // 角色字段 + 菜单绑定同一事务：绑定是"先清空再插入"，
        // 若插入失败而清空已提交，该角色下的用户会瞬间失去全部权限
        let tx = self.db.inner().begin().await?;
        let model = active_model.update(&tx).await?;

        if let Some(ids) = menu_ids {
            role_menu::Entity::delete_many()
                .filter(role_menu::Column::RoleId.eq(&model.id))
                .exec(&tx)
                .await?;

            let models: Vec<role_menu::ActiveModel> = dedup_ids(ids).into_iter().map(|menu_id| role_menu::ActiveModel {
                role_id: Set(model.id.clone()),
                menu_id: Set(menu_id),
                ..Default::default()
            }).collect();
            role_menu::Entity::insert_many_with_fill(models, &tx).await?;
        }
        tx.commit().await?;

        if menus_changed {
            // 提交后刷新该角色下用户的权限快照（内部含 redis 通知，不能纳入事务）
            self.sync_role_users(&model.id).await;
        }

        let mut vo: RoleVo = model.into();
        vo.menu_ids = None;
        Ok(vo)
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {

        let model: role::Model = role::Entity::find()
            .filter(role::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@role_not_found".to_string()))?;

        let children: Vec<role::Model> = role::Entity::find()
            .filter(role::Column::ParentId.eq(&id))
            .all(&self.db)
            .await?;

        if !children.is_empty() {
            return Err(AppError::BadRequest("@role_has_children".to_string()));
        }

        // 角色软删与其菜单绑定清理同一事务，避免半删状态留下孤儿绑定
        let tx = self.db.inner().begin().await?;
        let mut am: role::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&tx).await?;

        role_menu::Entity::delete_many()
            .filter(role_menu::Column::RoleId.eq(&id))
            .exec(&tx)
            .await?;
        tx.commit().await?;

        self.sync_role_users(&id).await;

        Ok(())
    }

    pub async fn assign_menus(&self, role_id: String, menu_ids: Vec<String>) -> Result<(), AppError> {
        // 仅校验角色存在，后续绑定只用 role_id，不需要模型本身
        role::Entity::find()
            .filter(role::Column::Id.eq(&role_id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@role_not_found".to_string()))?;

        self.assert_menus_exist(&self.db, &menu_ids).await?;

        // 先清空再插入必须同一事务，否则插入失败会让角色丢光菜单
        let tx = self.db.inner().begin().await?;
        role_menu::Entity::delete_many()
            .filter(role_menu::Column::RoleId.eq(&role_id))
            .exec(&tx)
            .await?;

        let models: Vec<role_menu::ActiveModel> = dedup_ids(menu_ids).into_iter().map(|menu_id| role_menu::ActiveModel {
            role_id: Set(role_id.clone()),
            menu_id: Set(menu_id),
            ..Default::default()
        }).collect();
        role_menu::Entity::insert_many_with_fill(models, &tx).await?;
        tx.commit().await?;

        self.sync_role_users(&role_id).await;

        Ok(())
    }

    /// 角色菜单变更后：刷新该角色所有用户的接口权限快照，并推送前端刷新菜单
    async fn sync_role_users(&self, role_id: &str) {
        let user_ids: Vec<String> = match user_role::Entity::find()
            .filter(user_role::Column::RoleId.eq(role_id))
            .all(&self.db)
            .await
        {
            Ok(rows) => rows.into_iter().map(|r| r.user_id).collect(),
            Err(e) => {
                log::warn!("query users of role {} failed: {}", role_id, e);
                return;
            }
        };
        permission_sync::sync_and_notify(&self.db, &self.redis, &user_ids).await;
    }

    pub async fn get_role_menu_ids(&self, role_id: String) -> Result<Vec<String>, AppError> {

        let menus = role_menu::Entity::find()
            .filter(role_menu::Column::RoleId.eq(&role_id))
            .all(&self.db)
            .await?;

        Ok(menus.into_iter().map(|m| m.menu_id).collect())
    }
}

// ===================== 列表导出（同步表头/行映射 + 异步导出注册） =====================

/// 角色导出表头
pub const ROLE_HEADERS: &[&str] = &[
    "角色名称", "角色编码", "上级角色", "排序", "状态", "备注", "创建时间", "创建人", "创建人ID",
    "修改时间", "修改人", "修改人ID",
];

fn ts(t: chrono::DateTime<chrono::Utc>) -> String {
    t.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 角色列表 → 导出行
pub fn role_rows(vos: &[RoleVo]) -> Vec<Vec<String>> {
    vos.iter()
        .map(|v| {
            vec![
                v.role_name.clone(),
                v.role_code.clone(),
                v.parent_id.clone(),
                v.role_sort.to_string(),
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

/// 异步导出（> 10 万条）：导出中心按 `task_type = "role"` 直接调用。
async fn role_export_rows(ctx: ExportTaskContext) -> Result<Vec<Vec<String>>, AppError> {
    let service = App::global()
        .try_get_component::<RoleAppService>()
        .map_err(|e| AppError::Internal(format!("角色服务组件未就绪：{e}")))?;
    let query: RoleExportQuery = serde_json::from_value(ctx.query.clone())
        .map_err(|e| AppError::BadRequest(format!("导出参数不合法: {e}")))?;
    let vos = service.export(query).await?;
    Ok(role_rows(&vos))
}

// 注册即完成：链接期自动登记，无执行器、无 install()、无需 main.rs 聚合。
fast_excel::export_task! {
    task_type = "role",
    sheet_name = "角色数据",
    headers = ROLE_HEADERS,
    rows = role_export_rows,
}
