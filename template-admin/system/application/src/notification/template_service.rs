//! 通知模板管理（auth_sys_notification_template）
//!
//! 提供模板 CRUD、试渲染，以及供通知发送链路按 `template_code + channel`
//! 取启用模板的内部方法。模板正文中的 `${varName}` 由 common::template::render 渲染。

use common::error::AppError;
use common::pagination::PageResult;
use common::template;
use sea_orm::ActiveModelTrait;
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use sea_orm_ext::DbConn;
use summer::plugin::service::Service;
use system_entity::notification_template;

use super::dto::*;

/// 支持的渠道：站内信 / 邮件
pub const CHANNEL_IN_APP: &str = "in_app";
pub const CHANNEL_EMAIL: &str = "email";

#[derive(Clone, Service)]
pub struct NotificationTemplateService {
    #[inject(component)]
    db: DbConn,
}

impl NotificationTemplateService {
    fn fmt_time(dt: &chrono::DateTime<chrono::Utc>) -> String {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    fn to_vo(m: &notification_template::Model) -> NotificationTemplateVo {
        NotificationTemplateVo {
            id: m.id.clone(),
            template_code: m.template_code.clone(),
            template_name: m.template_name.clone(),
            notify_type: m.notify_type.clone(),
            channel: m.channel.clone(),
            title_template: m.title_template.clone(),
            content_template: m.content_template.clone(),
            vars_hint: m.vars_hint.clone(),
            status: m.status,
            remark: m.remark.clone(),
            create_time: Self::fmt_time(&m.create_time),
            update_time: Self::fmt_time(&m.update_time),
        }
    }

    /// 入参校验：编码非空、渠道合法、正文非空
    fn validate(code: &str, channel: &str, content: &str) -> Result<(), AppError> {
        if code.is_empty() || code.len() > 64 {
            return Err(AppError::BadRequest("@template_code_format_invalid".to_string()));
        }
        if !code.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.') {
            return Err(AppError::BadRequest(
                "@template_code_chars_invalid".to_string(),
            ));
        }
        if !matches!(channel, CHANNEL_IN_APP | CHANNEL_EMAIL) {
            return Err(AppError::BadRequest(
                "@template_channel_invalid".to_string(),
            ));
        }
        if content.trim().is_empty() {
            return Err(AppError::BadRequest("@template_content_required".to_string()));
        }
        Ok(())
    }

    pub async fn list(
        &self,
        query: &NotificationTemplateQuery,
    ) -> Result<PageResult<NotificationTemplateVo>, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(10).clamp(1, 200);

        let mut select =
            notification_template::Entity::find().order_by_asc(notification_template::Column::TemplateCode);

        if let Some(v) = query.template_code.as_deref().filter(|s| !s.trim().is_empty()) {
            select = select.filter(notification_template::Column::TemplateCode.contains(v.trim()));
        }
        if let Some(v) = query.template_name.as_deref().filter(|s| !s.trim().is_empty()) {
            select = select.filter(notification_template::Column::TemplateName.contains(v.trim()));
        }
        if let Some(v) = query.notify_type.as_deref().filter(|s| !s.trim().is_empty()) {
            select = select.filter(notification_template::Column::NotifyType.eq(v));
        }
        if let Some(v) = query.channel.as_deref().filter(|s| !s.trim().is_empty()) {
            select = select.filter(notification_template::Column::Channel.eq(v));
        }
        if let Some(v) = query.status {
            select = select.filter(notification_template::Column::Status.eq(v));
        }

