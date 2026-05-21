use serde::{Deserialize, Serialize};
use system_entity::menu;
use sea_orm::ActiveValue::Set;
use chrono::{DateTime, Utc};
use common::datetime_format;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMenuDto {
    pub parent_id: String,
    pub menu_name: String,
    pub menu_type: String,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub permission: Option<String>,
    pub status: Option<i32>,
    pub visible: Option<i32>,
}

impl CreateMenuDto {
    pub fn into_active_model(self) -> menu::ActiveModel {
        menu::ActiveModel {
            parent_id: Set(self.parent_id),
            menu_name: Set(self.menu_name),
            menu_type: Set(self.menu_type),
            path: Set(self.path),
            component: Set(self.component),
            icon: Set(self.icon),
            sort_order: Set(self.sort_order.unwrap_or(0)),
            permission: Set(self.permission),
            status: Set(self.status.unwrap_or(1)),
            visible: Set(self.visible.unwrap_or(1)),
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMenuDto {
    #[serde(default)]
    pub id: Option<String>,
    pub parent_id: Option<String>,
    pub menu_name: Option<String>,
    pub menu_type: Option<String>,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i32>,
    pub permission: Option<String>,
    pub status: Option<i32>,
    pub visible: Option<i32>,
    #[serde(default)]
    pub version: Option<i32>,
}

impl UpdateMenuDto {
    pub fn into_active_model(self) -> menu::ActiveModel {
        let mut model = menu::ActiveModel {
            id: Set(self.id.unwrap_or_default()),
            version: Set(self.version.unwrap_or(0)),
            ..Default::default()
        };
        if let Some(v) = self.parent_id { model.parent_id = Set(v); }
        if let Some(v) = self.menu_name { model.menu_name = Set(v); }
        if let Some(v) = self.menu_type { model.menu_type = Set(v); }
        if let Some(v) = self.path { model.path = Set(Some(v)); }
        if let Some(v) = self.component { model.component = Set(Some(v)); }
        if let Some(v) = self.icon { model.icon = Set(Some(v)); }
        if let Some(v) = self.sort_order { model.sort_order = Set(v); }
        if let Some(v) = self.permission { model.permission = Set(Some(v)); }
        if let Some(v) = self.status { model.status = Set(v); }
        if let Some(v) = self.visible { model.visible = Set(v); }
        model
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MenuVo {
    pub id: String,
    pub parent_id: String,
    pub menu_name: String,
    pub menu_type: String,
    pub path: Option<String>,
    pub component: Option<String>,
    pub icon: Option<String>,
    pub sort_order: i32,
    pub permission: Option<String>,
    pub status: i32,
    pub visible: i32,
    #[serde(with = "datetime_format")]
    pub create_time: DateTime<Utc>,
    #[serde(with = "datetime_format")]
    pub update_time: DateTime<Utc>,
    pub tenant_id: Option<String>,
    pub children: Option<Vec<MenuVo>>,
}

impl From<menu::Model> for MenuVo {
    fn from(m: menu::Model) -> Self {
        Self {
            id: m.id,
            parent_id: m.parent_id,
            menu_name: m.menu_name,
            menu_type: m.menu_type,
            path: m.path,
            component: m.component,
            icon: m.icon,
            sort_order: m.sort_order,
            permission: m.permission,
            status: m.status,
            visible: m.visible,
            create_time: m.create_time,
            update_time: m.update_time,
            tenant_id: m.tenant_id,
            children: None,
        }
    }
}
