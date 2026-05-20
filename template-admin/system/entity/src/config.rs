use common::datetime_format;
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveAutoFillSoftDeleteTenant;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFillSoftDeleteTenant)]
#[sea_orm(table_name = "auth_sys_config")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: i64,
    pub config_name: String,
    pub config_key: String,
    pub config_value: String,
    pub config_type: String,
    pub remark: Option<String>,
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

