use common::error::AppError;
use common::pagination::PageResult;
use sea_orm::ActiveValue::Set;
use sea_orm::prelude::*;
use sea_orm::{ColumnTrait, PaginatorTrait, QueryFilter, QueryOrder, QuerySelect};
use sea_orm_ext::DbConn;
use summer::plugin::service::Service;
use system_entity::attachment;
use system_infrastructure::StorageService;
use std::collections::HashMap;

use super::dto::*;
use super::image_compress;

const ALLOWED_MIME: &[(&str, &str)] = &[
    ("image/jpeg", "jpg"),
    ("image/png", "png"),
    ("image/gif", "gif"),
    ("image/webp", "webp"),
    ("image/bmp", "bmp"),
    ("image/svg+xml", "svg"),
    ("application/pdf", "pdf"),
    ("application/msword", "doc"),
    ("application/vnd.openxmlformats-officedocument.wordprocessingml.document", "docx"),
    ("application/vnd.ms-excel", "xls"),
    ("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", "xlsx"),
    ("application/vnd.ms-powerpoint", "ppt"),
    ("application/vnd.openxmlformats-officedocument.presentationml.presentation", "pptx"),
    ("text/plain", "txt"),
    ("text/csv", "csv"),
    ("application/zip", "zip"),
    ("application/x-rar-compressed", "rar"),
    ("video/mp4", "mp4"),
    ("video/webm", "webm"),
    ("audio/mpeg", "mp3"),
    ("audio/wav", "wav"),
];

const MAX_FILE_SIZE: usize = 50 * 1024 * 1024;

#[derive(Clone, Service)]
pub struct AttachmentService {
    #[inject(component)]
    db: DbConn,
    #[inject(component)]
    storage: StorageService,
}

impl AttachmentService {
    fn fmt_time(dt: &chrono::DateTime<chrono::Utc>) -> String {
        dt.format("%Y-%m-%d %H:%M:%S").to_string()
    }

    fn to_vo(&self, m: &attachment::Model) -> AttachmentVo {
        AttachmentVo {
            url: Some(format!("/api/system/attachment/{}", m.id)),
            id: m.id.clone(),
            category: m.category.clone(),
            biz_type: m.biz_type.clone(),
            biz_id: m.biz_id.clone(),
            image_type: m.image_type.clone(),
            sort: m.sort,
            caption: m.caption.clone(),
            original_name: m.original_name.clone(),
            mime_type: m.mime_type.clone(),
            file_size: m.file_size,
            storage_path: m.storage_path.clone(),
            uploader_id: m.uploader_id.clone(),
            status: m.status.clone(),
            remark: m.remark.clone(),
            create_time: Self::fmt_time(&m.create_time),
            create_by: m.create_by.clone(),
        }
    }

    fn get_ext(&self, mime_type: &str) -> Result<String, AppError> {
        ALLOWED_MIME
            .iter()
            .find(|(m, _)| *m == mime_type)
            .map(|(_, e)| String::from(*e))
            .ok_or_else(|| AppError::BadRequest("@file_type_unsupported".to_string()))
    }

    pub async fn upload(
        &self,
        uploader_id: &str,
        uploader_name: &str,
        category: &str,
        biz_type: &str,
        biz_id: Option<&str>,
        image_type: Option<&str>,
        sort: i32,
        caption: Option<&str>,
        original_name: &str,
        mime_type: &str,
        remark: Option<&str>,
        data: Vec<u8>,
    ) -> Result<AttachmentVo, AppError> {
        self.upload_inner(
            uploader_id,
            uploader_name,
            category,
            biz_type,
            biz_id,
            image_type,
            sort,
            caption,
            original_name,
            mime_type,
            remark,
            data,
            MAX_FILE_SIZE,
        )
        .await
    }

