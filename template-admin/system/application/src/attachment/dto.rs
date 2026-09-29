use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentVo {
    pub id: String,
    pub url: Option<String>,
    pub category: String,
    pub biz_type: String,
    pub biz_id: Option<String>,
    pub image_type: Option<String>,
    pub sort: i32,
    pub caption: Option<String>,
    pub original_name: String,
    pub mime_type: String,
    pub file_size: i64,
    pub storage_path: String,
    pub uploader_id: Option<String>,
    pub status: String,
    pub remark: Option<String>,
    pub create_time: String,
    pub create_by: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentQuery {
    /// 查询条件 — 「业务域」（如 purchase）；对应数据库列 auth_sys_attachment.category；精确匹配（eq）
    pub category: Option<String>,
    /// 查询条件 — 「业务类型」（如 product/factory）；对应数据库列 auth_sys_attachment.biz_type；精确匹配（eq）
    pub biz_type: Option<String>,
    /// 查询条件 — 「业务记录 ID」；对应数据库列 auth_sys_attachment.biz_id；精确匹配（eq）
    pub biz_id: Option<String>,
    /// 页号（>=1）
    pub page: Option<u64>,
    /// 每页条数（默认 20，1..=200）
    pub page_size: Option<u64>,
}

/// 附件直读参数（附件 id + 业务 id 双因子校验，防跨业务越权读取）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentReadQuery {
    /// 附件业务类型，与业务 ID 双因子校验；对应数据库列 auth_sys_attachment.biz_type；精确匹配（eq）
    pub biz_type: Option<String>,
    /// 附件业务记录 ID，与业务类型双因子校验；对应数据库列 auth_sys_attachment.biz_id；精确匹配（eq）
    pub biz_id: Option<String>,
}

/// 附件元信息更新（排序 / 主图标记）：仅更新传入字段，用于图片位置调整
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentMetaUpdateRequest {
    pub sort: Option<i32>,
    pub image_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentUploadParams {
    pub category: String,
    pub biz_type: String,
    pub biz_id: Option<String>,
    pub image_type: Option<String>,
    pub sort: Option<i32>,
    pub caption: Option<String>,
    pub remark: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUploadResult {
    pub success: Vec<AttachmentVo>,
    pub failed: Vec<BatchUploadError>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchUploadError {
    pub file_name: String,
    pub error: String,
}