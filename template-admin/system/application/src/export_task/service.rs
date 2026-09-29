//! 通用导出任务服务（导出中心，系统管理内）
//!
//! 两种导出方式都在导出中心留记录、都能在「导出中心」看到并下载：
//! - **异步导出**（> 同步上限）：`create` 校验 `task_type` 已注册 + 同用户仅一个进行中任务 →
//!   落一条 pending 任务并 `tokio::spawn` 后台直接调用注册的异步函数 → 生成文件到
//!   `uploads/export/` → **生成文件上传到附件中心**（`auth_sys_attachment`，category=sys /
//!   biz_type=export_task，不受 50MB 上限约束，落统一存储而非本地临时盘）→ 删除本地临时文件 →
//!   更新任务 success，`file_path` 列复用存附件 id；下载/清理均走附件中心。失败记 failed + 原因。
//! - **同步导出**（≤ 同步上限，功能页直接下载）：功能页把生成好的文件交给
//!   [`ExportTaskAppService::record_sync_export`]，同样上传附件中心并落一条 success 任务，
//!   于是「导出中心」同时展示同步/异步两种结果。
//!
//! 约束：
//! - 生成逻辑按 `task_type` 直接查 `inventory` 注册的异步函数，无执行器概念；
//! - 同一用户同时只允许一个进行中的导出任务；
//! - 附件中心记录在创建新任务时懒清理（>7 天，删附件 + 删任务行）。

use std::path::PathBuf;

use common::error::AppError;
use common::get_current_user_id;
use common::pagination::{PageQuery, PageResult};
use sea_orm::ActiveValue::Set;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, PaginatorTrait, QueryFilter, QueryOrder,
};
use sea_orm_ext::DbConn;
use summer::plugin::service::Service;
use system_entity::export_task;

use crate::attachment::service::AttachmentService;
use super::dto::*;
use fast_excel::{ExportTaskContext, export_task_registered, run_export_task};

/// 生成文件目录（相对进程工作目录）
const EXPORT_DIR: &str = "uploads/export";
/// 生成文件保留天数
const KEEP_DAYS: i64 = 7;
/// 导出文件 MIME（xlsx）
const EXPORT_MIME: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
/// 同步导出登记用的备注
const SYNC_REMARK: &str = "同步导出（功能页直接下载）登记";

/// 按扩展名推断 MIME（任务未显式声明时用；导出中心同时支持 xlsx 与 csv）
fn mime_of(file_name: &str) -> &'static str {
    match file_name.rsplit('.').next().unwrap_or("").to_ascii_lowercase().as_str() {
        "csv" => "text/csv; charset=utf-8",
        _ => EXPORT_MIME,
    }
}

/// 生成业务单号（前缀 + 年月日时分秒毫秒），如 `EXP20260101120000000`
fn gen_no(prefix: &str) -> String {
    format!("{}{}", prefix, chrono::Utc::now().format("%Y%m%d%H%M%S%3f"))
}

/// 判断 `file_path` 存的是附件中心附件 id（纯数字雪花 ID）还是迁移前的本地文件路径。
///
/// 不要写死位数：雪花 ID 的十进制位数随生成器起始纪元而变，本项目实际生成 18 位，
/// 写死 19 位会把所有生成文件都判成「非附件」而无法下载。
fn is_attachment_id(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_digit())
}

#[derive(Clone, Service)]
pub struct ExportTaskAppService {
    #[inject(component)]
    db: DbConn,
    /// 统一附件中心：生成文件上传（大文件）与下载字节读取都走它
    #[inject(component)]
    attachment: AttachmentService,
}

impl ExportTaskAppService {
    /// 提交导出任务（立即返回；文件在后台生成，完成状态在导出中心可见）
    pub async fn create(&self, dto: CreateExportTaskDto) -> Result<ExportTaskVo, AppError> {
        let actor = current_actor();
        // 类型未注册直接报错（防止把任务落表后才失败）
        if !export_task_registered(&dto.task_type) {
            return Err(AppError::BadRequest(format!(
                "导出类型「{}」暂不支持",
                dto.task_type
            )));
        }
        // 同一用户同时只允许一个进行中的导出任务
        let running = export_task::Entity::find()
            .filter(export_task::Column::CreateId.eq(Some(actor.clone())))
            .filter(export_task::Column::Status.is_in(["pending", "processing"]))
            .count(&self.db)
            .await?;
        if running > 0 {
            return Err(AppError::BadRequest(
                "已有导出任务在进行中，请等待完成后再发起".to_string(),
            ));
        }
        // 懒清理过期文件
        self.cleanup_expired(&actor).await?;

        let query_json = dto.query.unwrap_or_else(|| serde_json::json!({}));
        let model = export_task::ActiveModel {
            task_no: Set(gen_no("EXP")),
            task_type: Set(dto.task_type),
            query: Set(Some(query_json.to_string())),
            status: Set("pending".to_string()),
            ..Default::default()
        };
        let created = model.insert(&self.db).await?;
        let task_id = created.id.clone();

        // 后台异步生成（进程内任务；失败回写任务状态）
        let svc = self.clone();
        tokio::spawn(async move {
            if let Err(e) = svc.generate(&task_id).await {
                log::warn!("导出任务 {} 生成失败: {}", task_id, e);
                let _ = svc.mark_failed(&task_id, &e.to_string()).await;
            }
        });
        Ok(ExportTaskVo::from(created))
    }

