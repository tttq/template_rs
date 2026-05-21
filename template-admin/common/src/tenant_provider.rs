use sea_orm_ext::TenantIdProvider;
use sea_query::Value;

pub struct SaTokenTenantIdProvider;

impl TenantIdProvider for SaTokenTenantIdProvider {
    fn get_tenant_id(&self) -> Option<Value> {
        crate::user::get_current_tenant_id()
            .map(|id| Value::String(Some(id)))
    }
}
