use common::error::AppError;
use common::pagination::PageResult;
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use sea_orm_ext::plugin::TenantManagerComponent;
use system_entity::{tenant, user, role, role_menu, menu, tenant_user, user_role};
use sea_orm::{QueryFilter, ColumnTrait, PaginatorTrait, ActiveValue::Set, ConnectionTrait, DatabaseBackend, Database, TransactionTrait};
use sea_orm::prelude::*;
use chrono::Utc;
use fast_excel::ExportTaskContext;
use summer::App;
use summer::plugin::ComponentRegistry;

use super::dto::{CreateTenantDto, UpdateTenantDto, TenantVo, TestConnectionDto, CreateDatabaseDto, InitDatabaseDto, CreateTenantFullDto, TenantQuery, TenantExportQuery};

const INIT_SQL: &str = include_str!("init_schema.sql");

#[derive(Clone, Service)]
pub struct TenantAppService {
    #[inject(component)]
    db: DbConn,
    // 仅在 database 模式 + DynamicTenantPlugin 启用时为 Some，
    // table 模式下为 None（此时 create_full/delete 中的缓存同步被跳过）。
    #[inject(component)]
    tenant_manager: Option<TenantManagerComponent>,
}

impl TenantAppService {
    pub async fn list(&self, query: TenantQuery) -> Result<PageResult<TenantVo>, AppError> {
        let mut select = tenant::Entity::find();

        if let Some(v) = &query.tenant_name {
            if !v.is_empty() {
                select = select.filter(tenant::Column::TenantName.contains(v));
            }
        }
        if let Some(v) = &query.tenant_code {
            if !v.is_empty() {
                select = select.filter(tenant::Column::TenantCode.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(tenant::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(tenant::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(tenant::Column::CreateBy.contains(v));
            }
        }

        let paginator = select.paginate(&self.db, query.page_query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<tenant::Model> = paginator.fetch_page(query.page_query.page - 1).await?;

        let vos: Vec<TenantVo> = items.into_iter().map(TenantVo::from).collect();
        Ok(PageResult::new(vos, total, query.page_query.page, query.page_query.page_size))
    }

    /// 导出租户列表
    pub async fn export(&self, query: TenantExportQuery) -> Result<Vec<TenantVo>, AppError> {
        let items: Vec<tenant::Model> = Self::export_select(&query).all(&self.db).await?;
        Ok(items.into_iter().map(TenantVo::from).collect())
    }

    /// 可导出行数（同步/异步分流判定；与 `export` 同口径，只 COUNT 不拉数据）
    pub async fn export_count(&self, query: &TenantExportQuery) -> Result<u64, AppError> {
        Self::export_select(query).count(&self.db).await.map_err(AppError::from)
    }

    /// 导出筛选条件（列表导出与行数统计共用同一口径）
    fn export_select(query: &TenantExportQuery) -> sea_orm::Select<tenant::Entity> {
        let mut select = tenant::Entity::find();

        if let Some(v) = &query.tenant_name {
            if !v.is_empty() {
                select = select.filter(tenant::Column::TenantName.contains(v));
            }
        }
        if let Some(v) = &query.tenant_code {
            if !v.is_empty() {
                select = select.filter(tenant::Column::TenantCode.contains(v));
            }
        }
        if let Some(v) = &query.create_time_start {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(tenant::Column::CreateTime.gte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_time_end {
            if !v.is_empty() {
                if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(v) {
                    select = select.filter(tenant::Column::CreateTime.lte(dt.with_timezone(&Utc)));
                }
            }
        }
        if let Some(v) = &query.create_by {
            if !v.is_empty() {
                select = select.filter(tenant::Column::CreateBy.contains(v));
            }
        }
        if let Some(ids_str) = &query.ids {
            if !ids_str.is_empty() {
                let ids: Vec<String> = ids_str.split(',').map(|s| s.trim().to_string()).collect();
                select = select.filter(tenant::Column::Id.is_in(ids));
            }
        }

        select
    }

    pub async fn get_by_id(&self, id: String) -> Result<TenantVo, AppError> {
        let model: tenant::Model = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@tenant_not_found".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateTenantDto) -> Result<TenantVo, AppError> {
        let existing: Option<tenant::Model> = tenant::Entity::find()
            .filter(tenant::Column::TenantCode.eq(&dto.tenant_code))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("@tenant_code_exists".to_string()));
        }

        let active_model = dto.into_active_model();
        let model = active_model.insert(&self.db).await?;
        Ok(model.into())
    }

    pub async fn create_full(&self, dto: CreateTenantFullDto) -> Result<TenantVo, AppError> {
        let existing = tenant::Entity::find()
            .filter(tenant::Column::TenantCode.eq(&dto.tenant_code))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("@tenant_code_exists".to_string()));
        }

        let db_type = dto.database_type.clone().unwrap_or_default();
        let db_url_base = dto.database_url.clone().unwrap_or_default();
        let db_name = dto.database_name.clone().unwrap_or(format!("tenant_{}", dto.tenant_code));

        let db_url = build_database_url(&db_type, &db_url_base, None);
        let conn = Database::connect(&db_url)
            .await
            .map_err(|e| AppError::BadRequest(format!("@db_connect_failed:{}", e)))?;

        conn.ping().await
            .map_err(|e| AppError::BadRequest(format!("@db_connect_test_failed:{}", e)))?;

        let backend = get_database_backend(&db_type);

        let sql = match backend {
            DatabaseBackend::Postgres => format!("CREATE DATABASE \"{}\"", db_name),
            DatabaseBackend::MySql => format!("CREATE DATABASE `{}`", db_name),
            _ => {
                return Err(AppError::BadRequest("SQLite 不支持创建数据库".to_string()));
            }
        };

        conn.execute_unprepared(&sql)
            .await
            .map_err(|e| AppError::BadRequest(format!("@db_create_failed:{}", e)))?;

        let tenant_db_url = build_database_url(&db_type, &db_url_base, Some(&db_name));
        let tenant_db = Database::connect(&tenant_db_url)
            .await
            .map_err(|e| AppError::BadRequest(format!("@tenant_db_connect_failed:{}", e)))?;

        let sql = adapt_init_sql(INIT_SQL, &backend);
        for statement in sql.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                tenant_db.execute_unprepared(trimmed)
                    .await
                    .map_err(|e| AppError::BadRequest(format!("@tenant_schema_init_failed:{}", e)))?;
            }
        }

        let admin_user = user::ActiveModel {
            user_name: Set(dto.admin_user_name.clone()),
            pass_word: Set(common::hash_password(&dto.admin_pass_word)?),
            nick_name: Set(dto.admin_nick_name.clone()),
            email: Set(None),
            phone: Set(None),
            identity_type: Set(Some("username".to_string())),
            identity_value: Set(None),
            status: Set(1),
            admin_flag: Set(1),
            dept_id: Set(None),
            ..Default::default()
        };
        // 租户库内的初始化（管理员用户 + 管理员角色 + 菜单授权 + 用户角色）放在同一事务：
        // 任一步失败都不会留下"有管理员但无角色/无菜单"的半初始化租户库
        // （租户库与主库是两个连接，无法合并为一个事务，主库写入在事务提交后执行）
        let tenant_tx = tenant_db.begin().await?;
        let admin_result = admin_user.insert(&tenant_tx).await?;

        // 客户端归属走列默认值（默认客户端「管理后台」，见 migration initial.sql / tenant init_schema.sql）：
        // 客户端注册表是全局表，新建租户库不复制客户端数据
        let admin_role = role::ActiveModel {
            role_name: Set("管理员".to_string()),
            role_code: Set("admin".to_string()),
            role_sort: Set(1),
            status: Set(1),
            ..Default::default()
        };
        let role_result = admin_role.insert(&tenant_tx).await?;

        let menus = menu::Entity::find()
            .filter(menu::Column::Status.eq(1))
            .all(&tenant_tx)
            .await?;

        for m in menus {
            let rm = role_menu::ActiveModel {
                role_id: Set(role_result.id.clone()),
                menu_id: Set(m.id.clone()),
                ..Default::default()
            };
            rm.insert(&tenant_tx).await?;
        }

        let user_role_model = user_role::ActiveModel {
            user_id: Set(admin_result.id.clone()),
            role_id: Set(role_result.id),
            ..Default::default()
        };
        user_role_model.insert(&tenant_tx).await?;
        tenant_tx.commit().await?;

        let tenant_model = tenant::ActiveModel {
            tenant_name: Set(dto.tenant_name),
            tenant_code: Set(dto.tenant_code),
            mode: Set("database".to_string()),
            database_type: Set(Some(db_type)),
            database_url: Set(Some(db_url_base)),
            database_name: Set(Some(db_name)),
            status: Set(1),
            contact_name: Set(dto.contact_name),
            contact_phone: Set(dto.contact_phone),
            contact_email: Set(dto.contact_email),
            expire_time: Set(dto.expire_time),
            remark: Set(dto.remark),
            ..Default::default()
        };
        let tenant_result = tenant_model.insert(&self.db).await?;

        tenant_user::ActiveModel {
            user_name: Set(admin_result.user_name),
            tenant_id: Set(tenant_result.id.clone()),
            tenant_code: Set(tenant_result.tenant_code.clone()),
            user_id: Set(admin_result.id),
            identity_type: Set("username".to_string()),
            identity_value: Set(None),
            status: Set(1),
            ..Default::default()
        }.insert(&self.db).await?;

        // 通过 TenantManager 注册租户连接：查询主库配置 → 建立连接池 → 缓存
        // 替代手动的 store.insert()，由 TenantManager 统一管理连接生命周期和健康检查。
        if let Some(ref tm) = self.tenant_manager
            && let Err(e) = tm.manager().add_tenant(&tenant_result.id).await {
                log::warn!("Failed to register tenant database connection via TenantManager: {}", e);
            }

        Ok(tenant_result.into())
    }

