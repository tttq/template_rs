use common::datetime_format;
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveAutoFillTenant;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFillTenant)]
#[sea_orm(table_name = "auth_sys_user_role")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: String,
    pub user_id: String,
    pub role_id: String,
    #[serde(with = "datetime_format")]
    #[sea_orm_ext(insert)]
    pub create_time: DateTimeUtc,
    #[sea_orm_ext(TENANT)]
    pub tenant_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::UserId",
        to = "super::user::Column::Id"
    )]
    User,
    #[sea_orm(
        belongs_to = "super::role::Entity",
        from = "Column::RoleId",
        to = "super::role::Column::Id"
    )]
    Role,
}