    /// 内部大文件上传（导出中心把生成的 Excel 落附件中心等场景），不受 50MB 上限约束；
    /// 仅服务端内部调用（业务直接落库 + 对象存储，不走 /api/system/attachment 上传接口）。
    pub async fn upload_big(
        &self,
        uploader_id: &str,
        uploader_name: &str,
        category: &str,
        biz_type: &str,
        biz_id: Option<&str>,
        image_type: Option<&str>,
        sort: i32,
        caption: Option<&str>,
        original_name: &str,
        mime_type: &str,
        remark: Option<&str>,
        data: Vec<u8>,
    ) -> Result<AttachmentVo, AppError> {
        self.upload_inner(
            uploader_id,
            uploader_name,
            category,
            biz_type,
            biz_id,
            image_type,
            sort,
            caption,
            original_name,
            mime_type,
            remark,
            data,
            usize::MAX,
        )
        .await
    }

    /// 上传核心：无大小上限时按内部场景返回错误（不暴露给面向用户的消息）
    #[allow(clippy::too_many_arguments)]
    async fn upload_inner(
        &self,
        uploader_id: &str,
        uploader_name: &str,
        category: &str,
        biz_type: &str,
        biz_id: Option<&str>,
        image_type: Option<&str>,
        sort: i32,
        caption: Option<&str>,
        original_name: &str,
        mime_type: &str,
        remark: Option<&str>,
        data: Vec<u8>,
        max_size: usize,
    ) -> Result<AttachmentVo, AppError> {
        let ext = self.get_ext(mime_type)?;
        if data.len() > max_size {
            if max_size == usize::MAX {
                return Err(AppError::Internal("@file_too_large".to_string()));
            }
            return Err(AppError::BadRequest("@file_too_large_50mb".to_string()));
        }
        if data.is_empty() {
            return Err(AppError::BadRequest("@file_empty".to_string()));
        }

        // 图片压缩：位图超阈值时按长边等比缩放后重编码，在保证清晰度的前提下降低存储占用。
        // 格式不变，故 mime / 扩展名 / 原始文件名都无需调整；失败或压缩后更大则保留原图。
        let raw_len = data.len();
        let (data, compressed) = image_compress::compress(mime_type, data);
        if compressed {
            log::info!("[attachment] 图片已压缩 {original_name}: {raw_len} -> {} bytes", data.len());
        }

        let record = attachment::ActiveModel {
            category: Set(category.to_string()),
            biz_type: Set(biz_type.to_string()),
            biz_id: Set(biz_id.map(|s| s.to_string())),
            image_type: Set(image_type.map(|s| s.to_string())),
            sort: Set(sort),
            caption: Set(caption.map(|s| s.to_string())),
            original_name: Set(original_name.chars().take(255).collect()),
            storage_path: Set(String::new()),
            mime_type: Set(mime_type.to_string()),
            file_size: Set(data.len() as i64),
            uploader_id: Set(Some(uploader_id.to_string())),
            status: Set("normal".to_string()),
            remark: Set(remark.map(|s| s.to_string())),
            create_by: Set(Some(uploader_name.to_string())),
            ..Default::default()
        };
        let created = record.insert(&self.db).await?;
        let id = created.id.clone();

        let ym = chrono::Utc::now().format("%Y%m").to_string();
        let storage_path = format!("{}/{}/{}/{}.{}", category, biz_type, ym, id, ext);
        self.storage.put(&storage_path, data, mime_type).await.map_err(|e| {
            let db = self.db.clone();
            let fid = id.clone();
            tokio::spawn(async move {
                // 上传失败：这条记录是多余的，直接物理删除
                // （注意 `delete_by_id` 会被软删宏改写成 UPDATE，必须用 `delete_many`）
                let _ = attachment::Entity::delete_many()
                    .filter(attachment::Column::Id.eq(fid))
                    .exec(&db)
                    .await;
            });
            e
        })?;

        let mut am: attachment::ActiveModel = created.into();
        am.storage_path = Set(storage_path);
        let updated = am.update(&self.db).await?;

        Ok(self.to_vo(&updated))
    }

    /// 读取附件元信息（含 uploader_id，供门户归属校验与管理端展示；不读取对象内容）
    pub async fn get(&self, id: &str) -> Result<AttachmentVo, AppError> {
        let m = attachment::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@attachment_not_found".to_string()))?;
        Ok(self.to_vo(&m))
    }

    pub async fn read(&self, id: &str) -> Result<(Vec<u8>, String, String), AppError> {
        let m = attachment::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@attachment_not_found".to_string()))?;
        let bytes = self.storage.get(&m.storage_path).await?;
        Ok((bytes, m.mime_type, m.original_name))
    }