    /// 后台生成主体：置 processing → 调用注册函数 → 写文件 → 更新任务成功
    async fn generate(&self, task_id: &str) -> Result<(), AppError> {
        let model = export_task::Entity::find_by_id(task_id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("导出任务不存在".to_string()))?;
        let mut am: export_task::ActiveModel = model.clone().into();
        am.status = Set("processing".to_string());
        am.update(&self.db).await?;

        let query_json: serde_json::Value =
            serde_json::from_str(&model.query.clone().unwrap_or_default())
                .unwrap_or_else(|_| serde_json::json!({}));
        // 生成文件（此段无 await 之间持有本地借用，安全）
        let dir = export_dir();
        std::fs::create_dir_all(&dir)
            .map_err(|e| AppError::Internal(format!("创建导出目录失败: {}", e)))?;
        let output = run_export_task(
            &model.task_type,
            ExportTaskContext {
                task_no: model.task_no.clone(),
                query: query_json,
                out_dir: dir.clone(),
            },
        )
        .await
        .map_err(AppError::Internal)?;

        // 生成文件上传附件中心（大文件不限 50MB，落统一存储）；成功后删除本地临时文件
        let file_path = dir.join(&output.file_name);
        let bytes = std::fs::read(&file_path)
            .map_err(|e| AppError::Internal(format!("读取导出文件失败: {}", e)))?;
        let uploader = model.create_by.clone().unwrap_or_else(|| "system".to_string());
        let mime = output
            .mime
            .clone()
            .unwrap_or_else(|| mime_of(&output.file_name).to_string());
        let attached = self
            .attachment
            .upload_big(
                &uploader,
                &model.task_no,
                "sys",
                "export_task",
                Some(&model.id),
                None,
                0,
                None,
                &output.file_name,
                &mime,
                Some("导出中心生成文件"),
                bytes,
            )
            .await
            .map_err(|e| AppError::Internal(format!("导出文件上传附件中心失败: {}", e)))?;
        let _ = std::fs::remove_file(&file_path);

        // 更新任务成功（file_path 列复用存附件 id，下载/清理统一走附件中心）
        let mut am: export_task::ActiveModel = model.into();
        am.status = Set("success".to_string());
        am.total_rows = Set(output.total_rows);
        am.success_rows = Set(output.total_rows);
        am.error_rows = Set(0);
        am.error_msg = Set(None);
        am.file_path = Set(Some(attached.id));
        am.file_name = Set(Some(output.file_name));
        am.file_size = Set(Some(attached.file_size));
        am.update(&self.db).await?;
        Ok(())
    }

    /// 同步导出登记：功能页「≤ 同步上限直接下载」的结果同样落导出中心。
    ///
    /// 步骤：先落一条 `processing` 任务（保证导出中心一定看得到这次导出）→ 上传附件中心 →
    /// 更新为 `success`（`file_path` 存附件 id，下载走附件中心 `read`）。
    /// 生成/上传失败则置 `failed` 并写原因；调用方对错误**只记日志、不阻断同步下载**。
    pub async fn record_sync_export(
        &self,
        task_type: &str,
        query: &serde_json::Value,
        file_name: &str,
        mime: &str,
        total_rows: i32,
        bytes: &[u8],
    ) -> Result<(), AppError> {
        let actor = current_actor();
        // 懒清理过期记录（与异步 create 同一口径）
        let _ = self.cleanup_expired(&actor).await;

        let model = export_task::ActiveModel {
            task_no: Set(gen_no("EXP")),
            task_type: Set(task_type.to_string()),
            query: Set(Some(query.to_string())),
            status: Set("processing".to_string()),
            total_rows: Set(total_rows),
            ..Default::default()
        };
        let created = model.insert(&self.db).await?;

        let uploader = created.create_by.clone().unwrap_or_else(|| actor.clone());
        let mime = if mime.is_empty() { mime_of(file_name).to_string() } else { mime.to_string() };
        let attached = match self
            .attachment
            .upload_big(
                &uploader,
                &created.task_no,
                "sys",
                "export_task",
                Some(&created.id),
                None,
                0,
                None,
                file_name,
                &mime,
                Some(SYNC_REMARK),
                bytes.to_vec(),
            )
            .await
        {
            Ok(a) => a,
            Err(e) => {
                let _ = self
                    .mark_failed(&created.id, &format!("同步导出文件上传附件中心失败: {e}"))
                    .await;
                return Err(e);
            }
        };

        let mut am: export_task::ActiveModel = created.into();
        am.status = Set("success".to_string());
        am.success_rows = Set(total_rows);
        am.error_rows = Set(0);
        am.error_msg = Set(None);
        am.file_path = Set(Some(attached.id));
        am.file_name = Set(Some(file_name.to_string()));
        am.file_size = Set(Some(attached.file_size));
        am.update(&self.db).await?;
        Ok(())
    }

