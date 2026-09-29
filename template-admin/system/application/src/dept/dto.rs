use serde::{Deserialize, Serialize};
use system_entity::dept;
use sea_orm::ActiveValue::Set;
use chrono::{DateTime, Utc};
use common::datetime_format;
use common::pagination::PageQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeptQuery {
    /// 分页参数（page / pageSize，含钳制）
    #[serde(flatten)]
    pub page_query: PageQuery,
    /// 查询条件 — 「部门名称」输入框；对应数据库列 auth_sys_dept.dept_name；模糊匹配（contains）
    pub dept_name: Option<String>,
    /// 查询条件 — 创建时间起始；>= 开始时间；对应数据库列 auth_sys_dept.create_time；区间（>=）
    pub create_time_start: Option<String>,
    /// 查询条件 — 创建时间结束；<= 结束时间；对应数据库列 auth_sys_dept.create_time；区间（<=）
    pub create_time_end: Option<String>,
    /// 查询条件 — 「创建人」输入框；对应数据库列 auth_sys_dept.create_by；模糊匹配（contains）
    pub create_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeptExportQuery {
    /// 查询条件 — 「部门名称」输入框；对应数据库列 auth_sys_dept.dept_name；模糊匹配（contains）
    pub dept_name: Option<String>,
    /// 查询条件 — 创建时间起始；>= 开始时间；对应数据库列 auth_sys_dept.create_time；区间（>=）
    pub create_time_start: Option<String>,
    /// 查询条件 — 创建时间结束；<= 结束时间；对应数据库列 auth_sys_dept.create_time；区间（<=）
    pub create_time_end: Option<String>,
    /// 查询条件 — 「创建人」输入框；对应数据库列 auth_sys_dept.create_by；模糊匹配（contains）
    pub create_by: Option<String>,
    /// 导出 ID 列表（逗号分隔，勾选导出）；对应数据库列 auth_sys_dept.id；列表（IN）
    pub ids: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDeptDto {
    pub parent_id: String,
    pub dept_name: String,
    pub dept_sort: Option<i32>,
    pub status: Option<i32>,
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

impl CreateDeptDto {
    pub fn into_active_model(self) -> dept::ActiveModel {
        dept::ActiveModel {
            parent_id: Set(self.parent_id),
            dept_name: Set(self.dept_name),
            dept_sort: Set(self.dept_sort.unwrap_or(0)),
            status: Set(self.status.unwrap_or(1)),
            leader: Set(self.leader),
            phone: Set(self.phone),
            email: Set(self.email),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDeptDto {
    #[serde(default)]
    pub id: Option<String>,
    pub parent_id: Option<String>,
    pub dept_name: Option<String>,
    pub dept_sort: Option<i32>,
    pub status: Option<i32>,
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub version: Option<i32>,
}

impl UpdateDeptDto {
    pub fn into_active_model(self) -> dept::ActiveModel {
        let mut model = dept::ActiveModel {
            id: Set(self.id.unwrap_or_default()),
            version: Set(self.version.unwrap_or(0)),
            ..Default::default()
        };
        if let Some(v) = self.parent_id { model.parent_id = Set(v); }
        if let Some(v) = self.dept_name { model.dept_name = Set(v); }
        if let Some(v) = self.dept_sort { model.dept_sort = Set(v); }
        if let Some(v) = self.status { model.status = Set(v); }
        if let Some(v) = self.leader { model.leader = Set(Some(v)); }
        if let Some(v) = self.phone { model.phone = Set(Some(v)); }
        if let Some(v) = self.email { model.email = Set(Some(v)); }
        model
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeptVo {
    pub id: String,
    pub parent_id: String,
    pub dept_name: String,
    pub dept_sort: i32,
    pub status: i32,
    pub leader: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    #[serde(with = "datetime_format")]
    pub create_time: DateTime<Utc>,
    pub create_by: Option<String>,
    pub create_id: Option<String>,
    #[serde(with = "datetime_format")]
    pub update_time: DateTime<Utc>,
    pub update_by: Option<String>,
    pub update_id: Option<String>,
    pub tenant_id: Option<String>,
    pub children: Option<Vec<DeptVo>>,
}

impl From<dept::Model> for DeptVo {
    fn from(m: dept::Model) -> Self {
        Self {
            id: m.id,
            parent_id: m.parent_id,
            dept_name: m.dept_name,
            dept_sort: m.dept_sort,
            status: m.status,
            leader: m.leader,
            phone: m.phone,
            email: m.email,
            create_time: m.create_time,
            create_by: m.create_by,
            create_id: m.create_id,
            update_time: m.update_time,
            update_by: m.update_by,
            update_id: m.update_id,
            tenant_id: m.tenant_id,
            children: None,
        }
    }
}
