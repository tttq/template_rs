//! auth_sys_notification — 统一通知中心
//!
//! notify_type: system（系统消息）/ user（用户消息）/ kyc（KYC通知）/ recharge（充值）/ invoice（发票）/ rebate（返利）
//! channel: in_app（站内信）/ email（邮件）/ sms（短信）

use common::datetime_format;
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveAutoFill;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFill)]
#[sea_orm(table_name = "auth_sys_notification")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: String,
    pub user_id: Option<String>,
    pub notify_type: String,
    pub channel: String,
    pub title: String,
    pub content: String,
    /// 来源模板编码（按模板发送时写入，便于追溯；手工发送为 NULL）
    pub template_code: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub read_flag: i32,
    pub read_at: Option<DateTimeUtc>,
    pub send_status: String,
    pub send_error: Option<String>,
    #[serde(with = "datetime_format")]
    #[sea_orm_ext(insert)]
    pub create_time: DateTimeUtc,
    pub create_by: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}