    pub async fn update(&self, dto: UpdateTenantDto) -> Result<TenantVo, AppError> {
        let active_model = dto.into_active_model();
        let model = active_model.update(&self.db).await?;
        Ok(model.into())
    }

    pub async fn delete(&self, id: String) -> Result<(), AppError> {
        let model: tenant::Model = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@tenant_not_found".to_string()))?;

        let mut am: tenant::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;

        // 通过 TenantManager 移除租户连接缓存（同时清理失败计数）
        if let Some(ref tm) = self.tenant_manager
            && let Err(e) = tm.manager().remove_tenant(&id).await {
                log::warn!("Failed to remove tenant database connection via TenantManager: {}", e);
            }

        Ok(())
    }

    pub async fn test_connection(&self, dto: TestConnectionDto) -> Result<bool, AppError> {
        let db_url = build_database_url(&dto.database_type, &dto.database_url, None);
        let conn = Database::connect(&db_url)
            .await
            .map_err(|e| AppError::BadRequest(format!("@connect_failed:{}", e)))?;

        conn.ping().await
            .map_err(|e| AppError::BadRequest(format!("@db_connect_test_failed:{}", e)))?;

        Ok(true)
    }

    pub async fn create_database(&self, dto: CreateDatabaseDto) -> Result<String, AppError> {
        let db_url = build_database_url(&dto.database_type, &dto.database_url, None);
        let conn = Database::connect(&db_url)
            .await
            .map_err(|e| AppError::BadRequest(format!("@db_connect_failed:{}", e)))?;

        let backend = get_database_backend(&dto.database_type);
        let sql = match backend {
            DatabaseBackend::Postgres => format!("CREATE DATABASE \"{}\"", dto.database_name),
            DatabaseBackend::MySql => format!("CREATE DATABASE `{}`", dto.database_name),
            _ => {
                return Err(AppError::BadRequest("SQLite 不支持创建数据库".to_string()));
            }
        };

        conn.execute_unprepared(&sql)
            .await
            .map_err(|e| AppError::BadRequest(format!("@db_create_failed:{}", e)))?;

        Ok(format!("数据库 {} 创建成功", dto.database_name))
    }

