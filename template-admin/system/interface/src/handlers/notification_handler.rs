//! 统一通知中心（/api/system/notification*）
//! 通知模板管理（/api/system/notification-template*）

use common::get_current_user_id;
use common::notify::NotifySender;
use common::response::ApiResponse;
use system_application::notification::dto::*;
use system_application::notification::service::NotificationService;
use system_application::notification::template_service::NotificationTemplateService;
use summer_sa_token::sa_check_permission;
use summer_web::axum::response::IntoResponse;
use summer_web::error::WebError;
use summer_web::extractor::{Component, Json, Path, Query};
use summer_web::{delete, get, nest, post, put};

#[nest("/system")]
mod controller {
    use super::*;

    // ---------------- 通知中心 ----------------

    /// 定向发送：二选一
    /// - 传 `templateCode` → 按模板渲染发送（模板 channel 决定落库 / 投递邮件）
    /// - 否则使用 `title` / `content` 原文
    ///
    /// 接收人支持多选（`userIds`），逐个发送并汇总结果；未指定接收人时仅落库（历史行为）。
    #[post("/notification")]
    #[sa_check_permission("notification:send")]
    async fn send(
        Component(service): Component<NotificationService>,
        Json(req): Json<NotificationSendRequest>,
    ) -> Result<impl IntoResponse, WebError> {
        let user_ids = req.resolved_user_ids();

        if let Some(code) = trimmed(&req.template_code) {
            if user_ids.is_empty() {
                return Ok(Json(ApiResponse::error(400, "@notif_recipient_required")));
            }
            let mut ok = 0usize;
            let mut failed: Vec<String> = Vec::new();
            for uid in &user_ids {
                match service
                    .send_template(
                        uid,
                        code,
                        req.vars.clone(),
                        trimmed(&req.ref_type),
                        trimmed(&req.ref_id),
                    )
                    .await
                {
                    Ok(_) => ok += 1,
                    Err(e) => failed.push(format!("{uid}: {e}")),
                }
            }
            return Ok(batch_response(ok, &failed));
        }

        let title = req.title.as_deref().unwrap_or("").trim().to_string();
        let content = req.content.as_deref().unwrap_or("").trim().to_string();
        if title.is_empty() || content.is_empty() {
            return Ok(Json(ApiResponse::error(400, "@notif_content_required")));
        }

        // 未指定接收人：保持历史行为（user_id 为 NULL，仅落库）
        if user_ids.is_empty() {
            return Ok(match service
                .send_impl(
                    None,
                    &req.notify_type,
                    &req.channel,
                    &title,
                    &content,
                    None,
                    trimmed(&req.ref_type),
                    trimmed(&req.ref_id),
                )
                .await
            {
                Ok(_) => Json(ApiResponse::success("@sent_ok".to_string())),
                Err(e) => Json(ApiResponse::error(500, &e.to_string())),
            });
        }

        let mut ok = 0usize;
        let mut failed: Vec<String> = Vec::new();
        for uid in &user_ids {
            match service
                .send_impl(
                    Some(uid),
                    &req.notify_type,
                    &req.channel,
                    &title,
                    &content,
                    None,
                    trimmed(&req.ref_type),
                    trimmed(&req.ref_id),
                )
                .await
            {
                Ok(_) => ok += 1,
                Err(e) => failed.push(format!("{uid}: {e}")),
            }
        }
        Ok(batch_response(ok, &failed))
    }

    #[get("/notification")]
    #[sa_check_permission("notification:list")]
    async fn my_list(
        Component(service): Component<NotificationService>,
        Query(query): Query<NotificationQuery>,
    ) -> Result<impl IntoResponse, WebError> {
        let uid = get_current_user_id().unwrap_or_default();
        Ok(match service.my_list(&uid, &query).await {
            Ok(v) => Json(ApiResponse::success(v)),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        })
    }

    #[get("/notification/unread-count")]
    #[sa_check_permission("notification:list")]
    async fn unread_count(
        Component(service): Component<NotificationService>,
    ) -> Result<impl IntoResponse, WebError> {
        let uid = get_current_user_id().unwrap_or_default();
        Ok(match service.unread_count(&uid).await {
            Ok(v) => Json(ApiResponse::success(v)),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        })
    }

    /// 接收人候选：供通知中心的用户选择框检索（按用户名 / 昵称过滤）
    ///
    /// 权限刻意与发送权限保持一致（`notification:send`）：
    /// 「能发通知」即「能选接收人」，无需再授予系统用户管理的 `user:list`。
    #[get("/notification/recipients")]
    #[sa_check_permission("notification:send")]
    async fn recipients(
        Component(service): Component<NotificationService>,
        Query(query): Query<RecipientQuery>,
    ) -> Result<impl IntoResponse, WebError> {
        Ok(match service
            .recipient_options(query.keyword.as_deref(), query.limit.unwrap_or(20))
            .await
        {
            Ok(v) => Json(ApiResponse::success(v)),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        })
    }

    #[put("/notification/{id}/read")]
    #[sa_check_permission("notification:list")]
    async fn mark_read(
        Component(service): Component<NotificationService>,
        Path(id): Path<String>,
    ) -> Result<impl IntoResponse, WebError> {
        let uid = get_current_user_id().unwrap_or_default();
        Ok(match service.mark_read(&uid, &id).await {
            Ok(_) => Json(ApiResponse::success("@read_ok")),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        })
    }

