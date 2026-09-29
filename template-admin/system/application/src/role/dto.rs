use serde::{Deserialize, Serialize};
use system_entity::role;
use sea_orm::ActiveValue::Set;
use chrono::{DateTime, Utc};
use common::datetime_format;
use common::pagination::PageQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleQuery {
    /// 分页参数（page / pageSize，含钳制）
    #[serde(flatten)]
    pub page_query: PageQuery,
    /// 查询条件 — 所属客户端ID；对应数据库列 auth_sys_role.client_id
    pub client_id: Option<String>,
    /// 查询条件 — 「角色名称」输入框；对应数据库列 auth_sys_role.role_name；模糊匹配（contains）
    pub role_name: Option<String>,
    /// 查询条件 — 「角色编码」输入框；对应数据库列 auth_sys_role.role_code；模糊匹配（contains）
    pub role_code: Option<String>,
    /// 查询条件 — 创建时间起始；>= 开始时间；对应数据库列 auth_sys_role.create_time；区间（>=）
    pub create_time_start: Option<String>,
    /// 查询条件 — 创建时间结束；<= 结束时间；对应数据库列 auth_sys_role.create_time；区间（<=）
    pub create_time_end: Option<String>,
    /// 查询条件 — 「创建人」输入框；对应数据库列 auth_sys_role.create_by；模糊匹配（contains）
    pub create_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleExportQuery {
    /// 查询条件 — 所属客户端ID；对应数据库列 auth_sys_role.client_id
    pub client_id: Option<String>,
    /// 查询条件 — 「角色名称」输入框；对应数据库列 auth_sys_role.role_name；模糊匹配（contains）
    pub role_name: Option<String>,
    /// 查询条件 — 「角色编码」输入框；对应数据库列 auth_sys_role.role_code；模糊匹配（contains）
    pub role_code: Option<String>,
    /// 查询条件 — 创建时间起始；>= 开始时间；对应数据库列 auth_sys_role.create_time；区间（>=）
    pub create_time_start: Option<String>,
    /// 查询条件 — 创建时间结束；<= 结束时间；对应数据库列 auth_sys_role.create_time；区间（<=）
    pub create_time_end: Option<String>,
    /// 查询条件 — 「创建人」输入框；对应数据库列 auth_sys_role.create_by；模糊匹配（contains）
    pub create_by: Option<String>,
    /// 导出 ID 列表（逗号分隔，勾选导出）；对应数据库列 auth_sys_role.id；列表（IN）
    pub ids: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoleTreeQuery {
    /// 按客户端过滤角色（用户分配角色、角色管理页选择客户端后使用）
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRoleDto {
    /// 所属客户端ID（角色只在所属客户端内生效）
    pub client_id: Option<String>,
    pub parent_id: Option<String>,
    pub role_name: String,
    pub role_code: String,
    pub role_sort: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub menu_ids: Option<Vec<String>>,
}

impl CreateRoleDto {
    pub fn into_active_model(self) -> role::ActiveModel {
        role::ActiveModel {
            client_id: Set(self.client_id.unwrap_or_default().trim().to_string()),
            parent_id: Set(self.parent_id.unwrap_or_default()),
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
    pub id: Option<String>,
    #[serde(default)]
    pub client_id: Option<String>,
    pub parent_id: Option<String>,
    pub role_name: Option<String>,
    pub role_code: Option<String>,
    pub role_sort: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub menu_ids: Option<Vec<String>>,
    #[serde(default)]
    pub version: Option<i32>,
}

impl UpdateRoleDto {
    pub fn into_active_model(self) -> role::ActiveModel {
        let mut model = role::ActiveModel {
            id: Set(self.id.unwrap_or_default()),
            version: Set(self.version.unwrap_or(0)),
            ..Default::default()
        };
        if let Some(v) = self.parent_id { model.parent_id = Set(v); }
        if let Some(v) = self.client_id {
            let v = v.trim().to_string();
            if !v.is_empty() { model.client_id = Set(v); }
        }
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
    pub id: String,
    pub parent_id: String,
    pub client_id: String,
    pub role_name: String,
    pub role_code: String,
    pub role_sort: i32,
    pub status: i32,
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
    pub menu_ids: Option<Vec<String>>,
    pub children: Option<Vec<RoleVo>>,
}

impl From<role::Model> for RoleVo {
    fn from(m: role::Model) -> Self {
        Self {
            id: m.id,
            parent_id: m.parent_id,
            client_id: m.client_id,
            role_name: m.role_name,
            role_code: m.role_code,
            role_sort: m.role_sort,
            status: m.status,
            remark: m.remark,
            create_time: m.create_time,
            create_by: m.create_by,
            create_id: m.create_id,
            update_time: m.update_time,
            update_by: m.update_by,
            update_id: m.update_id,
            tenant_id: m.tenant_id,
            menu_ids: None,
            children: None,
        }
    }
}
