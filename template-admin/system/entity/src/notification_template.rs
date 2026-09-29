//! auth_sys_notification_template — 通知模板（邮件 / 站内消息）
//!
//! - `channel`: in_app（站内信）/ email（邮件）
//! - `title_template` / `content_template` 支持 `${varName}` 占位，
//!   发送时由调用方传入变量表渲染（见 common::template::render）
//! - `template_code` + `channel` 唯一，同一业务场景可分别配置站内信版与邮件版
//! - `vars_hint` 仅用于在管理界面提示该模板有哪些可用变量

use common::datetime_format;
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveAutoFillSoftDelete;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFillSoftDelete)]
#[sea_orm(table_name = "auth_sys_notification_template")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: String,
    /// 模板编码（业务侧引用，如 kyc_approved / email_code_register）
    pub template_code: String,
    /// 模板名称（管理界面展示）
    pub template_name: String,
    /// 通知类型（system / kyc / recharge / invoice / team ...）
    pub notify_type: String,
    /// 渠道：in_app（站内信）/ email（邮件）
    pub channel: String,
    /// 标题模板（邮件渠道时作为邮件主题）
    pub title_template: Option<String>,
    /// 正文模板，支持 ${varName}
    pub content_template: String,
    /// 可用变量提示（如 "code" / "applyNo,amount"），仅展示用
    pub vars_hint: Option<String>,
    /// 1 启用 / 0 停用（停用后按模板发送会报错，避免静默发出错误内容）
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
