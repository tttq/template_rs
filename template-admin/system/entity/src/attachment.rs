//! auth_sys_attachment — 统一附件管理
//! category 取值：purchase（采购附件）/ ai（AI附件）/ system（系统附件）等

use common::datetime_format;
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveAutoFillSoftDeleteTenant;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFillSoftDeleteTenant)]
#[sea_orm(table_name = "auth_sys_attachment")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: String,
    pub category: String,
    pub biz_type: String,
    pub biz_id: Option<String>,
    pub image_type: Option<String>,
    pub sort: i32,
    pub caption: Option<String>,
    pub original_name: String,
    pub storage_path: String,
    pub mime_type: String,
    pub file_size: i64,
    pub uploader_id: Option<String>,
    pub status: String,
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