use serde::{Deserialize, Serialize};
use system_entity::dict_type;
use sea_orm::ActiveValue::Set;
use chrono::{DateTime, Utc};
use common::datetime_format;
use common::pagination::PageQuery;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictTypeQuery {
    /// 分页参数（page / pageSize，含钳制）
    #[serde(flatten)]
    pub page_query: PageQuery,
    /// 查询条件 — 「字典名称」输入框；对应数据库列 auth_sys_dict_type.dict_name；模糊匹配（contains）
    pub dict_name: Option<String>,
    /// 查询条件 — 「字典编码」输入框；对应数据库列 auth_sys_dict_type.dict_type；模糊匹配（contains）
    pub dict_type: Option<String>,
    /// 查询条件 — 创建时间起始；>= 开始时间；对应数据库列 auth_sys_dict_type.create_time；区间（>=）
    pub create_time_start: Option<String>,
    /// 查询条件 — 创建时间结束；<= 结束时间；对应数据库列 auth_sys_dict_type.create_time；区间（<=）
    pub create_time_end: Option<String>,
    /// 查询条件 — 「创建人」输入框；对应数据库列 auth_sys_dict_type.create_by；模糊匹配（contains）
    pub create_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictTypeExportQuery {
    /// 查询条件 — 「字典名称」输入框；对应数据库列 auth_sys_dict_type.dict_name；模糊匹配（contains）
    pub dict_name: Option<String>,
    /// 查询条件 — 「字典编码」输入框；对应数据库列 auth_sys_dict_type.dict_type；模糊匹配（contains）
    pub dict_type: Option<String>,
    /// 查询条件 — 创建时间起始；>= 开始时间；对应数据库列 auth_sys_dict_type.create_time；区间（>=）
    pub create_time_start: Option<String>,
    /// 查询条件 — 创建时间结束；<= 结束时间；对应数据库列 auth_sys_dict_type.create_time；区间（<=）
    pub create_time_end: Option<String>,
    /// 查询条件 — 「创建人」输入框；对应数据库列 auth_sys_dict_type.create_by；模糊匹配（contains）
    pub create_by: Option<String>,
    /// 导出 ID 列表（逗号分隔，勾选导出）；对应数据库列 auth_sys_dict_type.id；列表（IN）
    pub ids: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDictTypeDto {
    pub dict_name: String,
    pub dict_type: String,
    pub status: Option<i32>,
    pub remark: Option<String>,
}

impl CreateDictTypeDto {
    pub fn into_active_model(self) -> dict_type::ActiveModel {
        dict_type::ActiveModel {
            dict_name: Set(self.dict_name),
            dict_type: Set(self.dict_type),
            status: Set(self.status.unwrap_or(1)),
            remark: Set(self.remark),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDictTypeDto {
    #[serde(default)]
    pub id: Option<String>,
    pub dict_name: Option<String>,
    pub dict_type: Option<String>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    #[serde(default)]
    pub version: Option<i32>,
}

impl UpdateDictTypeDto {
    pub fn into_active_model(self) -> dict_type::ActiveModel {
        let mut model = dict_type::ActiveModel {
            id: Set(self.id.unwrap_or_default()),
            version: Set(self.version.unwrap_or(0)),
            ..Default::default()
        };
        if let Some(v) = self.dict_name { model.dict_name = Set(v); }
        if let Some(v) = self.dict_type { model.dict_type = Set(v); }
        if let Some(v) = self.status { model.status = Set(v); }
        if let Some(v) = self.remark { model.remark = Set(Some(v)); }
        model
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictTypeVo {
    pub id: String,
    pub dict_name: String,
    pub dict_type: String,
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
}

impl From<dict_type::Model> for DictTypeVo {
    fn from(m: dict_type::Model) -> Self {
        Self {
            id: m.id,
            dict_name: m.dict_name,
            dict_type: m.dict_type,
            status: m.status,
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