    pub async fn init_database(&self, dto: InitDatabaseDto) -> Result<String, AppError> {
        let db_url = build_database_url(&dto.database_type, &dto.database_url, Some(&dto.database_name));
        let conn = Database::connect(&db_url)
            .await
            .map_err(|e| AppError::BadRequest(format!("@tenant_db_connect_failed:{}", e)))?;

        let backend = get_database_backend(&dto.database_type);

        let sql = adapt_init_sql(INIT_SQL, &backend);
        for statement in sql.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                conn.execute_unprepared(trimmed)
                    .await
                    .map_err(|e| AppError::BadRequest(format!("@tenant_sql_init_failed:{}", e)))?;
            }
        }

        Ok(format!("数据库 {} 初始化成功", dto.database_name))
    }
}

fn get_database_backend(database_type: &str) -> DatabaseBackend {
    match database_type.to_lowercase().as_str() {
        "mysql" => DatabaseBackend::MySql,
        "sqlite" => DatabaseBackend::Sqlite,
        _ => DatabaseBackend::Postgres,
    }
}

fn build_database_url(database_type: &str, base_url: &str, database_name: Option<&str>) -> String {
    match database_type.to_lowercase().as_str() {
        "mysql" => {
            if let Some(name) = database_name {
                let query = base_url.find('?').map(|pos| &base_url[pos..]).unwrap_or("");
                let url_without_query = base_url.split('?').next().unwrap_or(base_url);
                format!("{}/{}{}", url_without_query.trim_end_matches('/'), name, query)
            } else {
                base_url.to_string()
            }
        }
        "sqlite" => base_url.to_string(),
        _ => {
            if let Some(name) = database_name {
                let url_without_query = base_url.split('?').next().unwrap_or(base_url);
                let query = base_url.find('?').map(|pos| &base_url[pos..]).unwrap_or("");
                if let Some(pos) = url_without_query.rfind('/') {
                    format!("{}/{}{}", &url_without_query[..pos], name, query)
                } else {
                    format!("{}{}", base_url, "")
                }
            } else {
                base_url.to_string()
            }
        }
    }
}

