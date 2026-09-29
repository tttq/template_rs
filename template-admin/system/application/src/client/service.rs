use common::error::AppError;
use common::pagination::PageResult;
use chrono::Utc;
use fast_excel::ExportTaskContext;
use sa_token_core::StpUtil;
use sea_orm::prelude::*;
use sea_orm::{
    ActiveValue::Set, ColumnTrait, PaginatorTrait, QueryFilter, QueryOrder, TransactionTrait,
};
use sea_orm_ext::{DbConn, TenantIgnoreGuard};
use summer::plugin::ComponentRegistry;
use summer::plugin::service::Service;
use summer::App;
use system_entity::{client, menu, role, role_menu, user, user_role};

use super::dto::{
    ClientExportQuery, ClientOptionVo, ClientQuery, ClientVo, CreateClientDto, UpdateClientDto,
};

/// 客户端标识校验：非空、≤64、仅字母数字与 `-` `_`
fn validate_client_code(code: &str) -> Result<(), AppError> {
    let code = code.trim();
    if code.is_empty() {
        return Err(AppError::BadRequest("@client_code_required".to_string()));
    }
    if code.len() > 64
        || !code
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::BadRequest("@client_code_invalid".to_string()));
    }
    Ok(())
}

/// 客户端管理：客户端是权限体系的顶级维度，其下挂菜单（含按钮权限）与角色。
#[derive(Clone, Service)]
pub struct ClientAppService {
    #[inject(component)]
    db: DbConn,
}

impl ClientAppService {
    pub async fn list(&self, query: ClientQuery) -> Result<PageResult<ClientVo>, AppError> {
        let _guard = TenantIgnoreGuard::new();
        let select = Self::filtered_select(
            query.client_name.as_deref(),
            query.client_code.as_deref(),
            query.client_type.as_deref(),
            query.status,
            query.create_time_start.as_deref(),
            query.create_time_end.as_deref(),
            query.create_by.as_deref(),
            None,
        );

        let paginator = select.paginate(&self.db, query.page_query.page_size);
        let total = paginator.num_items().await?;
        let items: Vec<client::Model> = paginator.fetch_page(query.page_query.page - 1).await?;
        let vos: Vec<ClientVo> = items.into_iter().map(ClientVo::from).collect();
        Ok(PageResult::new(
            vos,
            total,
            query.page_query.page,
            query.page_query.page_size,
        ))
    }

    pub async fn export(&self, query: ClientExportQuery) -> Result<Vec<ClientVo>, AppError> {
        let _guard = TenantIgnoreGuard::new();
        let items: Vec<client::Model> = Self::filtered_select(
            query.client_name.as_deref(),
            query.client_code.as_deref(),
            query.client_type.as_deref(),
            query.status,
            query.create_time_start.as_deref(),
            query.create_time_end.as_deref(),
            query.create_by.as_deref(),
            query.ids.as_deref(),
        )
        .all(&self.db)
        .await?;
        Ok(items.into_iter().map(ClientVo::from).collect())
    }

    pub async fn export_count(&self, query: &ClientExportQuery) -> Result<u64, AppError> {
        let _guard = TenantIgnoreGuard::new();
        Self::filtered_select(
            query.client_name.as_deref(),
            query.client_code.as_deref(),
            query.client_type.as_deref(),
            query.status,
            query.create_time_start.as_deref(),
            query.create_time_end.as_deref(),
            query.create_by.as_deref(),
            query.ids.as_deref(),
        )
        .count(&self.db)
        .await
        .map_err(AppError::from)
    }

