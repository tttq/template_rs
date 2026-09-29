//! 统一通知中心服务
//!
//! 负责通知落库（站内信）与渠道投递（邮件）。文案来源有两种：
//! - 直接给定 `title` / `content`（手工发送、广播）
//! - 按 `template_code` 取 `auth_sys_notification_template` 中的模板，用 `${varName}` 变量渲染
//!
//! 邮件渠道会真实投递到收件人邮箱；投递失败不回滚站内信，
//! 而是把失败原因写入 `send_error`，便于在通知列表排查。

use std::collections::HashMap;

use common::error::AppError;
use common::mail;
use common::notify::NotifySender;
use common::pagination::PageResult;
use common::template;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, EntityTrait, PaginatorTrait, QueryFilter, QueryOrder,
    QuerySelect,
};
use sea_orm_ext::DbConn;
use summer::plugin::service::Service;
use summer_mail::config::MailerConfig;
use summer_mail::Mailer;
use system_entity::{notification, user};

use super::dto::*;
use super::template_service::{NotificationTemplateService, CHANNEL_EMAIL, CHANNEL_IN_APP};

#[derive(Clone, Service)]
pub struct NotificationService {
    #[inject(component)]
    db: DbConn,
    /// 模板服务：按 template_code 取模板
    #[inject(component)]
    templates: NotificationTemplateService,
    /// SMTP 客户端（summer-mail 插件按 [mail] 配置自动装配）
    #[inject(component)]
    mailer: Mailer,
    #[inject(config)]
    mail_cfg: MailerConfig,
}

pub fn install_as_global(svc: std::sync::Arc<NotificationService>) {
    common::notify::install_notify_sender(svc);
}

#[async_trait::async_trait]
impl NotifySender for NotificationService {
    async fn send(
        &self,
        user_id: &str,
        notify_type: &str,
        title: &str,
        content: &str,
    ) -> Result<(), AppError> {
        self.send_impl(
            Some(user_id),
            notify_type,
            CHANNEL_IN_APP,
            title,
            content,
            None,
            None,
            None,
        )
        .await
        .map(|_| ())
    }

    async fn send_with_ref(
        &self,
        user_id: &str,
        notify_type: &str,
        title: &str,
        content: &str,
        ref_type: Option<&str>,
        ref_id: Option<&str>,
    ) -> Result<(), AppError> {
        self.send_impl(
            Some(user_id),
            notify_type,
            CHANNEL_IN_APP,
            title,
            content,
            None,
            ref_type,
            ref_id,
        )
        .await
        .map(|_| ())
    }

    async fn send_template(
        &self,
        user_id: &str,
        template_code: &str,
        vars: HashMap<String, String>,
        ref_type: Option<&str>,
        ref_id: Option<&str>,
    ) -> Result<(), AppError> {
        // 同一编码可同时配置站内信版与邮件版，逐条按各自渠道处理
        let templates = self.templates.list_active_by_code(template_code).await?;
        if templates.is_empty() {
            return Err(AppError::BadRequest(format!(
                "通知模板 {template_code} 不存在或已停用"
            )));
        }

        for tpl in templates {
            let title = template::render(tpl.title_template.as_deref().unwrap_or(""), &vars);
            let content = template::render(&tpl.content_template, &vars);
            self.send_impl(
                Some(user_id),
                &tpl.notify_type,
                &tpl.channel,
                &title,
                &content,
                Some(template_code),
                ref_type,
                ref_id,
            )
            .await?;
        }
        Ok(())
    }

    async fn send_email_template(
        &self,
        to: &str,
        template_code: &str,
        vars: HashMap<String, String>,
    ) -> Result<(), AppError> {
        let tpl = self
            .templates
            .find_active(template_code, CHANNEL_EMAIL)
            .await?
            .ok_or_else(|| {
                AppError::BadRequest(format!("@mail_template_not_found:{}", template_code))
            })?;

        let subject = template::render(tpl.title_template.as_deref().unwrap_or(""), &vars);
        let body = template::render(&tpl.content_template, &vars);
        self.deliver_email(to, &subject, &body).await
    }
}

