use serde::{Deserialize, Serialize};
use system_entity::tenant;
use sea_orm::ActiveValue::Set;
use chrono::{DateTime, Utc};
use common::datetime_format;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTenantDto {
    pub tenant_name: String,
    pub tenant_code: String,
    pub mode: String,
    pub database_type: Option<String>,
    pub database_url: Option<String>,
    pub database_name: Option<String>,
    pub status: Option<i32>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    #[serde(with = "datetime_format::option")]
    pub expire_time: Option<DateTime<Utc>>,
    pub remark: Option<String>,
}

impl CreateTenantDto {
    pub fn into_active_model(self) -> tenant::ActiveModel {
        tenant::ActiveModel {
            tenant_name: Set(self.tenant_name),
            tenant_code: Set(self.tenant_code),
            mode: Set(self.mode),
            database_type: Set(self.database_type),
            database_url: Set(self.database_url),
            database_name: Set(self.database_name),
            status: Set(self.status.unwrap_or(1)),
            contact_name: Set(self.contact_name),
            contact_phone: Set(self.contact_phone),
            contact_email: Set(self.contact_email),
            expire_time: Set(self.expire_time),
            remark: Set(self.remark),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTenantFullDto {
    pub tenant_name: String,
    pub tenant_code: String,
    pub database_type: Option<String>,
    pub database_url: Option<String>,
    pub database_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    #[serde(with = "datetime_format::option")]
    pub expire_time: Option<DateTime<Utc>>,
    pub remark: Option<String>,
    pub admin_user_name: String,
    pub admin_pass_word: String,
    pub admin_nick_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTenantDto {
    #[serde(default)]
    pub id: Option<String>,
    pub tenant_name: Option<String>,
    pub tenant_code: Option<String>,
    pub mode: Option<String>,
    pub database_type: Option<String>,
    pub database_url: Option<String>,
    pub database_name: Option<String>,
    pub status: Option<i32>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    #[serde(with = "datetime_format::option")]
    pub expire_time: Option<DateTime<Utc>>,
    pub remark: Option<String>,
    #[serde(default)]
    pub version: Option<i32>,
}

impl UpdateTenantDto {
    pub fn into_active_model(self) -> tenant::ActiveModel {
        let mut model = tenant::ActiveModel {
            id: Set(self.id.unwrap_or_default()),
            version: Set(self.version.unwrap_or(0)),
            ..Default::default()
        };
        if let Some(v) = self.tenant_name { model.tenant_name = Set(v); }
        if let Some(v) = self.tenant_code { model.tenant_code = Set(v); }
        if let Some(v) = self.mode { model.mode = Set(v); }
        if let Some(v) = self.database_type { model.database_type = Set(Some(v)); }
        if let Some(v) = self.database_url { model.database_url = Set(Some(v)); }
        if let Some(v) = self.database_name { model.database_name = Set(Some(v)); }
        if let Some(v) = self.status { model.status = Set(v); }
        if let Some(v) = self.contact_name { model.contact_name = Set(Some(v)); }
        if let Some(v) = self.contact_phone { model.contact_phone = Set(Some(v)); }
        if let Some(v) = self.contact_email { model.contact_email = Set(Some(v)); }
        if let Some(v) = self.expire_time { model.expire_time = Set(Some(v)); }
        if let Some(v) = self.remark { model.remark = Set(Some(v)); }
        model
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantVo {
    pub id: String,
    pub tenant_name: String,
    pub tenant_code: String,
    pub mode: String,
    pub database_type: Option<String>,
    pub database_url: Option<String>,
    pub database_name: Option<String>,
    pub status: i32,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    #[serde(with = "datetime_format::option")]
    pub expire_time: Option<DateTime<Utc>>,
    pub remark: Option<String>,
    #[serde(with = "datetime_format")]
    pub create_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub update_time: DateTime<Utc>,
}

impl From<tenant::Model> for TenantVo {
    fn from(m: tenant::Model) -> Self {
        Self {
            id: m.id,
            tenant_name: m.tenant_name,
            tenant_code: m.tenant_code,
            mode: m.mode,
            database_type: m.database_type,
            database_url: m.database_url,
            database_name: m.database_name,
            status: m.status,
            contact_name: m.contact_name,
            contact_phone: m.contact_phone,
            contact_email: m.contact_email,
            expire_time: m.expire_time,
            remark: m.remark,
            create_time: m.create_time,
            update_time: m.update_time,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TestConnectionDto {
    pub database_type: String,
    pub database_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDatabaseDto {
    pub database_type: String,
    pub database_url: String,
    pub database_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitDatabaseDto {
    pub database_type: String,
    pub database_url: String,
    pub database_name: String,
}
