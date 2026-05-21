use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveTenant;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveTenant)]
#[sea_orm(table_name = "sea_orm_user_override")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: String,
    pub user_id: String,
    pub permission_id: String,
    pub resource_id: String,
    pub grant_or_deny: String,
    #[sea_orm_ext(TENANT)]
    pub tenant_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}