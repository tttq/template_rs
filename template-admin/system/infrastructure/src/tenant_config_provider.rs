//! 动态租户配置 Provider 实现
//!
//! 从主库 `auth_sys_tenant` 表查询 `mode = 'database'` 的启用租户，
//! 转换为 [`TenantConnectionConfig`] 供 `DynamicTenantPlugin` 建立连接。

use async_trait::async_trait;
use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, QueryFilter};
use sea_orm_ext::{DynamicTenantConfigProvider, TenantConnectionConfig};
use system_entity::{tenant, tenant::Column};

/// 从 `auth_sys_tenant` 表加载租户数据库连接配置。
///
/// 仅查询 `mode = 'database'`、`status = 1` 且未软删除的租户。
pub struct TenantConfigProvider;

impl TenantConfigProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TenantConfigProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DynamicTenantConfigProvider for TenantConfigProvider {
    async fn load_all(
        &self,
        main_db: &DatabaseConnection,
    ) -> Result<Vec<TenantConnectionConfig>, DbErr> {
        let models = tenant::Entity::find()
            .filter(Column::Mode.eq("database"))
            .filter(Column::Status.eq(1))
            .filter(Column::DeleteFlag.eq(0))
            .all(main_db)
            .await?;

        Ok(models.into_iter().map(model_to_config).collect())
    }

    async fn load_one(
        &self,
        main_db: &DatabaseConnection,
        tenant_id: &str,
    ) -> Result<Option<TenantConnectionConfig>, DbErr> {
        let model = tenant::Entity::find()
            .filter(Column::Id.eq(tenant_id))
            .filter(Column::Mode.eq("database"))
            .filter(Column::Status.eq(1))
            .filter(Column::DeleteFlag.eq(0))
            .one(main_db)
            .await?;

        Ok(model.map(model_to_config))
    }
}

/// 将租户 Model 转换为 TenantConnectionConfig
fn model_to_config(m: tenant::Model) -> TenantConnectionConfig {
    let db_driver = m
        .database_type
        .as_deref()
        .unwrap_or("postgres")
        .to_lowercase();

    let db_url = build_database_url(
        &db_driver,
        m.database_url.as_deref().unwrap_or(""),
        m.database_name.as_deref(),
    );

    TenantConnectionConfig {
        tenant_id: m.id,
        db_url,
        db_driver,
        ..Default::default()
    }
}

/// 根据 database_type 拼接完整连接 URL
///
/// - PostgreSQL: `postgres://user:pass@host:port/dbname`
/// - MySQL: `mysql://user:pass@host:port/dbname`
/// - SQLite: 原样返回
fn build_database_url(database_type: &str, base_url: &str, database_name: Option<&str>) -> String {
    match database_type.to_lowercase().as_str() {
        "mysql" => {
            if let Some(name) = database_name {
                let query = base_url.find('?').map(|pos| &base_url[pos..]).unwrap_or("");
                let url_without_query = base_url.split('?').next().unwrap_or(base_url);
                format!(
                    "{}/{}{}",
                    url_without_query.trim_end_matches('/'),
                    name,
                    query
                )
            } else {
                base_url.to_string()
            }
        }
        "sqlite" => base_url.to_string(),
        _ => {
            // postgres
            if let Some(name) = database_name {
                let url_without_query = base_url.split('?').next().unwrap_or(base_url);
                let query = base_url.find('?').map(|pos| &base_url[pos..]).unwrap_or("");
                if let Some(pos) = url_without_query.rfind('/') {
                    format!("{}/{}{}", &url_without_query[..pos], name, query)
                } else {
                    base_url.to_string()
                }
            } else {
                base_url.to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_postgres_url_with_name() {
        let url = build_database_url(
            "postgres",
            "postgres://user:pass@localhost:5432/postgres",
            Some("tenant_001"),
        );
        assert_eq!(url, "postgres://user:pass@localhost:5432/tenant_001");
    }

    #[test]
    fn test_build_postgres_url_without_name() {
        let url = build_database_url(
            "postgres",
            "postgres://user:pass@localhost:5432/postgres",
            None,
        );
        assert_eq!(url, "postgres://user:pass@localhost:5432/postgres");
    }

    #[test]
    fn test_build_mysql_url_with_name() {
        let url = build_database_url(
            "mysql",
            "mysql://user:pass@localhost:3306",
            Some("tenant_001"),
        );
        assert_eq!(url, "mysql://user:pass@localhost:3306/tenant_001");
    }
}