fn adapt_init_sql(sql: &str, _backend: &DatabaseBackend) -> String {
    sql.to_string()
}

// ===================== 列表导出（同步表头/行映射 + 异步导出注册） =====================

/// 租户导出表头
pub const TENANT_HEADERS: &[&str] = &[
    "租户名称", "租户编码", "隔离模式", "数据库类型", "数据库名称", "状态", "联系人", "联系电话",
    "联系邮箱", "创建时间", "创建人", "创建人ID", "修改时间", "修改人", "修改人ID",
];

fn ts(t: chrono::DateTime<chrono::Utc>) -> String {
    t.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// 租户列表 → 导出行
pub fn tenant_rows(vos: &[TenantVo]) -> Vec<Vec<String>> {
    vos.iter()
        .map(|v| {
            vec![
                v.tenant_name.clone(),
                v.tenant_code.clone(),
                if v.mode == "database" { "数据库隔离" } else { "表隔离" }.to_string(),
                v.database_type.clone().unwrap_or_default(),
                v.database_name.clone().unwrap_or_default(),
                if v.status == 1 { "启用" } else { "禁用" }.to_string(),
                v.contact_name.clone().unwrap_or_default(),
                v.contact_phone.clone().unwrap_or_default(),
                v.contact_email.clone().unwrap_or_default(),
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

/// 异步导出（> 10 万条）：导出中心按 `task_type = "tenant"` 直接调用。
async fn tenant_export_rows(ctx: ExportTaskContext) -> Result<Vec<Vec<String>>, AppError> {
    let service = App::global()
        .try_get_component::<TenantAppService>()
        .map_err(|e| AppError::Internal(format!("租户服务组件未就绪：{e}")))?;
    let query: TenantExportQuery = serde_json::from_value(ctx.query.clone())
        .map_err(|e| AppError::BadRequest(format!("导出参数不合法: {e}")))?;
    let vos = service.export(query).await?;
    Ok(tenant_rows(&vos))
}

// 注册即完成：链接期自动登记，无执行器、无 install()、无需 main.rs 聚合。
fast_excel::export_task! {
    task_type = "tenant",
    sheet_name = "租户数据",
    headers = TENANT_HEADERS,
    rows = tenant_export_rows,
}