        let total = select.clone().count(&self.db).await?;
        let items = select
            .offset((page - 1) * page_size)
            .limit(page_size)
            .all(&self.db)
            .await?;
        let vos: Vec<NotificationTemplateVo> = items.iter().map(Self::to_vo).collect();
        Ok(PageResult::new(vos, total, page, page_size))
    }

    pub async fn get(&self, id: &str) -> Result<NotificationTemplateVo, AppError> {
        let m = notification_template::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@template_not_found".to_string()))?;
        Ok(Self::to_vo(&m))
    }

    pub async fn create(
        &self,
        req: NotificationTemplateSaveRequest,
    ) -> Result<NotificationTemplateVo, AppError> {
        let code = req.template_code.trim().to_string();
        let channel = req.channel.trim().to_string();
        Self::validate(&code, &channel, &req.content_template)?;

        let dup = notification_template::Entity::find()
            .filter(notification_template::Column::TemplateCode.eq(&code))
            .filter(notification_template::Column::Channel.eq(&channel))
            .one(&self.db)
            .await?;
        if dup.is_some() {
            return Err(AppError::BadRequest(format!(
                "模板编码 {code}（{channel}）已存在"
            )));
        }

        let am = notification_template::ActiveModel {
            template_code: Set(code),
            template_name: Set(req.template_name.trim().to_string()),
            notify_type: Set(req.notify_type.trim().to_string()),
            channel: Set(channel),
            title_template: Set(req.title_template.filter(|s| !s.trim().is_empty())),
            content_template: Set(req.content_template),
            vars_hint: Set(req.vars_hint.filter(|s| !s.trim().is_empty())),
            status: Set(1),
            remark: Set(req.remark.filter(|s| !s.trim().is_empty())),
            ..Default::default()
        };
        let created = am.insert(&self.db).await?;
        Ok(Self::to_vo(&created))
    }

    pub async fn update(
        &self,
        id: &str,
        req: NotificationTemplateSaveRequest,
    ) -> Result<NotificationTemplateVo, AppError> {
        let model = notification_template::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@template_not_found".to_string()))?;

        let code = req.template_code.trim().to_string();
        let channel = req.channel.trim().to_string();
        Self::validate(&code, &channel, &req.content_template)?;

        // 编码 / 渠道变更时需保证唯一（部分唯一索引，此处先查以便返回友好提示）
        if code != model.template_code || channel != model.channel {
            let dup = notification_template::Entity::find()
                .filter(notification_template::Column::TemplateCode.eq(&code))
                .filter(notification_template::Column::Channel.eq(&channel))
                .filter(notification_template::Column::Id.ne(id))
                .one(&self.db)
                .await?;
            if dup.is_some() {
                return Err(AppError::BadRequest(format!(
                    "模板编码 {code}（{channel}）已存在"
                )));
            }
        }

        let mut am: notification_template::ActiveModel = model.into();
        am.template_code = Set(code);
        am.template_name = Set(req.template_name.trim().to_string());
        am.notify_type = Set(req.notify_type.trim().to_string());
        am.channel = Set(channel);
        am.title_template = Set(req.title_template.filter(|s| !s.trim().is_empty()));
        am.content_template = Set(req.content_template);
        am.vars_hint = Set(req.vars_hint.filter(|s| !s.trim().is_empty()));
        am.remark = Set(req.remark.filter(|s| !s.trim().is_empty()));
        let updated = am.update(&self.db).await?;
        Ok(Self::to_vo(&updated))
    }

    /// 切换启用 / 停用
    pub async fn set_status(&self, id: &str, status: i32) -> Result<NotificationTemplateVo, AppError> {
        let model = notification_template::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@template_not_found".to_string()))?;

        let mut am: notification_template::ActiveModel = model.into();
        am.status = Set(if status == 1 { 1 } else { 0 });
        let updated = am.update(&self.db).await?;
        Ok(Self::to_vo(&updated))
    }

    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        let model = notification_template::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@template_not_found".to_string()))?;

        // 软删除：保留 update 审计字段（who / when deleted）
        let mut am: notification_template::ActiveModel = model.into();
        am.delete_flag = Set(1);
        am.update(&self.db).await?;
        Ok(())
    }

    /// 按编码 + 渠道取启用中的模板（通知发送链路使用）
    pub async fn find_active(
        &self,
        code: &str,
        channel: &str,
    ) -> Result<Option<notification_template::Model>, AppError> {
        Ok(notification_template::Entity::find()
            .filter(notification_template::Column::TemplateCode.eq(code))
            .filter(notification_template::Column::Channel.eq(channel))
            .filter(notification_template::Column::Status.eq(1))
            .one(&self.db)
            .await?)
    }

    /// 取某编码下所有启用中的模板
    /// （同一编码可同时配置站内信版与邮件版，一次发送分别落库 / 投递）
    pub async fn list_active_by_code(
        &self,
        code: &str,
    ) -> Result<Vec<notification_template::Model>, AppError> {
        Ok(notification_template::Entity::find()
            .filter(notification_template::Column::TemplateCode.eq(code))
            .filter(notification_template::Column::Status.eq(1))
            .all(&self.db)
            .await?)
    }

    /// 试渲染：使用草稿内容 + 变量表渲染，并回传缺失变量便于管理员排查
    pub fn preview(req: &NotificationTemplatePreviewRequest) -> NotificationTemplatePreviewVo {
        let title_tpl = req.title_template.clone().unwrap_or_default();
        let mut used: Vec<String> = template::placeholders(&title_tpl);
        for k in template::placeholders(&req.content_template) {
            if !used.iter().any(|u| u == &k) {
                used.push(k);
            }
        }
        let missing_vars: Vec<String> = used
            .into_iter()
            .filter(|k| !req.vars.contains_key(k))
            .collect();

        NotificationTemplatePreviewVo {
            title: template::render(&title_tpl, &req.vars),
            content: template::render(&req.content_template, &req.vars),
            missing_vars,
        }
    }
}
