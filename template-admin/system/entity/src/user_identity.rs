use common::datetime_format;
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveAutoFillSoftDeleteTenant;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFillSoftDeleteTenant)]
#[sea_orm(table_name = "auth_sys_user_identity")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: String,
    pub user_id: String,
    pub provider: String,
    /// 第三方标识值：微信类存 unionid（拿不到时回退 openid），其他平台存原生 id。
    pub identity_value: String,
    pub nick_name: Option<String>,
    pub status: i32,
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
