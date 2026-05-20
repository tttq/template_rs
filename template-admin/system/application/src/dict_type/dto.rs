use serde::{Deserialize, Serialize};
use system_entity::dict_type;
use sea_orm::ActiveValue::Set;
use chrono::{DateTime, Utc};
use common::datetime_format;

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
    pub id: Option<i64>,
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
            id: Set(self.id.unwrap_or(0)),
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
    pub id: i64,
    pub dict_name: String,
    pub dict_type: String,
    pub status: i32,
    pub remark: Option<String>,
    #[serde(with = "datetime_format")]
    pub create_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub update_time: DateTime<Utc>,
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
            update_time: m.update_time,
            tenant_id: m.tenant_id,
        }
    }
}