    /// 附件直读（附件 id + 业务 id 双因子校验，防跨业务越权读取）。
    ///
    /// 与 [`Self::read`] 的区别：调用方可传 `biz_type` / `biz_id`，
    /// 后端校验与附件落库归属一致（不一致返回 Forbidden），适合业务端"按归属取内容"。
    pub async fn read_with_biz(
        &self,
        id: &str,
        biz_type: Option<&str>,
        biz_id: Option<&str>,
    ) -> Result<(Vec<u8>, String, String), AppError> {
        let m = attachment::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@attachment_not_found".to_string()))?;
        if let Some(bt) = biz_type {
            if !bt.is_empty() && m.biz_type != bt {
                return Err(AppError::Forbidden("@attachment_biz_type_mismatch".to_string()));
            }
        }
        if let Some(bid) = biz_id {
            if !bid.is_empty() && m.biz_id.as_deref() != Some(bid) {
                return Err(AppError::Forbidden("@attachment_biz_id_mismatch".to_string()));
            }
        }
        let bytes = self.storage.get(&m.storage_path).await?;
        Ok((bytes, m.mime_type, m.original_name))
    }

    /// 更新附件元信息（排序 / 主图标记）：仅更新传入字段
    pub async fn update_meta(
        &self,
        id: &str,
        req: &AttachmentMetaUpdateRequest,
    ) -> Result<AttachmentVo, AppError> {
        let m = attachment::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@attachment_not_found".to_string()))?;
        let mut am: attachment::ActiveModel = m.into();
        if let Some(sort) = req.sort {
            am.sort = Set(sort);
        }
        if let Some(image_type) = &req.image_type {
            am.image_type = Set(Some(image_type.to_string()));
        }
        let updated = am.update(&self.db).await?;
        Ok(self.to_vo(&updated))
    }

    pub async fn list(&self, query: &AttachmentQuery) -> Result<PageResult<AttachmentVo>, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(200);

        let mut select = attachment::Entity::find()
            .order_by_desc(attachment::Column::CreateTime);

        if let Some(ref cat) = query.category {
            if !cat.is_empty() {
                select = select.filter(attachment::Column::Category.eq(cat));
            }
        }
        if let Some(ref bt) = query.biz_type {
            if !bt.is_empty() {
                select = select.filter(attachment::Column::BizType.eq(bt));
            }
        }
        if let Some(ref bid) = query.biz_id {
            if !bid.is_empty() {
                select = select.filter(attachment::Column::BizId.eq(bid));
            }
        }

        let total = select.clone().count(&self.db).await?;
        let items = select
            .offset((page - 1) * page_size)
            .limit(page_size)
            .all(&self.db)
            .await?;

