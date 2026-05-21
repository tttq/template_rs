use serde::{Deserialize, Serialize};
use system_entity::config;
use sea_orm::ActiveValue::Set;
use chrono::{DateTime, Utc};
use common::datetime_format;

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
    #[serde(with = "datetime_format")]
    pub update_time: DateTime<Utc>,
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
            update_time: m.update_time,
            tenant_id: m.tenant_id,
        }
    }
}
