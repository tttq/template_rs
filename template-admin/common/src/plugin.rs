use sea_orm_ext::{FieldFillHandler, FieldFillOperation, set_field_fill_handler, set_tenant_config, TenantConfig, TenantMode};
use sea_query::Value;
use summer::plugin::Plugin;
use summer::{app::AppBuilder, async_trait};
use summer::config::{Configurable, ConfigRegistry};
use serde::Deserialize;

#[derive(Debug, Clone, Configurable, Deserialize)]
#[config_prefix = "common"]
pub struct CommonConfig {
    pub default_user: String,
    pub default_tenant_id: Option<String>,
}

impl Default for CommonConfig {
    fn default() -> Self {
        Self {
            default_user: "system".to_string(),
            default_tenant_id: None,
        }
    }
}

pub struct CommonPlugin;

impl CommonPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for CommonPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for CommonPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app.get_config::<CommonConfig>().unwrap_or_default();
        let handler = AuditFieldFillHandler::new(config.default_user);
        set_field_fill_handler(Box::new(handler));
    }

    fn name(&self) -> &'static str {
        "common"
    }
}

struct AuditFieldFillHandler {
    default_user: String,
}

impl AuditFieldFillHandler {
    fn new(default_user: String) -> Self {
        Self { default_user }
    }
}

impl FieldFillHandler for AuditFieldFillHandler {
    fn fill(
        &self,
        _entity_name: &str,
        field_name: &str,
        operation: FieldFillOperation,
    ) -> Option<Value> {
        let user_id = crate::user::get_current_user_id()
            .unwrap_or_else(|| self.default_user.clone());
        let user_name = crate::user::get_current_user_name()
            .unwrap_or_else(|| self.default_user.clone());

        match (field_name, operation) {
            ("create_time", FieldFillOperation::Insert)
            | ("update_time", FieldFillOperation::Update)
            | ("update_time", FieldFillOperation::Insert) => {
                Some(chrono::Utc::now().into())
            }
            ("create_by", FieldFillOperation::Insert)
            | ("update_by", FieldFillOperation::Update)
            | ("update_by", FieldFillOperation::Insert) => {
                Some(user_name.into())
            }
            ("create_id", FieldFillOperation::Insert)
            | ("update_id", FieldFillOperation::Update)
            | ("update_id", FieldFillOperation::Insert) => {
                Some(user_id.into())
            }
            ("version", FieldFillOperation::Insert) => Some(1i32.into()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Configurable, Deserialize)]
#[config_prefix = "sea-orm-ext.tenant"]
pub struct TenantPluginConfig {
    pub enabled: bool,
    pub mode: String,
    pub default_tenant_id: Option<i64>,
    pub header_name: Option<String>,
    pub ignore_if_missing: Option<bool>,
}

impl Default for TenantPluginConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: "table".to_string(),
            default_tenant_id: None,
            header_name: Some("x-tenant-id".to_string()),
            ignore_if_missing: Some(true),
        }
    }
}

pub struct TenantPlugin;

impl TenantPlugin {
    pub fn new() -> Self {
        Self
    }
}

impl Default for TenantPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for TenantPlugin {
    async fn build(&self, app: &mut AppBuilder) {
        let config = app.get_config::<TenantPluginConfig>().unwrap_or_default();

        if !config.enabled {
            tracing::info!("Tenant plugin is disabled, skipping initialization");
            return;
        }

        let mode = match config.mode.to_lowercase().as_str() {
            "database" => TenantMode::Database,
            _ => TenantMode::Table,
        };

        let default_tenant_id: Option<Value> =
            config.default_tenant_id.map(|id| Value::BigInt(Some(id)));

        set_tenant_config(TenantConfig {
            enabled: true,
            mode,
            default_tenant_id,
            ignored_tables: Default::default(),
        });

        tracing::info!("Tenant plugin initialized in {} mode", config.mode);
    }

    fn name(&self) -> &'static str {
        "tenant"
    }
}