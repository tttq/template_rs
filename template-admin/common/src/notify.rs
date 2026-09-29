//! 通知发送器 trait + 全局注册器
//!
//! 定义通知发送接口，由 system-application::NotificationService 实现，
//! relay-application 等下游模块通过 trait 调用，无需直接依赖 system-application。
//!
//! 三种调用方式：
//! 1. [`notify`] / [`notify_with_ref`] —— 直接给定标题与正文（手工拼文案场景）
//! 2. [`notify_template`] —— 按模板编码 + 变量表发送给指定用户；
//!    同一编码可同时配置站内信版与邮件版模板，按各自 channel 分别落库 / 投递
//! 3. [`send_email_template`] —— 按模板给指定邮箱直接发信并返回结果，
//!    用于注册 / 找回密码等"收件人尚未是系统用户"且需要感知失败的场景

use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use crate::error::AppError;

#[async_trait::async_trait]
pub trait NotifySender: Send + Sync {
    /// 发送通知给指定用户
    async fn send(
        &self,
        user_id: &str,
        notify_type: &str,
        title: &str,
        content: &str,
    ) -> Result<(), AppError>;

    /// 发送通知给指定用户（带关联信息）
    async fn send_with_ref(
        &self,
        user_id: &str,
        notify_type: &str,
        title: &str,
        content: &str,
        ref_type: Option<&str>,
        ref_id: Option<&str>,
    ) -> Result<(), AppError>;

    /// 按模板编码给指定用户发送：
    /// 查启用模板 → `${varName}` 渲染 → 按模板 channel 分发
    /// （in_app 落库站内信；email 投递到该用户邮箱）
    ///
    /// `ref_type` / `ref_id` 为可选业务关联（如 ("kyc", 申请id)），便于前端跳转详情。
    async fn send_template(
        &self,
        user_id: &str,
        template_code: &str,
        vars: HashMap<String, String>,
        ref_type: Option<&str>,
        ref_id: Option<&str>,
    ) -> Result<(), AppError>;

    /// 按模板编码给指定邮箱直接发信（不要求收件人已是系统用户）
    async fn send_email_template(
        &self,
        to: &str,
        template_code: &str,
        vars: HashMap<String, String>,
    ) -> Result<(), AppError>;
}

fn registry() -> &'static OnceLock<Arc<dyn NotifySender>> {
    static S: OnceLock<Arc<dyn NotifySender>> = OnceLock::new();
    &S
}

/// 安装全局通知发送器（应用启动时调用，幂等）
pub fn install_notify_sender(sender: Arc<dyn NotifySender>) {
    let _ = registry().set(sender);
}

/// 获取全局通知发送器
pub fn get_notify_sender() -> Option<Arc<dyn NotifySender>> {
    registry().get().cloned()
}

/// 发送通知（便捷方法，未安装时静默忽略）
pub async fn notify(user_id: &str, notify_type: &str, title: &str, content: &str) {
    if let Some(sender) = get_notify_sender() {
        let _ = sender.send(user_id, notify_type, title, content).await;
    }
}

/// 发送通知带关联（便捷方法，未安装时静默忽略）
pub async fn notify_with_ref(
    user_id: &str,
    notify_type: &str,
    title: &str,
    content: &str,
    ref_type: Option<&str>,
    ref_id: Option<&str>,
) {
    if let Some(sender) = get_notify_sender() {
        let _ = sender
            .send_with_ref(user_id, notify_type, title, content, ref_type, ref_id)
            .await;
    }
}

/// 按模板发送给指定用户（便捷方法，未安装时静默忽略）
pub async fn notify_template(
    user_id: &str,
    template_code: &str,
    vars: HashMap<String, String>,
    ref_type: Option<&str>,
    ref_id: Option<&str>,
) {
    if let Some(sender) = get_notify_sender() {
        let _ = sender
            .send_template(user_id, template_code, vars, ref_type, ref_id)
            .await;
    }
}

/// 按模板给指定邮箱发信（返回结果，调用方需感知失败，如验证码发送）
pub async fn send_email_template(
    to: &str,
    template_code: &str,
    vars: HashMap<String, String>,
) -> Result<(), AppError> {
    match get_notify_sender() {
        Some(sender) => sender.send_email_template(to, template_code, vars).await,
        None => Err(AppError::Internal("@processor_not_initialized".to_string())),
    }
}
