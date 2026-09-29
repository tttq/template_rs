use serde::{Deserialize, Serialize};
use system_entity::config;
use sea_orm::ActiveValue::Set;
use chrono::{DateTime, Utc};
use common::datetime_format;
use common::pagination::PageQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigQuery {
    /// 分页参数（page / pageSize，含钳制）
    #[serde(flatten)]
    pub page_query: PageQuery,
    /// 查询条件 — 「参数名称」输入框；对应数据库列 auth_sys_config.config_name；模糊匹配（contains）
    pub config_name: Option<String>,
    /// 查询条件 — 「参数键名」输入框；对应数据库列 auth_sys_config.config_key；模糊匹配（contains）
    pub config_key: Option<String>,
    /// 查询条件 — 创建时间起始；>= 开始时间；对应数据库列 auth_sys_config.create_time；区间（>=）
    pub create_time_start: Option<String>,
    /// 查询条件 — 创建时间结束；<= 结束时间；对应数据库列 auth_sys_config.create_time；区间（<=）
    pub create_time_end: Option<String>,
    /// 查询条件 — 「创建人」输入框；对应数据库列 auth_sys_config.create_by；模糊匹配（contains）
    pub create_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigExportQuery {
    /// 查询条件 — 「参数名称」输入框；对应数据库列 auth_sys_config.config_name；模糊匹配（contains）
    pub config_name: Option<String>,
    /// 查询条件 — 「参数键名」输入框；对应数据库列 auth_sys_config.config_key；模糊匹配（contains）
    pub config_key: Option<String>,
    /// 查询条件 — 创建时间起始；>= 开始时间；对应数据库列 auth_sys_config.create_time；区间（>=）
    pub create_time_start: Option<String>,
    /// 查询条件 — 创建时间结束；<= 结束时间；对应数据库列 auth_sys_config.create_time；区间（<=）
    pub create_time_end: Option<String>,
    /// 查询条件 — 「创建人」输入框；对应数据库列 auth_sys_config.create_by；模糊匹配（contains）
    pub create_by: Option<String>,
    /// 导出 ID 列表（逗号分隔，勾选导出）；对应数据库列 auth_sys_config.id；列表（IN）
    pub ids: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateConfigDto {
    pub config_name: String,
    pub config_key: String,
    pub config_value: String,
    pub config_type: Option<String>,
    pub remark: Option<String>,
}

impl CreateConfigDto {
    pub fn into_active_model(self) -> config::ActiveModel {
        config::ActiveModel {
            config_name: Set(self.config_name),
            config_key: Set(self.config_key),
            config_value: Set(self.config_value),
            config_type: Set(self.config_type.unwrap_or_else(|| "string".to_string())),
            remark: Set(self.remark),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateConfigDto {
    #[serde(default)]
    pub id: Option<String>,
    pub config_name: Option<String>,
    pub config_key: Option<String>,
    pub config_value: Option<String>,
    pub config_type: Option<String>,
    pub remark: Option<String>,
    #[serde(default)]
    pub version: Option<i32>,
}

impl UpdateConfigDto {
    pub fn into_active_model(self) -> config::ActiveModel {
        let mut model = config::ActiveModel {
            id: Set(self.id.unwrap_or_default()),
            version: Set(self.version.unwrap_or(0)),
            ..Default::default()
        };
        if let Some(v) = self.config_name { model.config_name = Set(v); }
        if let Some(v) = self.config_key { model.config_key = Set(v); }
        if let Some(v) = self.config_value { model.config_value = Set(v); }
        if let Some(v) = self.config_type { model.config_type = Set(v); }
        if let Some(v) = self.remark { model.remark = Set(Some(v)); }
        model
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigVo {
    pub id: String,
    pub config_name: String,
    pub config_key: String,
    pub config_value: String,
    pub config_type: String,
    pub remark: Option<String>,
    #[serde(with = "datetime_format")]
    pub create_time: DateTime<Utc>,
    pub create_by: Option<String>,
    pub create_id: Option<String>,
    #[serde(with = "datetime_format")]
    pub update_time: DateTime<Utc>,
    pub update_by: Option<String>,
    pub update_id: Option<String>,
    pub tenant_id: Option<String>,
}

impl From<config::Model> for ConfigVo {
    fn from(m: config::Model) -> Self {
        Self {
            id: m.id,
            config_name: m.config_name,
            config_key: m.config_key,
            config_value: m.config_value,
            config_type: m.config_type,
            remark: m.remark,
            create_time: m.create_time,
            create_by: m.create_by,
            create_id: m.create_id,
            update_time: m.update_time,
            update_by: m.update_by,
            update_id: m.update_id,
            tenant_id: m.tenant_id,
        }
    }
}