    async fn mark_failed(&self, task_id: &str, msg: &str) -> Result<(), AppError> {
        let model = export_task::Entity::find_by_id(task_id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("导出任务不存在".to_string()))?;
        let mut am: export_task::ActiveModel = model.into();
        am.status = Set("failed".to_string());
        am.error_msg = Set(Some(msg.chars().take(900).collect()));
        am.update(&self.db).await?;
        Ok(())
    }

    /// 任务列表（仅当前用户自己的任务）
    ///
    /// 归属按 `create_id`（审计字段里的用户 ID）过滤：`create_by` 存的是用户名，
    /// 与 `current_actor()` 的 ID 口径不同，用它过滤会恒为空集。
    pub async fn list(&self, page_query: PageQuery) -> Result<PageResult<ExportTaskVo>, AppError> {
        let actor = current_actor();
        let select = export_task::Entity::find()
            .filter(export_task::Column::CreateId.eq(Some(actor)))
            .order_by_desc(export_task::Column::CreateTime);
        let paginator = select.paginate(&self.db, page_query.page_size);
        let total = paginator.num_items().await?;
        let items = paginator.fetch_page(page_query.page - 1).await?;
        let vos = items.into_iter().map(ExportTaskVo::from).collect();
        Ok(PageResult::new(vos, total, page_query.page, page_query.page_size))
    }

    /// 任务详情（仅本人）
    pub async fn get(&self, id: &str) -> Result<ExportTaskVo, AppError> {
        let model = self.own_task(id).await?;
        Ok(ExportTaskVo::from(model))
    }

    /// 取本人任务的生成文件字节（下载走附件中心：校验 status=success 且附件存在）
    ///
    /// 返回 `(文件字节, 下载文件名, MIME)`，MIME 由附件中心回读，xlsx / csv 各自正确。
    pub async fn download(&self, id: &str) -> Result<(Vec<u8>, String, String), AppError> {
        let model = self.own_task(id).await?;
        if model.status != "success" {
            return Err(AppError::BadRequest("导出任务尚未完成".to_string()));
        }
        let att_id = model
            .file_path
            .as_deref()
            .filter(|p| is_attachment_id(p))
            .ok_or_else(|| AppError::NotFound("导出文件不存在或已过期清理".to_string()))?;
        let (bytes, mime, _name) = self.attachment.read(att_id).await?;
        let file_name = model
            .file_name
            .unwrap_or_else(|| if mime.contains("csv") { "export.csv".to_string() } else { "export.xlsx".to_string() });
        Ok((bytes, file_name, mime))
    }

    /// 本人任务（归属校验）
    async fn own_task(&self, id: &str) -> Result<export_task::Model, AppError> {
        let actor = current_actor();
        let model = export_task::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("导出任务不存在".to_string()))?;
        if model.create_id.as_deref() != Some(actor.as_str()) {
            return Err(AppError::NotFound("导出任务不存在".to_string()));
        }
        Ok(model)
    }

    /// 懒清理：删除本人超过保留期的成功任务文件与记录
    async fn cleanup_expired(&self, actor: &str) -> Result<(), AppError> {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(KEEP_DAYS);
        let expired: Vec<export_task::Model> = export_task::Entity::find()
            .filter(export_task::Column::CreateId.eq(Some(actor.to_string())))
            .filter(export_task::Column::Status.eq("success"))
            .filter(export_task::Column::CreateTime.lt(cutoff))
            .all(&self.db)
            .await?;
        if expired.is_empty() {
            return Ok(());
        }
        let ids: Vec<String> = expired.iter().map(|m| m.id.clone()).collect();
        for m in &expired {
            if let Some(p) = &m.file_path {
                if is_attachment_id(p) {
                    // 附件中心的导出文件：删附件（对象存储 + 记录，强删除）
                    let _ = self.attachment.delete(p).await;
                } else {
                    // 历史本地路径兜底（2026-09 迁移前的任务）
                    let _ = std::fs::remove_file(PathBuf::from(p));
                }
            }
        }
        export_task::Entity::delete_many()
            .filter(export_task::Column::Id.is_in(ids))
            .exec(&self.db)
            .await?;
        Ok(())
    }
}

/// 当前登录用户 id（未登录回退 "system"，与采购 util 一致）
fn current_actor() -> String {
    get_current_user_id().unwrap_or_else(|| "system".to_string())
}

/// 导出文件目录（uploads/export，进程工作目录下）
fn export_dir() -> PathBuf {
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(EXPORT_DIR)
}
