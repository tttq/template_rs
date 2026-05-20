use serde::{Deserialize, Serialize};
use system_entity::role;
use sea_orm::ActiveValue::Set;
use chrono::{DateTime, Utc};
use common::datetime_format;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleDto {
    pub parent_id: Option<i64>,
    pub role_name: String,
    pub role_code: String,
    pub role_sort: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub menu_ids: Option<Vec<i64>>,
}

impl CreateRoleDto {
    pub fn into_active_model(self) -> role::ActiveModel {
        role::ActiveModel {
            parent_id: Set(self.parent_id.unwrap_or(0)),
            role_name: Set(self.role_name),
            role_code: Set(self.role_code),
            role_sort: Set(self.role_sort.unwrap_or(0)),
            status: Set(self.status.unwrap_or(1)),
            remark: Set(self.remark),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleDto {
    #[serde(default)]
    pub id: Option<i64>,
    pub parent_id: Option<i64>,
    pub role_name: Option<String>,
    pub role_code: Option<String>,
    pub role_sort: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub menu_ids: Option<Vec<i64>>,
    #[serde(default)]
    pub version: Option<i32>,
}

impl UpdateRoleDto {
    pub fn into_active_model(self) -> role::ActiveModel {
        let mut model = role::ActiveModel {
            id: Set(self.id.unwrap_or(0)),
            version: Set(self.version.unwrap_or(0)),
            ..Default::default()
        };
        if let Some(v) = self.parent_id { model.parent_id = Set(v); }
        if let Some(v) = self.role_name { model.role_name = Set(v); }
        if let Some(v) = self.role_code { model.role_code = Set(v); }
        if let Some(v) = self.role_sort { model.role_sort = Set(v); }
        if let Some(v) = self.status { model.status = Set(v); }
        if let Some(v) = self.remark { model.remark = Set(Some(v)); }
        model
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleVo {
    pub id: i64,
    pub parent_id: i64,
    pub role_name: String,
    pub role_code: String,
    pub role_sort: i32,
    pub status: i32,
    pub remark: Option<String>,
    #[serde(with = "datetime_format")]
    pub create_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub update_time: DateTime<Utc>,
    pub tenant_id: Option<String>,
    pub menu_ids: Option<Vec<i64>>,
    pub children: Option<Vec<RoleVo>>,
}

impl From<role::Model> for RoleVo {
    fn from(m: role::Model) -> Self {
        Self {
            id: m.id,
            parent_id: m.parent_id,
            role_name: m.role_name,
            role_code: m.role_code,
            role_sort: m.role_sort,
            status: m.status,
            remark: m.remark,
            create_time: m.create_time,
            update_time: m.update_time,
            tenant_id: m.tenant_id,
            menu_ids: None,
            children: None,
        }
    }
}
