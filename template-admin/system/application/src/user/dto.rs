use serde::{Deserialize, Serialize};
use system_entity::user;
use sea_orm::ActiveValue::Set;
use chrono::{DateTime, Utc};
use common::datetime_format;
use common::hash_password;
use common::pagination::PageQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserQuery {
    /// 分页参数（page / pageSize，含钳制）
    #[serde(flatten)]
    pub page_query: PageQuery,
    /// 查询条件 — 「用户名」输入框；对应数据库列 auth_sys_user.user_name；模糊匹配（contains）
    pub user_name: Option<String>,
    /// 查询条件 — 「邮箱」输入框；对应数据库列 auth_sys_user.email；模糊匹配（contains）
    pub email: Option<String>,
    /// 查询条件 — 「手机号」输入框；对应数据库列 auth_sys_user.phone；模糊匹配（contains）
    pub phone: Option<String>,
    /// 查询条件 — 创建时间起始；>= 开始时间；对应数据库列 auth_sys_user.create_time；区间（>=）
    pub create_time_start: Option<String>,
    /// 查询条件 — 创建时间结束；<= 结束时间；对应数据库列 auth_sys_user.create_time；区间（<=）
    pub create_time_end: Option<String>,
    /// 查询条件 — 「创建人」输入框；对应数据库列 auth_sys_user.create_by；模糊匹配（contains）
    pub create_by: Option<String>,
}

/// 导出查询参数（支持按ID列表导出）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserExportQuery {
    /// 查询条件 — 「用户名」输入框；对应数据库列 auth_sys_user.user_name；模糊匹配（contains）
    pub user_name: Option<String>,
    /// 查询条件 — 「邮箱」输入框；对应数据库列 auth_sys_user.email；模糊匹配（contains）
    pub email: Option<String>,
    /// 查询条件 — 「手机号」输入框；对应数据库列 auth_sys_user.phone；模糊匹配（contains）
    pub phone: Option<String>,
    /// 查询条件 — 创建时间起始；>= 开始时间；对应数据库列 auth_sys_user.create_time；区间（>=）
    pub create_time_start: Option<String>,
    /// 查询条件 — 创建时间结束；<= 结束时间；对应数据库列 auth_sys_user.create_time；区间（<=）
    pub create_time_end: Option<String>,
    /// 查询条件 — 「创建人」输入框；对应数据库列 auth_sys_user.create_by；模糊匹配（contains）
    pub create_by: Option<String>,
    /// 导出 ID 列表（逗号分隔，勾选导出）；对应数据库列 auth_sys_user.id；列表（IN）
    pub ids: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserDto {
    pub user_name: String,
    pub pass_word: String,
    pub nick_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Option<i32>,
    pub dept_id: Option<String>,
    pub role_ids: Option<Vec<String>>,
}

impl CreateUserDto {
    pub fn into_active_model(self) -> Result<user::ActiveModel, common::error::AppError> {
        Ok(user::ActiveModel {
            user_name: Set(self.user_name),
            pass_word: Set(hash_password(&self.pass_word)?),
            nick_name: Set(self.nick_name),
            email: Set(self.email),
            phone: Set(self.phone),
            status: Set(self.status.unwrap_or(1)),
            admin_flag: Set(0),
            dept_id: Set(self.dept_id),
            ..Default::default()
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserDto {
    #[serde(default)]
    pub id: Option<String>,
    pub user_name: Option<String>,
    pub nick_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: Option<i32>,
    pub dept_id: Option<String>,
    pub role_ids: Option<Vec<String>>,
    #[serde(default)]
    pub version: Option<i32>,
}

impl UpdateUserDto {
    pub fn into_active_model(self) -> user::ActiveModel {
        let mut model = user::ActiveModel {
            id: Set(self.id.unwrap_or_default()),
            version: Set(self.version.unwrap_or(0)),
            ..Default::default()
        };
        if let Some(v) = self.user_name { model.user_name = Set(v); }
        if let Some(v) = self.nick_name { model.nick_name = Set(Some(v)); }
        if let Some(v) = self.email { model.email = Set(Some(v)); }
        if let Some(v) = self.phone { model.phone = Set(Some(v)); }
        if let Some(v) = self.status { model.status = Set(v); }
        if let Some(v) = self.dept_id { model.dept_id = Set(Some(v)); }
        model
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserVo {
    pub id: String,
    pub user_name: String,
    pub nick_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub status: i32,
    pub admin_flag: i32,
    pub dept_id: Option<String>,
    #[serde(with = "datetime_format")]
    pub create_time: DateTime<Utc>,
    pub create_by: Option<String>,
    pub create_id: Option<String>,
    #[serde(with = "datetime_format")]
    pub update_time: DateTime<Utc>,
    pub update_by: Option<String>,
    pub update_id: Option<String>,
    pub tenant_id: Option<String>,
    pub role_ids: Option<Vec<String>>,
}

impl From<user::Model> for UserVo {
    fn from(m: user::Model) -> Self {
        Self {
            id: m.id,
            user_name: m.user_name,
            nick_name: m.nick_name,
            email: m.email,
            phone: m.phone,
            avatar: m.avatar,
            status: m.status,
            admin_flag: m.admin_flag,
            dept_id: m.dept_id,
            create_time: m.create_time,
            create_by: m.create_by,
            create_id: m.create_id,
            update_time: m.update_time,
            update_by: m.update_by,
            update_id: m.update_id,
            tenant_id: m.tenant_id,
            role_ids: None,
        }
    }
}
