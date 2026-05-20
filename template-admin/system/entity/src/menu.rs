use common::datetime_format;
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveAutoFillSoftDeleteTenant;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFillSoftDeleteTenant)]
#[sea_orm(table_name = "auth_sys_menu")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: i64,
    pub parent_id: i64,
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
    #[sea_orm_ext(insert)]
    pub create_time: DateTimeUtc,
    #[sea_orm_ext(insert)]
    pub create_by: Option<String>,
    #[sea_orm_ext(insert)]
    pub create_id: Option<String>,
    #[serde(with = "datetime_format")]
    #[sea_orm_ext(update)]
    pub update_time: DateTimeUtc,
    #[sea_orm_ext(update)]
    pub update_by: Option<String>,
    #[sea_orm_ext(update)]
    pub update_id: Option<String>,
    #[sea_orm(version)]
    #[sea_orm_ext(insert_update)]
    pub version: i32,
    #[soft_delete(default = 0, del = 1)]
    pub delete_flag: i32,
    #[sea_orm_ext(TENANT)]
    pub tenant_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

