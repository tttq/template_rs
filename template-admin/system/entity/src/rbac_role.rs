use common::datetime_format;
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveTenant;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveTenant)]
#[sea_orm(table_name = "sea_orm_role")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: i64,
    pub role_name: String,
    #[serde(with = "datetime_format")]
    pub create_time: DateTimeUtc,
    #[sea_orm_ext(TENANT)]
    pub tenant_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}