impl NotificationService {
    /// 事务内落库通知（不投递邮件；send_status 直接置 sent）
    #[allow(clippy::too_many_arguments)]
    pub async fn send_tx<C: ConnectionTrait>(
        conn: &C,
        user_id: &str,
        notify_type: &str,
        channel: &str,
        title: &str,
        content: &str,
        ref_type: Option<&str>,
        ref_id: Option<&str>,
    ) -> Result<(), AppError> {
        notification::ActiveModel {
            user_id: Set(Some(user_id.to_string())),
            notify_type: Set(notify_type.to_string()),
            channel: Set(channel.to_string()),
            title: Set(title.to_string()),
            content: Set(content.to_string()),
            ref_type: Set(ref_type.map(|s| s.to_string())),
            ref_id: Set(ref_id.map(|s| s.to_string())),
            read_flag: Set(0),
            send_status: Set("sent".to_string()),
            ..Default::default()
        }
        .insert(conn)
        .await?;
        Ok(())
    }

    fn fmt_time(dt: &chrono::DateTime<chrono::Utc>) -> String {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    fn to_vo(m: &notification::Model) -> NotificationVo {
        NotificationVo {
            id: m.id.clone(),
            user_id: m.user_id.clone(),
            notify_type: m.notify_type.clone(),
            channel: m.channel.clone(),
            title: m.title.clone(),
            content: m.content.clone(),
            template_code: m.template_code.clone(),
            ref_type: m.ref_type.clone(),
            ref_id: m.ref_id.clone(),
            read_flag: m.read_flag,
            read_at: m.read_at.map(|t| Self::fmt_time(&t)),
            send_status: m.send_status.clone(),
            create_time: Self::fmt_time(&m.create_time),
        }
    }

    /// 落库 + 按渠道投递。
    ///
    /// 先写 `pending` 保证站内信一定可见，再按需投递邮件并回写 `sent` / `failed`。
    #[allow(clippy::too_many_arguments)]
    pub async fn send_impl(
        &self,
        user_id: Option<&str>,
        notify_type: &str,
        channel: &str,
        title: &str,
        content: &str,
        template_code: Option<&str>,
        ref_type: Option<&str>,
        ref_id: Option<&str>,
    ) -> Result<NotificationVo, AppError> {
        let record = notification::ActiveModel {
            user_id: Set(user_id.map(|s| s.to_string())),
            notify_type: Set(notify_type.to_string()),
            channel: Set(channel.to_string()),
            title: Set(title.to_string()),
            content: Set(content.to_string()),
            template_code: Set(template_code.map(|s| s.to_string())),
            ref_type: Set(ref_type.map(|s| s.to_string())),
            ref_id: Set(ref_id.map(|s| s.to_string())),
            read_flag: Set(0),
            send_status: Set("pending".to_string()),
            ..Default::default()
        };
        let created = record.insert(&self.db).await?;

        // 邮件渠道真实投递；失败不回滚站内信，仅记录原因
        let (status, error) = if channel == CHANNEL_EMAIL {
            match self.deliver_email_to_user(user_id, title, content).await {
                Ok(()) => ("sent".to_string(), None),
                Err(e) => {
                    log::warn!(
                        "通知邮件投递失败 template={:?} user={:?}: {}",
                        template_code,
                        user_id,
                        e
                    );
                    ("failed".to_string(), Some(e.to_string()))
                }
            }
        } else {
            ("sent".to_string(), None)
        };

        let mut am: notification::ActiveModel = created.into();
        am.send_status = Set(status);
        am.send_error = Set(error);
        let updated = am.update(&self.db).await?;

        Ok(Self::to_vo(&updated))
    }

    /// 按模板渲染出标题与正文（不发送）
    /// 供广播等"先渲染、再自行决定如何落库"的场景复用
    pub async fn render_template(
        &self,
        template_code: &str,
        channel: &str,
        vars: &HashMap<String, String>,
    ) -> Result<(String, String), AppError> {
        let tpl = self
            .templates
            .find_active(template_code, channel)
            .await?
            .ok_or_else(|| {
                AppError::BadRequest(format!(
                    "通知模板 {template_code}（{channel}）不存在或已停用"
                ))
            })?;

        Ok((
            template::render(tpl.title_template.as_deref().unwrap_or(""), vars),
            template::render(&tpl.content_template, vars),
        ))
    }

    /// 直接投递到指定邮箱
    async fn deliver_email(&self, to: &str, subject: &str, body: &str) -> Result<(), AppError> {
        let from = mail::resolve_sender(&self.mail_cfg);
        mail::send_text(&self.mailer, &from, to, subject, body).await
    }

    /// 取接收用户邮箱后投递
    async fn deliver_email_to_user(
        &self,
        user_id: Option<&str>,
        subject: &str,
        body: &str,
    ) -> Result<(), AppError> {
        let uid = user_id
            .ok_or_else(|| AppError::BadRequest("@email_recipient_required".to_string()))?;

        let u = user::Entity::find_by_id(uid.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@recipient_not_found".to_string()))?;

        let to = u
            .email
            .filter(|e| !e.trim().is_empty())
            .ok_or_else(|| AppError::BadRequest("@recipient_no_email".to_string()))?;

        self.deliver_email(&to, subject, body).await
    }

    /// 接收人候选列表（供通知中心的用户选择框检索）
    ///
    /// 与「系统用户管理」解耦：权限由 handler 的 `notification:send` 控制，
    /// 具备发送通知能力的角色即可检索接收人，无需再额外授予 `user:list`。
    /// 仅返回 id / 用户名 / 昵称，不外泄手机号、邮箱等敏感字段。
    pub async fn recipient_options(
        &self,
        keyword: Option<&str>,
        limit: u64,
    ) -> Result<Vec<RecipientOptionVo>, AppError> {
        let limit = limit.clamp(1, 50);
        let mut select = user::Entity::find()
            .filter(user::Column::Status.eq(1))
            .order_by_asc(user::Column::UserName);

        if let Some(kw) = keyword.map(str::trim).filter(|s| !s.is_empty()) {
            select = select.filter(
                Condition::any()
                    .add(user::Column::UserName.contains(kw))
                    .add(user::Column::NickName.contains(kw)),
            );
        }

        let items = select.limit(limit).all(&self.db).await?;
        Ok(items
            .into_iter()
            .map(|u| RecipientOptionVo {
                id: u.id,
                user_name: u.user_name,
                nick_name: u.nick_name,
            })
            .collect())
    }

    pub async fn my_list(
        &self,
        user_id: &str,
        query: &NotificationQuery,
    ) -> Result<PageResult<NotificationVo>, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(200);

        let mut select = notification::Entity::find()
            .filter(notification::Column::UserId.eq(user_id))
            .order_by_desc(notification::Column::CreateTime);

        if let Some(ref nt) = query.notify_type {
            if !nt.is_empty() {
                select = select.filter(notification::Column::NotifyType.eq(nt));
            }
        }
        if let Some(rf) = query.read_flag {
            select = select.filter(notification::Column::ReadFlag.eq(rf));
        }

        let total = select.clone().count(&self.db).await?;
        let items = select
            .offset((page - 1) * page_size)
            .limit(page_size)
            .all(&self.db)
            .await?;
        let vos: Vec<NotificationVo> = items.iter().map(|m| Self::to_vo(m)).collect();
        Ok(PageResult::new(vos, total, page, page_size))
    }

    pub async fn unread_count(&self, user_id: &str) -> Result<u64, AppError> {
        let count = notification::Entity::find()
            .filter(notification::Column::UserId.eq(user_id))
            .filter(notification::Column::ReadFlag.eq(0))
            .count(&self.db)
            .await?;
        Ok(count)
    }

    pub async fn mark_read(&self, user_id: &str, id: &str) -> Result<(), AppError> {
        let m = notification::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@notification_not_found".to_string()))?;
        if m.user_id.as_deref() != Some(user_id) {
            return Err(AppError::Forbidden("@operation_forbidden".to_string()));
        }
        let mut am: notification::ActiveModel = m.into();
        am.read_flag = Set(1);
        am.read_at = Set(Some(chrono::Utc::now()));
        am.update(&self.db).await?;
        Ok(())
    }

    pub async fn mark_all_read(&self, user_id: &str) -> Result<(), AppError> {
        notification::Entity::update_many()
            .col_expr(notification::Column::ReadFlag, sea_orm::sea_query::Expr::value(1))
            .col_expr(notification::Column::ReadAt, sea_orm::sea_query::Expr::value(chrono::Utc::now()))
            .filter(notification::Column::UserId.eq(user_id))
            .filter(notification::Column::ReadFlag.eq(0))
            .exec(&self.db)
            .await?;
        Ok(())
    }

    /// 广播（user_id 为 NULL，所有用户可见的公告）
    pub async fn broadcast(
        &self,
        notify_type: &str,
        channel: &str,
        title: &str,
        content: &str,
        template_code: Option<&str>,
    ) -> Result<(), AppError> {
        let record = notification::ActiveModel {
            user_id: Set(None),
            notify_type: Set(notify_type.to_string()),
            channel: Set(channel.to_string()),
            title: Set(title.to_string()),
            content: Set(content.to_string()),
            template_code: Set(template_code.map(|s| s.to_string())),
            read_flag: Set(0),
            send_status: Set("sent".to_string()),
            ..Default::default()
        };
        record.insert(&self.db).await?;
        Ok(())
    }
}
