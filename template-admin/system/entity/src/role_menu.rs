use common::datetime_format;
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveAutoFillSoftDeleteTenant;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFillSoftDeleteTenant)]
#[sea_orm(table_name = "auth_sys_role_menu")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: i64,
    pub role_id: i64,
    pub menu_id: i64,
    #[serde(with = "datetime_format")]
    #[sea_orm_ext(insert)]
    pub create_time: DateTimeUtc,
    #[sea_orm(version)]
    #[sea_orm_ext(insert_update)]
    pub version: i32,
    #[soft_delete(default = 0, del = 1)]
    pub delete_flag: i32,
    #[sea_orm_ext(TENANT)]
    pub tenant_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::role::Entity",
        from = "Column::RoleId",
        to = "super::role::Column::Id"
    )]
    Role,
    #[sea_orm(
        belongs_to = "super::menu::Entity",
        from = "Column::MenuId",
        to = "super::menu::Column::Id"
    )]
    Menu,
}