    /// 列表/导出共用筛选口径（保证「可导出行数」与导出行一致）
    #[allow(clippy::too_many_arguments)]
    fn filtered_select(
        client_name: Option<&str>,
        client_code: Option<&str>,
        client_type: Option<&str>,
        status: Option<i32>,
        create_time_start: Option<&str>,
        create_time_end: Option<&str>,
        create_by: Option<&str>,
        ids: Option<&str>,
    ) -> sea_orm::Select<client::Entity> {
        let mut select = client::Entity::find().order_by_asc(client::Column::SortOrder);
        if let Some(v) = client_name.filter(|v| !v.is_empty()) {
            select = select.filter(client::Column::ClientName.contains(v));
        }
        if let Some(v) = client_code.filter(|v| !v.is_empty()) {
            select = select.filter(client::Column::ClientCode.contains(v));
        }
        if let Some(v) = client_type.filter(|v| !v.is_empty()) {
            select = select.filter(client::Column::ClientType.eq(v));
        }
        if let Some(v) = status {
            select = select.filter(client::Column::Status.eq(v));
        }
        if let Some(v) = create_time_start.filter(|v| !v.is_empty()) {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                select = select.filter(client::Column::CreateTime.gte(dt.with_timezone(&Utc)));
            }
        }
        if let Some(v) = create_time_end.filter(|v| !v.is_empty()) {
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                select = select.filter(client::Column::CreateTime.lte(dt.with_timezone(&Utc)));
            }
        }
        if let Some(v) = create_by.filter(|v| !v.is_empty()) {
            select = select.filter(client::Column::CreateBy.contains(v));
        }
        if let Some(v) = ids.filter(|v| !v.is_empty()) {
            let ids: Vec<String> = v.split(',').map(|s| s.trim().to_string()).collect();
            select = select.filter(client::Column::Id.is_in(ids));
        }
        select
    }

    /// 启用状态的客户端下拉项（菜单管理按客户端筛选、角色归属选择）
    pub async fn list_options(&self) -> Result<Vec<ClientOptionVo>, AppError> {
        let _guard = TenantIgnoreGuard::new();
        let items: Vec<client::Model> = client::Entity::find()
            .filter(client::Column::Status.eq(1))
            .order_by_asc(client::Column::SortOrder)
            .all(&self.db)
            .await?;
        Ok(items.into_iter().map(ClientOptionVo::from).collect())
    }

    pub async fn get_by_id(&self, id: String) -> Result<ClientVo, AppError> {
        let _guard = TenantIgnoreGuard::new();
        let model: client::Model = client::Entity::find()
            .filter(client::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@client_not_found".to_string()))?;
        Ok(model.into())
    }

    /// 按客户端标识查询（登录时识别客户端用，未登录场景需 `TenantIgnoreGuard`）
    pub async fn find_by_code(&self, client_code: &str) -> Result<Option<client::Model>, AppError> {
        let _guard = TenantIgnoreGuard::new();
        Ok(client::Entity::find()
            .filter(client::Column::ClientCode.eq(client_code.trim()))
            .one(&self.db)
            .await?)
    }

    pub async fn create(&self, dto: CreateClientDto) -> Result<ClientVo, AppError> {
        let _guard = TenantIgnoreGuard::new();
        validate_client_code(&dto.client_code)?;
        let exists = client::Entity::find()
            .filter(client::Column::ClientCode.eq(dto.client_code.trim()))
            .one(&self.db)
            .await?;
        if exists.is_some() {
            return Err(AppError::BadRequest("@client_code_exists".to_string()));
        }

        let model = dto.into_active_model().insert(&self.db).await?;
        Ok(model.into())
    }

    pub async fn update(&self, dto: UpdateClientDto) -> Result<ClientVo, AppError> {
        let _guard = TenantIgnoreGuard::new();
        let id = dto
            .id
            .clone()
            .filter(|v| !v.is_empty())
            .ok_or_else(|| AppError::BadRequest("@client_id_required".to_string()))?;

        let existing: client::Model = client::Entity::find()
            .filter(client::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@client_not_found".to_string()))?;

        // 客户端标识是登录入参，允许修改会让已发布的前端集体登录失败 —— 直接拒绝
        if let Some(code) = dto.client_code.as_deref().map(str::trim) {
            if !code.is_empty() && code != existing.client_code {
                return Err(AppError::BadRequest("@client_code_immutable".to_string()));
            }
        }

        let secret_changed = dto
            .client_secret
            .as_deref()
            .map(|s| !s.trim().is_empty() && s.trim() != existing.client_secret)
            .unwrap_or(false);
        let status_disabled = dto.status == Some(0) && existing.status != 0;
        let client_code = existing.client_code.clone();

        let model = dto.into_active_model().update(&self.db).await?;

        // 密钥变更/停用后回收该客户端下的在线会话，避免旧 token 继续访问
        if secret_changed || status_disabled {
            self.revoke_client_sessions(&client_code).await;
        }

        Ok(model.into())
    }

    /// 删除客户端：连同其菜单、角色以及角色的绑定关系一并回收（同一事务）
    pub async fn delete(&self, id: String) -> Result<(), AppError> {
        let _guard = TenantIgnoreGuard::new();
        let model: client::Model = client::Entity::find()
            .filter(client::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@client_not_found".to_string()))?;
        let client_code = model.client_code.clone();

        let tx = self.db.inner().begin().await?;

        let menus: Vec<menu::Model> = menu::Entity::find()
            .filter(menu::Column::ClientId.eq(&id))
            .all(&tx)
            .await?;
        if !menus.is_empty() {
            let models: Vec<menu::ActiveModel> = menus.into_iter().map(Into::into).collect();
            menu::Entity::delete_many_soft(models, &tx).await?;
        }

        let roles: Vec<role::Model> = role::Entity::find()
            .filter(role::Column::ClientId.eq(&id))
            .all(&tx)
            .await?;
        if !roles.is_empty() {
            let role_ids: Vec<String> = roles.iter().map(|r| r.id.clone()).collect();
            role_menu::Entity::delete_many()
                .filter(role_menu::Column::RoleId.is_in(role_ids.clone()))
                .exec(&tx)
                .await?;
            user_role::Entity::delete_many()
                .filter(user_role::Column::RoleId.is_in(role_ids))
                .exec(&tx)
                .await?;
            let models: Vec<role::ActiveModel> = roles.into_iter().map(Into::into).collect();
            role::Entity::delete_many_soft(models, &tx).await?;
        }

        let mut am: client::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&tx).await?;

        tx.commit().await?;

        // 事务外回收会话（Redis 副作用不能放进事务）
        self.revoke_client_sessions(&client_code).await;
        Ok(())
    }

    /// 回收某客户端下所有在线会话（login_id 形如 `用户ID#客户端编码`）
    async fn revoke_client_sessions(&self, client_code: &str) {
        let _guard = TenantIgnoreGuard::new();
        let users: Vec<String> = match user::Entity::find().all(&self.db).await {
            Ok(rows) => rows.into_iter().map(|u| u.id).collect(),
            Err(e) => {
                log::warn!("query users for client session revoke failed: {}", e);
                return;
            }
        };
        for user_id in users {
            let login_id = common::user::build_scoped_login_id(&user_id, Some(client_code));
            if let Err(e) = StpUtil::logout_by_login_id(login_id.as_str()).await {
                log::warn!("revoke client session failed: login_id={}, err={}", login_id, e);
            }
        }
    }
}

// ===================== 列表导出（同步表头/行映射 + 异步导出注册） =====================

/// 客户端导出表头
pub const CLIENT_HEADERS: &[&str] = &[
    "客户端名称",
    "客户端标识",
    "客户端类型",
    "首页路径",
    "排序",
    "状态",
    "备注",
    "创建时间",
    "创建人",
    "创建人ID",
    "修改时间",
    "修改人",
    "修改人ID",
];

fn ts(t: chrono::DateTime<chrono::Utc>) -> String {
    t.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 客户端列表 → 导出行
pub fn client_rows(vos: &[ClientVo]) -> Vec<Vec<String>> {
    vos.iter()
        .map(|v| {
            let type_str = match v.client_type.as_str() {
                "web" => "PC端",
                "miniapp" => "小程序",
                "app" => "移动端",
                "server" => "服务端",
                other => other,
            };
            vec![
                v.client_name.clone(),
                v.client_code.clone(),
                type_str.to_string(),
                v.home_path.clone().unwrap_or_default(),
                v.sort_order.to_string(),
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

/// 异步导出（> 10 万条）：导出中心按 `task_type = "client"` 调用
async fn client_export_rows(ctx: ExportTaskContext) -> Result<Vec<Vec<String>>, AppError> {
    let service = App::global()
        .try_get_component::<ClientAppService>()
        .map_err(|e| AppError::Internal(format!("客户端服务组件未就绪：{e}")))?;
    let query: ClientExportQuery = serde_json::from_value(ctx.query.clone())
        .map_err(|e| AppError::BadRequest(format!("导出参数不合法: {e}")))?;
    let vos = service.export(query).await?;
    Ok(client_rows(&vos))
}

// 注册即完成：链接期自动登记，无执行器、无 install()、无需 main.rs 聚合。
fast_excel::export_task! {
    task_type = "client",
    sheet_name = "客户端数据",
    headers = CLIENT_HEADERS,
    rows = client_export_rows,
}
