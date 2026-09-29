use chrono::{DateTime, Utc};
use common::datetime_format;
use common::pagination::PageQuery;
use sea_orm::ActiveValue::Set;
use serde::{Deserialize, Serialize};
use system_entity::client;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientQuery {
    /// 分页参数（page / pageSize，含钳制）
    #[serde(flatten)]
    pub page_query: PageQuery,
    /// 客户端名称（模糊匹配）
    pub client_name: Option<String>,
    /// 客户端标识（模糊匹配）
    pub client_code: Option<String>,
    /// 客户端类型（web / miniapp / app / server）
    pub client_type: Option<String>,
    /// 状态（1=启用 0=禁用）
    pub status: Option<i32>,
    pub create_time_start: Option<String>,
    pub create_time_end: Option<String>,
    pub create_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientExportQuery {
    pub client_name: Option<String>,
    pub client_code: Option<String>,
    pub client_type: Option<String>,
    pub status: Option<i32>,
    pub create_time_start: Option<String>,
    pub create_time_end: Option<String>,
    pub create_by: Option<String>,
    /// 导出 ID 列表（逗号分隔，勾选导出）
    pub ids: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateClientDto {
    /// 客户端标识（登录时传递，创建后不可修改）
    pub client_code: String,
    /// 客户端密钥；留空自动生成 `cs-` 开头的随机串
    pub client_secret: Option<String>,
    pub client_name: String,
    pub client_type: Option<String>,
    pub logo: Option<String>,
    pub home_path: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
}

impl CreateClientDto {
    pub fn into_active_model(self) -> client::ActiveModel {
        let secret = self
            .client_secret
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .unwrap_or_else(common::generate_client_secret);
        client::ActiveModel {
            client_code: Set(self.client_code.trim().to_string()),
            client_secret: Set(secret),
            client_name: Set(self.client_name),
            client_type: Set(self.client_type.unwrap_or_else(|| "web".to_string())),
            logo: Set(self.logo),
            home_path: Set(self.home_path),
            sort_order: Set(self.sort_order.unwrap_or(0)),
            status: Set(self.status.unwrap_or(1)),
            remark: Set(self.remark),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateClientDto {
    #[serde(default)]
    pub id: Option<String>,
    /// client_code 创建后不可修改（只用于回显，后端忽略变更）
    pub client_code: Option<String>,
    /// 留空表示不修改密钥
    pub client_secret: Option<String>,
    pub client_name: Option<String>,
    pub client_type: Option<String>,
    pub logo: Option<String>,
    pub home_path: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    #[serde(default)]
    pub version: Option<i32>,
}

impl UpdateClientDto {
    pub fn into_active_model(self) -> client::ActiveModel {
        let mut model = client::ActiveModel {
            id: Set(self.id.unwrap_or_default()),
            version: Set(self.version.unwrap_or(0)),
            ..Default::default()
        };
        if let Some(v) = self.client_secret {
            let v = v.trim().to_string();
            if !v.is_empty() {
                model.client_secret = Set(v);
            }
        }
        if let Some(v) = self.client_name {
            model.client_name = Set(v);
        }
        if let Some(v) = self.client_type {
            model.client_type = Set(v);
        }
        if let Some(v) = self.logo {
            model.logo = Set(Some(v));
        }
        if let Some(v) = self.home_path {
            model.home_path = Set(Some(v));
        }
        if let Some(v) = self.sort_order {
            model.sort_order = Set(v);
        }
        if let Some(v) = self.status {
            model.status = Set(v);
        }
        if let Some(v) = self.remark {
            model.remark = Set(Some(v));
        }
        model
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientVo {
    pub id: String,
    pub client_code: String,
    pub client_secret: String,
    pub client_name: String,
    pub client_type: String,
    pub logo: Option<String>,
    pub home_path: Option<String>,
    pub sort_order: i32,
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
}

impl From<client::Model> for ClientVo {
    fn from(m: client::Model) -> Self {
        Self {
            id: m.id,
            client_code: m.client_code,
            client_secret: m.client_secret,
            client_name: m.client_name,
            client_type: m.client_type,
            logo: m.logo,
            home_path: m.home_path,
            sort_order: m.sort_order,
            status: m.status,
            remark: m.remark,
            create_time: m.create_time,
            create_by: m.create_by,
            create_id: m.create_id,
            update_time: m.update_time,
            update_by: m.update_by,
            update_id: m.update_id,
        }
    }
}

/// 下拉项（登录页/菜单页选择客户端用，不含密钥）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientOptionVo {
    pub id: String,
    pub client_code: String,
    pub client_name: String,
    pub client_type: String,
    pub logo: Option<String>,
    pub home_path: Option<String>,
}

impl From<client::Model> for ClientOptionVo {
    fn from(m: client::Model) -> Self {
        Self {
            id: m.id,
            client_code: m.client_code,
            client_name: m.client_name,
            client_type: m.client_type,
            logo: m.logo,
            home_path: m.home_path,
        }
    }
}
