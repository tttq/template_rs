use sea_orm_ext::{get_database_for_tenant, DbConn};
use sea_query::Value;
use crate::user::{get_current_tenant_id, get_current_tenant_mode};
use crate::error::AppError;

pub async fn get_effective_db(default_db: &DbConn) -> Result<DbConn, AppError> {
    let tenant_mode = get_current_tenant_mode();
    if tenant_mode.as_deref() == Some("database")
        && let Some(tenant_id) = get_current_tenant_id() {
            let tenant_id_value = Value::String(Some(tenant_id));
            if let Some(db) = get_database_for_tenant(&tenant_id_value)? {
                return Ok(DbConn::new(db));
            }
        }
    Ok(default_db.clone())
}


pub async fn get_effective_db_by_tenant_id(default_db: &DbConn, tenant_id: Option<String>) -> Result<DbConn, AppError> {
    let tenant_mode = get_current_tenant_mode();
    if tenant_mode.as_deref() == Some("database") {
        // 优先使用当前上下文中的租户ID，若不存在则使用入参提供的租户ID
        let effective_tenant_id = get_current_tenant_id().or(tenant_id);

        if let Some(id) = effective_tenant_id {
            let tenant_id_value = Value::String(Some(id));
            if let Some(db) = get_database_for_tenant(&tenant_id_value)? {
                return Ok(DbConn::new(db));
            }
        }
    }
    Ok(default_db.clone())
}