    #[put("/notification/read-all")]
    #[sa_check_permission("notification:list")]
    async fn mark_all_read(
        Component(service): Component<NotificationService>,
    ) -> Result<impl IntoResponse, WebError> {
        let uid = get_current_user_id().unwrap_or_default();
        Ok(match service.mark_all_read(&uid).await {
            Ok(_) => Json(ApiResponse::success("@all_read_ok")),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        })
    }

    /// 广播：指定模板时先渲染再落库（广播无具体收件人，不投递邮件）
    #[post("/notification/broadcast")]
    #[sa_check_permission("notification:broadcast")]
    async fn broadcast(
        Component(service): Component<NotificationService>,
        Json(req): Json<NotificationSendRequest>,
    ) -> Result<impl IntoResponse, WebError> {
        let (title, content) = if let Some(code) = trimmed(&req.template_code) {
            match service.render_template(code, &req.channel, &req.vars).await {
                Ok(v) => v,
                Err(e) => return Ok(Json(ApiResponse::error(500, &e.to_string()))),
            }
        } else {
            let t = req.title.as_deref().unwrap_or("").trim().to_string();
            let c = req.content.as_deref().unwrap_or("").trim().to_string();
            if t.is_empty() || c.is_empty() {
                return Ok(Json(ApiResponse::error(400, "@notif_content_required")));
            }
            (t, c)
        };

        Ok(match service
            .broadcast(
                &req.notify_type,
                &req.channel,
                &title,
                &content,
                trimmed(&req.template_code),
            )
            .await
        {
            Ok(_) => Json(ApiResponse::success("@broadcast_ok")),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        })
    }

    // ---------------- 通知模板管理 ----------------

    #[get("/notification-template")]
    #[sa_check_permission("notification:template:list")]
    async fn list_templates(
        Component(service): Component<NotificationTemplateService>,
        Query(query): Query<NotificationTemplateQuery>,
    ) -> Result<impl IntoResponse, WebError> {
        Ok(match service.list(&query).await {
            Ok(v) => Json(ApiResponse::success(v)),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        })
    }

    #[get("/notification-template/{id}")]
    #[sa_check_permission("notification:template:list")]
    async fn get_template(
        Component(service): Component<NotificationTemplateService>,
        Path(id): Path<String>,
    ) -> Result<impl IntoResponse, WebError> {
        Ok(match service.get(&id).await {
            Ok(v) => Json(ApiResponse::success(v)),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        })
    }

    #[post("/notification-template")]
    #[sa_check_permission("notification:template:add")]
    async fn create_template(
        Component(service): Component<NotificationTemplateService>,
        Json(req): Json<NotificationTemplateSaveRequest>,
    ) -> Result<impl IntoResponse, WebError> {
        Ok(match service.create(req).await {
            Ok(v) => Json(ApiResponse::success(v)),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        })
    }

    #[put("/notification-template/{id}")]
    #[sa_check_permission("notification:template:edit")]
    async fn update_template(
        Component(service): Component<NotificationTemplateService>,
        Path(id): Path<String>,
        Json(req): Json<NotificationTemplateSaveRequest>,
    ) -> Result<impl IntoResponse, WebError> {
        Ok(match service.update(&id, req).await {
            Ok(v) => Json(ApiResponse::success(v)),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        })
    }

    #[put("/notification-template/{id}/status")]
    #[sa_check_permission("notification:template:edit")]
    async fn set_template_status(
        Component(service): Component<NotificationTemplateService>,
        Path(id): Path<String>,
        Json(req): Json<NotificationTemplateStatusRequest>,
    ) -> Result<impl IntoResponse, WebError> {
        Ok(match service.set_status(&id, req.status).await {
            Ok(v) => Json(ApiResponse::success(v)),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        })
    }

    #[delete("/notification-template/{id}")]
    #[sa_check_permission("notification:template:delete")]
    async fn delete_template(
        Component(service): Component<NotificationTemplateService>,
        Path(id): Path<String>,
    ) -> Result<impl IntoResponse, WebError> {
        Ok(match service.delete(&id).await {
            Ok(_) => Json(ApiResponse::success("@deleted_ok")),
            Err(e) => Json(ApiResponse::error(500, &e.to_string())),
        })
    }

    /// 试渲染：可用草稿内容 + 变量表预览最终文案，并提示缺失变量
    #[post("/notification-template/preview")]
    #[sa_check_permission("notification:template:list")]
    async fn preview_template(
        Json(req): Json<NotificationTemplatePreviewRequest>,
    ) -> Result<impl IntoResponse, WebError> {
        Ok(Json(ApiResponse::success(NotificationTemplateService::preview(
            &req,
        ))))
    }
}

/// 取出非空且去除首尾空白的可选字符串引用
fn trimmed(v: &Option<String>) -> Option<&str> {
    v.as_deref().map(str::trim).filter(|s| !s.is_empty())
}

/// 批量发送结果：全部成功返回成功提示；存在失败时返回失败明细（便于定位哪个用户失败）
fn batch_response(ok: usize, failed: &[String]) -> Json<ApiResponse<String>> {
    if failed.is_empty() {
        Json(ApiResponse::success(format!("@sent_count_ok:{ok}")))
    } else {
        Json(ApiResponse::error(
            500,
            &format!(
                "成功 {ok} 条，失败 {} 条：{}",
                failed.len(),
                failed.join("；")
            ),
        ))
    }
}
