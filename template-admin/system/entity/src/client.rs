use common::datetime_format;
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveAutoFillSoftDelete;
use serde::{Deserialize, Serialize};

/// 客户端（端/应用）：权限体系的顶级维度。
///
/// 菜单（含按钮权限）与角色都归属于某个客户端；登录时携带 client_code +
/// client_secret 识别客户端（OAuth2 的 client_id/client_secret 模式），
/// 只加载该客户端下的菜单与功能权限。
///
/// 客户端注册表是**全局表**（与 `auth_sys_tenant` 同类）：登录发生在确定租户之前，
/// 客户端标识必须全局唯一，因此不参与租户隔离（无 tenant_id 列）。
#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFillSoftDelete)]
#[sea_orm(table_name = "auth_sys_client")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: String,
    /// 客户端标识（登录时传递，创建后不可修改）
    pub client_code: String,
    /// 客户端密钥（登录时校验）
    pub client_secret: String,
    pub client_name: String,
    /// web / miniapp / app / server
    pub client_type: String,
    pub logo: Option<String>,
    /// 登录后默认首页路径
    pub home_path: Option<String>,
    pub sort_order: i32,
    pub status: i32,
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
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
