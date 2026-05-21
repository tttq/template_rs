use serde::{Deserialize, Serialize};
use system_entity::dict_item;
use sea_orm::ActiveValue::Set;
use chrono::{DateTime, Utc};
use common::datetime_format;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateDictItemDto {
    pub dict_type_id: String,
    pub dict_label: String,
    pub dict_value: String,
    pub sort_order: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub css_class: Option<String>,
    pub list_class: Option<String>,
}

impl CreateDictItemDto {
    pub fn into_active_model(self) -> dict_item::ActiveModel {
        dict_item::ActiveModel {
            dict_type_id: Set(self.dict_type_id),
            dict_label: Set(self.dict_label),
            dict_value: Set(self.dict_value),
            sort_order: Set(self.sort_order.unwrap_or(0)),
            status: Set(self.status.unwrap_or(1)),
            remark: Set(self.remark),
            css_class: Set(self.css_class),
            list_class: Set(self.list_class),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateDictItemDto {
    #[serde(default)]
    pub id: Option<String>,
    pub dict_type_id: Option<String>,
    pub dict_label: Option<String>,
    pub dict_value: Option<String>,
    pub sort_order: Option<i32>,
    pub status: Option<i32>,
    pub remark: Option<String>,
    pub css_class: Option<String>,
    pub list_class: Option<String>,
    #[serde(default)]
    pub version: Option<i32>,
}

impl UpdateDictItemDto {
    pub fn into_active_model(self) -> dict_item::ActiveModel {
        let mut model = dict_item::ActiveModel {
            id: Set(self.id.unwrap_or_default()),
            version: Set(self.version.unwrap_or(0)),
            ..Default::default()
        };
        if let Some(v) = self.dict_type_id { model.dict_type_id = Set(v); }
        if let Some(v) = self.dict_label { model.dict_label = Set(v); }
        if let Some(v) = self.dict_value { model.dict_value = Set(v); }
        if let Some(v) = self.sort_order { model.sort_order = Set(v); }
        if let Some(v) = self.status { model.status = Set(v); }
        if let Some(v) = self.remark { model.remark = Set(Some(v)); }
        if let Some(v) = self.css_class { model.css_class = Set(Some(v)); }
        if let Some(v) = self.list_class { model.list_class = Set(Some(v)); }
        model
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DictItemVo {
    pub id: String,
    pub dict_type_id: String,
    pub dict_label: String,
    pub dict_value: String,
    pub sort_order: i32,
    pub status: i32,
    pub remark: Option<String>,
    pub css_class: Option<String>,
    pub list_class: Option<String>,
    #[serde(with = "datetime_format")]
    pub create_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub update_time: DateTime<Utc>,
    pub tenant_id: Option<String>,
}

impl From<dict_item::Model> for DictItemVo {
    fn from(m: dict_item::Model) -> Self {
        Self {
            id: m.id,
            dict_type_id: m.dict_type_id,
            dict_label: m.dict_label,
            dict_value: m.dict_value,
            sort_order: m.sort_order,
            status: m.status,
            remark: m.remark,
            css_class: m.css_class,
            list_class: m.list_class,
            create_time: m.create_time,
            update_time: m.update_time,
            tenant_id: m.tenant_id,
        }
    }
}
