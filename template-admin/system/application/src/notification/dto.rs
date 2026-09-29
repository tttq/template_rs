use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationVo {
    pub id: String,
    pub user_id: Option<String>,
    pub notify_type: String,
    pub channel: String,
    pub title: String,
    pub content: String,
    pub template_code: Option<String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
    pub read_flag: i32,
    pub read_at: Option<String>,
    pub send_status: String,
    pub create_time: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationQuery {
    /// 接收通知的用户 ID；服务端按当前登录用户过滤，对应数据库列 auth_sys_notification.user_id；精确匹配（eq）
    pub user_id: Option<String>,
    /// 查询条件 — 「通知类型」（如 system/order）；对应数据库列 auth_sys_notification.notify_type；精确匹配（eq）
    pub notify_type: Option<String>,
    /// 查询条件 — 「已读状态」（0 未读 / 1 已读）；对应数据库列 auth_sys_notification.read_flag；精确匹配（eq）
    pub read_flag: Option<i32>,
    /// 页号（>=1）
    pub page: Option<u64>,
    /// 每页条数（默认 20，1..=200）
    pub page_size: Option<u64>,
}

/// 发送通知请求：二选一
/// - 传 `templateCode`：按模板渲染后发送（`vars` 为变量表）
/// - 不传：直接使用 `title` / `content` 原文
///
/// 接收人支持多选：`userIds` 优先；仅传 `userId` 时按单个处理；
/// 两者都为空则不指定接收人（仅落库，行为与历史一致）。
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSendRequest {
    pub user_id: Option<String>,
    /// 接收用户 ID 列表（多选）
    #[serde(default)]
    pub user_ids: Vec<String>,
    pub notify_type: String,
    pub channel: String,
    pub title: Option<String>,
    pub content: Option<String>,
    /// 模板编码（与 vars 配合，按模板渲染后发送）
    pub template_code: Option<String>,
    /// 模板变量表
    #[serde(default)]
    pub vars: HashMap<String, String>,
    pub ref_type: Option<String>,
    pub ref_id: Option<String>,
}

impl NotificationSendRequest {
    /// 归一化接收人：`userIds` 优先，其次 `userId`；去空白、去重、剔除空串
    pub fn resolved_user_ids(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        let push = |out: &mut Vec<String>, raw: &str| {
            let v = raw.trim();
            if !v.is_empty() && !out.iter().any(|x| x == v) {
                out.push(v.to_string());
            }
        };

        for id in &self.user_ids {
            push(&mut out, id);
        }
        if out.is_empty() {
            if let Some(id) = self.user_id.as_deref() {
                push(&mut out, id);
            }
        }
        out
    }
}

/// 通知模板视图
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationTemplateVo {
    pub id: String,
    pub template_code: String,
    pub template_name: String,
    pub notify_type: String,
    pub channel: String,
    pub title_template: Option<String>,
    pub content_template: String,
    pub vars_hint: Option<String>,
    pub status: i32,
    pub remark: Option<String>,
    pub create_time: String,
    pub update_time: String,
}

/// 模板分页查询
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationTemplateQuery {
    /// 查询条件 — 「模板编码」输入框；对应数据库列 auth_sys_notification_template.template_code；模糊匹配（contains）
    pub template_code: Option<String>,
    /// 查询条件 — 「模板名称」输入框；对应数据库列 auth_sys_notification_template.template_name；模糊匹配（contains）
    pub template_name: Option<String>,
    /// 查询条件 — 「通知类型」（如 system/order）；对应数据库列 auth_sys_notification_template.notify_type；精确匹配（eq）
    pub notify_type: Option<String>,
    /// 查询条件 — 「投递渠道」（如 in_app/email）；对应数据库列 auth_sys_notification_template.channel；精确匹配（eq）
    pub channel: Option<String>,
    /// 查询条件 — 「启用状态」（1 启用 / 0 停用）；对应数据库列 auth_sys_notification_template.status；精确匹配（eq）
    pub status: Option<i32>,
    /// 页号（>=1）
    pub page: Option<u64>,
    /// 每页条数（默认 10，1..=200）
    pub page_size: Option<u64>,
}

/// 新增 / 编辑模板
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationTemplateSaveRequest {
    pub template_code: String,
    pub template_name: String,
    pub notify_type: String,
    pub channel: String,
    pub title_template: Option<String>,
    pub content_template: String,
    pub vars_hint: Option<String>,
    pub remark: Option<String>,
}

/// 模板试渲染请求（支持对尚未保存的草稿内容预览）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationTemplatePreviewRequest {
    pub title_template: Option<String>,
    pub content_template: String,
    #[serde(default)]
    pub vars: HashMap<String, String>,
}

/// 模板试渲染结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationTemplatePreviewVo {
    pub title: String,
    pub content: String,
    /// 模板中出现但变量表未提供的占位符（提示管理员补全）
    pub missing_vars: Vec<String>,
}

/// 模板启用 / 停用
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationTemplateStatusRequest {
    /// 1 启用 / 0 停用
    pub status: i32,
}

/// 接收人检索参数（通知中心用户选择框）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipientQuery {
    /// 关键字：匹配用户名 / 昵称
    pub keyword: Option<String>,
    /// 返回条数上限（默认 20，最大 50）
    pub limit: Option<u64>,
}

/// 接收人候选（仅返回选择框展示所需字段，不外泄手机号 / 邮箱等）
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecipientOptionVo {
    pub id: String,
    pub user_name: String,
    pub nick_name: Option<String>,
}