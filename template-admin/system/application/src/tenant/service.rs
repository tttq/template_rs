use common::error::AppError;
use common::pagination::{PageQuery, PageResult};
use summer::plugin::service::Service;
use summer_sea_orm::DbConn;
use system_entity::{tenant, user, role, role_menu, menu, tenant_user, user_role};
use sea_orm::{QueryFilter, ColumnTrait, PaginatorTrait, ActiveValue::Set, ConnectionTrait, DatabaseBackend, Database};
use sea_orm::prelude::*;
use sea_orm_ext::{get_tenant_store};
use sea_query::Value;

use super::dto::{CreateTenantDto, UpdateTenantDto, TenantVo, TestConnectionDto, CreateDatabaseDto, InitDatabaseDto, CreateTenantFullDto};

const INIT_SQL: &str = include_str!("init_schema.sql");

#[derive(Clone, Service)]
pub struct TenantAppService {
    #[inject(component)]
    db: DbConn,
}

impl TenantAppService {
    pub async fn list(&self, query: PageQuery) -> Result<PageResult<TenantVo>, AppError> {
        let paginator = tenant::Entity::find()
            .paginate(&self.db, query.page_size);

        let total = paginator.num_items().await?;
        let items: Vec<tenant::Model> = paginator.fetch_page(query.page - 1).await?;

        let vos: Vec<TenantVo> = items.into_iter().map(TenantVo::from).collect();
        Ok(PageResult::new(vos, total, query.page, query.page_size))
    }

    pub async fn get_by_id(&self, id: String) -> Result<TenantVo, AppError> {
        let model: tenant::Model = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(&id))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("租户不存在".to_string()))?;

        Ok(model.into())
    }

    pub async fn create(&self, dto: CreateTenantDto) -> Result<TenantVo, AppError> {
        let existing: Option<tenant::Model> = tenant::Entity::find()
            .filter(tenant::Column::TenantCode.eq(&dto.tenant_code))
            .one(&self.db)
            .await?;

        if existing.is_some() {
            return Err(AppError::BadRequest("租户编码已存在".to_string()));
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
            return Err(AppError::BadRequest("租户编码已存在".to_string()));
        }

        let db_type = dto.database_type.clone().unwrap_or_default();
        let db_url_base = dto.database_url.clone().unwrap_or_default();
        let db_name = dto.database_name.clone().unwrap_or(format!("tenant_{}", dto.tenant_code));

        let db_url = build_database_url(&db_type, &db_url_base, None);
        let conn = Database::connect(&db_url)
            .await
            .map_err(|e| AppError::BadRequest(format!("连接数据库服务器失败: {}", e)))?;

        conn.ping().await
            .map_err(|e| AppError::BadRequest(format!("连接测试失败: {}", e)))?;

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
            .map_err(|e| AppError::BadRequest(format!("创建数据库失败: {}", e)))?;

        let tenant_db_url = build_database_url(&db_type, &db_url_base, Some(&db_name));
        let tenant_db = Database::connect(&tenant_db_url)
            .await
            .map_err(|e| AppError::BadRequest(format!("连接租户数据库失败: {}", e)))?;

        let sql = adapt_init_sql(INIT_SQL, &backend);
        for statement in sql.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                tenant_db.execute_unprepared(trimmed)
                    .await
                    .map_err(|e| AppError::BadRequest(format!("初始化Schema失败: {}", e)))?;
            }
        }

        let admin_user = user::ActiveModel {
            user_name: Set(dto.admin_user_name.clone()),
            pass_word: Set(dto.admin_pass_word.clone()),
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
        let admin_result = admin_user.insert(&tenant_db).await?;

        let admin_role = role::ActiveModel {
            role_name: Set("管理员".to_string()),
            role_code: Set("admin".to_string()),
            role_sort: Set(1),
            status: Set(1),
            ..Default::default()
        };
        let role_result = admin_role.insert(&tenant_db).await?;

        let menus = menu::Entity::find()
            .filter(menu::Column::Status.eq(1))
            .all(&tenant_db)
            .await?;

        for m in menus {
            let rm = role_menu::ActiveModel {
                role_id: Set(role_result.id.clone()),
                menu_id: Set(m.id.clone()),
                ..Default::default()
            };
            rm.insert(&tenant_db).await?;
        }

        let user_role_model = user_role::ActiveModel {
            user_id: Set(admin_result.id.clone()),
            role_id: Set(role_result.id),
            ..Default::default()
        };
        user_role_model.insert(&tenant_db).await?;

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

        let tenant_id_value = Value::String(Some(tenant_result.id.clone()));
        if let Some(store) = get_tenant_store() {
            if let Err(e) = store.insert(tenant_id_value, tenant_db) {
                log::warn!("Failed to register tenant database connection: {}", e);
            }
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
            .ok_or_else(|| AppError::NotFound("租户不存在".to_string()))?;

        let mut am: tenant::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;

        if let Some(store) = get_tenant_store() {
            let _ = store.remove(&Value::String(Some(id)));
        }

        Ok(())
    }

    pub async fn test_connection(&self, dto: TestConnectionDto) -> Result<bool, AppError> {
        let db_url = build_database_url(&dto.database_type, &dto.database_url, None);
        let conn = Database::connect(&db_url)
            .await
            .map_err(|e| AppError::BadRequest(format!("连接失败: {}", e)))?;

        conn.ping().await
            .map_err(|e| AppError::BadRequest(format!("连接测试失败: {}", e)))?;

        Ok(true)
    }

    pub async fn create_database(&self, dto: CreateDatabaseDto) -> Result<String, AppError> {
        let db_url = build_database_url(&dto.database_type, &dto.database_url, None);
        let conn = Database::connect(&db_url)
            .await
            .map_err(|e| AppError::BadRequest(format!("连接数据库服务器失败: {}", e)))?;

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
            .map_err(|e| AppError::BadRequest(format!("创建数据库失败: {}", e)))?;

        Ok(format!("数据库 {} 创建成功", dto.database_name))
    }

    pub async fn init_database(&self, dto: InitDatabaseDto) -> Result<String, AppError> {
        let db_url = build_database_url(&dto.database_type, &dto.database_url, Some(&dto.database_name));
        let conn = Database::connect(&db_url)
            .await
            .map_err(|e| AppError::BadRequest(format!("连接数据库失败: {}", e)))?;

        let backend = get_database_backend(&dto.database_type);

        let sql = adapt_init_sql(INIT_SQL, &backend);
        for statement in sql.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                conn.execute_unprepared(trimmed)
                    .await
                    .map_err(|e| AppError::BadRequest(format!("执行初始化SQL失败: {}", e)))?;
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
                if base_url.contains('?') {
                    format!("{}/{}{}", base_url.trim_end_matches('/'), name, &base_url[base_url.find('?').unwrap()..])
                } else {
                    format!("{}/{}", base_url.trim_end_matches('/'), name)
                }
            } else {
                base_url.to_string()
            }
        }
        "sqlite" => base_url.to_string(),
        _ => {
            if let Some(name) = database_name {
                if let Some(pos) = base_url.rfind('/') {
                    format!("{}/{}", &base_url[..pos], name)
                } else {
                    base_url.to_string()
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