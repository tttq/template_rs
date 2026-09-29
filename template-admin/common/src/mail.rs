//! 邮件发送工具（基于 summer-mail 插件自动装配的 `Mailer`）
//!
//! 由 system（通知中心按模板投递）与 relay（门户邮箱验证码）等模块共用，
//! 避免各业务模块重复实现 SMTP 细节。
//!
//! 说明：
//! - `MailPlugin` 读取 `config/app.toml` 的 `[mail]`（含 `[mail.auth]` 授权码）自动装配 SMTP 客户端
//! - `stub = true` 时不联网（lettre stub 仅记日志），本地联调无需真实邮箱
//! - 发件人取 `[mail]` 的认证账号（网易 163 等要求 From 与登录账号一致）

use crate::error::AppError;
use summer_mail::config::MailerConfig;
use summer_mail::header::ContentType;
use summer_mail::{AsyncTransport, Mailer, Message};

/// 邮件显示名（From 头部的友好名称）
pub const FROM_DISPLAY_NAME: &str = "Template Admin";

/// 解析发件人邮箱：优先取 SMTP 认证账号（`[mail.auth].user`），
/// 未配置时回退到占位地址（真实发送前请保证已配置）。
pub fn resolve_sender(cfg: &MailerConfig) -> String {
    cfg.transport
        .as_ref()
        .and_then(|t| t.auth.as_ref())
        .map(|a| a.user.clone())
        .filter(|u| !u.trim().is_empty())
        .unwrap_or_else(|| "no-reply@localhost".to_string())
}

/// 发送纯文本邮件（发件人带显示名）
pub async fn send_text(
    mailer: &Mailer,
    from: &str,
    to: &str,
    subject: &str,
    body: &str,
) -> Result<(), AppError> {
    let from_header = format!("{} <{from}>", FROM_DISPLAY_NAME);
    let email = Message::builder()
        .from(
            from_header
                .parse()
                .map_err(|e| AppError::Internal(format!("@mail_from_invalid:{e}")))?,
        )
        .to(
            to.trim()
                .parse()
                .map_err(|e| AppError::Internal(format!("@mail_to_invalid:{e}")))?,
        )
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body.to_string())
        .map_err(|e| AppError::Internal(format!("@mail_build_failed:{e}")))?;

    mailer
        .send(email)
        .await
        .map_err(|e| AppError::Internal(format!("@mail_send_failed:{e}")))?;
    Ok(())
}