        let vos: Vec<AttachmentVo> = items.iter().map(|m| self.to_vo(m)).collect();
        Ok(PageResult::new(vos, total, page, page_size))
    }

    pub async fn list_by_biz(
        &self,
        category: &str,
        biz_type: &str,
        biz_id: Option<&str>,
    ) -> Result<Vec<AttachmentVo>, AppError> {
        let mut select = attachment::Entity::find()
            .filter(attachment::Column::Category.eq(category))
            .filter(attachment::Column::BizType.eq(biz_type))
            .order_by_asc(attachment::Column::Sort)
            .order_by_asc(attachment::Column::CreateTime);

        if let Some(bid) = biz_id {
            if !bid.is_empty() {
                select = select.filter(attachment::Column::BizId.eq(bid));
            }
        }

        let items = select.all(&self.db).await?;
        Ok(items.iter().map(|m| self.to_vo(m)).collect())
    }

    /// 批量按 biz_id 分组读取附件（一次查询避免列表 N+1）：
    /// 返回 `biz_id → 附件列表` 映射。
    pub async fn list_by_biz_map(
        &self,
        category: &str,
        biz_type: &str,
        biz_ids: &[&str],
    ) -> Result<HashMap<String, Vec<AttachmentVo>>, AppError> {
        let ids: Vec<String> = biz_ids.iter().filter(|s| !s.is_empty()).map(|s| String::from(*s)).collect();
        if ids.is_empty() {
            return Ok(HashMap::new());
        }
        let items = attachment::Entity::find()
            .filter(attachment::Column::Category.eq(category))
            .filter(attachment::Column::BizType.eq(biz_type))
            .filter(attachment::Column::BizId.is_in(ids))
            .order_by_asc(attachment::Column::Sort)
            .order_by_asc(attachment::Column::CreateTime)
            .all(&self.db)
            .await?;
        let mut map: HashMap<String, Vec<AttachmentVo>> = HashMap::new();
        for item in items {
            if let Some(bid) = &item.biz_id {
                map.entry(bid.clone()).or_default().push(self.to_vo(&item));
            }
        }
        Ok(map)
    }

    pub async fn first_image_urls(
        &self,
        category: &str,
        biz_type: &str,
        biz_ids: &[String],
    ) -> HashMap<String, String> {
        if biz_ids.is_empty() {
            return HashMap::new();
        }
        let items = attachment::Entity::find()
            .filter(attachment::Column::Category.eq(category))
            .filter(attachment::Column::BizType.eq(biz_type))
            .filter(attachment::Column::BizId.is_in(biz_ids.to_vec()))
            .filter(attachment::Column::ImageType.eq("main"))
            .filter(attachment::Column::Status.eq("normal"))
            .filter(attachment::Column::DeleteFlag.eq(0))
            .order_by_asc(attachment::Column::Sort)
            .order_by_asc(attachment::Column::CreateTime)
            .all(&self.db)
            .await
            .unwrap_or_default();
        let mut map: HashMap<String, String> = HashMap::new();
        for item in items {
            if let Some(biz_id) = &item.biz_id {
                map.entry(biz_id.clone())
                    .or_insert_with(|| format!("/api/system/attachment/{}", item.id));
            }
        }

        // 回退：未标记主图（image_type != 'main'，如历史数据或第三方写入）的对象取第一张图片，
        // 避免"明明有图但列表首图为空"；只回退图片类型，防止把视频当缩略图渲染
        let missing: Vec<String> = biz_ids
            .iter()
            .filter(|id| !map.contains_key(*id))
            .cloned()
            .collect();
        if !missing.is_empty() {
            let fallback = attachment::Entity::find()
                .filter(attachment::Column::Category.eq(category))
                .filter(attachment::Column::BizType.eq(biz_type))
                .filter(attachment::Column::BizId.is_in(missing))
                .filter(attachment::Column::Status.eq("normal"))
                .filter(attachment::Column::MimeType.starts_with("image/"))
                .order_by_asc(attachment::Column::Sort)
                .order_by_asc(attachment::Column::CreateTime)
                .all(&self.db)
                .await
                .unwrap_or_default();
            for item in fallback {
                if let Some(biz_id) = &item.biz_id {
                    map.entry(biz_id.clone())
                        .or_insert_with(|| format!("/api/system/attachment/{}", item.id));
                }
            }
        }

        map
    }

    /// 删除附件：**先删对象存储文件、再物理删除记录**（强删除，不做软删除）。
    ///
    /// 两点关键：
    /// 1. 顺序：对象删除失败必须让调用方看到错误（接口返回「删除失败」），记录保持不动可重试；
    ///    原实现用 `let _ =` 吞掉错误并软删记录，正是「删了但文件还在」的根源；
    /// 2. 强删除：附件是「文件 + 记录」的一体物，软删只会留下永远不可见的僵尸行，
    ///    既查不到也删不掉（对象只能靠 `scripts/cleanup_orphan_objects.py` 兜底）。
    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        let m = attachment::Entity::find_by_id(id.to_string())
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::NotFound("@attachment_not_found".to_string()))?;
        self.remove_object(&m.storage_path).await?;
        self.hard_delete_row(&m.id).await?;
        Ok(())
    }

    /// 物理删除附件记录（强删除）。
    ///
    /// 必须用 `delete_many`：`Entity::delete_by_id` / `ActiveModel::delete` 会被 sea-orm-ext 的
    /// 软删宏改写成 `UPDATE ... SET delete_flag = 1`，只有 `delete_many` 走真正的 `DELETE`
    /// （批量软删的入口是 `delete_many_soft`）。表内租户过滤由宏在 `delete_many` 上自动叠加。
    async fn hard_delete_row(&self, id: &str) -> Result<u64, AppError> {
        let result = attachment::Entity::delete_many()
            .filter(attachment::Column::Id.eq(id))
            .exec(&self.db)
            .await?;
        Ok(result.rows_affected)
    }

    /// 删除对象存储文件。
    ///
    /// - `storage_path` 为空（历史/占位数据）直接跳过：空 key 会被 S3 当成桶级请求，绝不能下发；
    /// - 失败向上抛出：调用方与日志都必须看得见，避免产生无人知晓的残留对象。
    async fn remove_object(&self, storage_path: &str) -> Result<(), AppError> {
        let key = storage_path.trim();
        if key.is_empty() {
            log::warn!("附件 storage_path 为空，跳过对象删除（记录仍会删除）");
            return Ok(());
        }
        if let Err(e) = self.storage.delete(key).await {
            log::error!("对象存储删除失败 key={}: {}", key, e);
            return Err(e);
        }
        Ok(())
    }

    /// 批量删除：逐条删除，失败写日志并返回**首个**错误（原实现整体 `let _ =`，失败无声无息）
    pub async fn batch_delete(&self, ids: &[String]) -> Result<(), AppError> {
        let mut first_err: Option<AppError> = None;
        for id in ids {
            if let Err(e) = self.delete(id).await {
                log::warn!("批量删除附件失败 id={}: {}", id, e);
                if first_err.is_none() {
                    first_err = Some(e);
                }
            }
        }
        match first_err {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    /// （重）绑定附件归属：把一组附件关联到业务对象（biz_type + biz_id）。
    ///
    /// 用于"先上传后落位"的业务（如上报照片先传文件、报表提交时归位），
    /// 幂等：重复调用仅覆盖为同一归属，不影响存储对象。
    pub async fn bind_biz(
        &self,
        ids: &[String],
        biz_type: &str,
        biz_id: &str,
    ) -> Result<(), AppError> {
        use sea_orm::sea_query::Expr;
        for id in ids {
            attachment::Entity::update_many()
                .col_expr(
                    attachment::Column::BizType,
                    Expr::value(biz_type.to_string()),
                )
                .col_expr(attachment::Column::BizId, Expr::value(biz_id.to_string()))
                .filter(attachment::Column::Id.eq(id))
                .filter(attachment::Column::Status.eq("normal"))
                .filter(attachment::Column::DeleteFlag.eq(0))
                .exec(&self.db)
                .await?;
        }
        Ok(())
    }

    /// 按业务归属清理附件：软删除记录 + 删除对象存储文件，返回实际清理条数。
    ///
    /// 业务对象（选品 / 工厂 / 包装规格…）删除时调用。业务侧只软删自己的行、
    /// 不管附件中心，会造成对象存储里留下没人引用的图片（本次修复的核心场景）。
    /// 单条失败只记日志不中断：业务删除已经发生，这里做尽力而为的清理并留下日志线索。
    pub async fn delete_by_biz(&self, category: &str, biz_type: &str, biz_id: &str) -> u64 {
        let items = match attachment::Entity::find()
            .filter(attachment::Column::Category.eq(category))
            .filter(attachment::Column::BizType.eq(biz_type))
            .filter(attachment::Column::BizId.eq(biz_id))
            .all(&self.db)
            .await
        {
            Ok(items) => items,
            Err(e) => {
                log::warn!(
                    "按业务查询附件失败 {}/{}/{}: {}",
                    category, biz_type, biz_id, e
                );
                return 0;
            }
        };
        let mut deleted: u64 = 0;
        for item in items {
            match self.delete(&item.id).await {
                Ok(()) => deleted += 1,
                Err(e) => log::warn!(
                    "业务附件清理失败 id={} storage_path={}: {}",
                    item.id, item.storage_path, e
                ),
            }
        }
        if deleted > 0 {
            log::info!("已清理业务附件 {} 个（{}/{}/{}）", deleted, category, biz_type, biz_id);
        }
        deleted
    }
}