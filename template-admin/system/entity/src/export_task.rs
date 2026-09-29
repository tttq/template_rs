//! auth_sys_export_task — 通用异步导出任务（导出中心：提交 → 后台生成 Excel → 完成可下载）
//!
//! 系统管理内的通用导出中心设施：按 `task_type` 注册异步函数（如采购 `product`），
//! 业务模块负责具体生成逻辑，本表只记录任务台账 / 状态 / 生成文件。

use common::datetime_format;
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveAutoFillSoftDeleteTenant;
use serde::{Deserialize, Serialize};

/// 导出任务实体。
/// `status` 取值：`pending`（等待）/ `processing`（生成中）/ `success`（可下载）/ `failed`（失败，看 error_msg）。
#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFillSoftDeleteTenant)]
#[sea_orm(table_name = "auth_sys_export_task")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: String,
    /// 业务单号（如 EXP20260101...）
    pub task_no: String,
    /// 任务类型：product（选品带图导出）等，由注册的异步函数处理
    pub task_type: String,
    /// 筛选参数（JSON 文本，创建时原样保存）
    pub query: Option<String>,
    /// 状态：pending / processing / success / failed
    pub status: String,
    /// 总计行数
    pub total_rows: i32,
    /// 成功行数
    pub success_rows: i32,
    /// 失败行数
    pub error_rows: i32,
    /// 失败原因
    pub error_msg: Option<String>,
    /// 生成文件在附件中心（auth_sys_attachment）的附件 id（生成成功后上传统一存储）
    pub file_path: Option<String>,
    /// 下载文件名
    pub file_name: Option<String>,
    /// 文件字节数
    pub file_size: Option<i64>,
    #[serde(with = "datetime_format")]
    #[sea_orm_ext(insert)]
    pub create_time: DateTimeUtc,
    #[sea_orm_ext(insert)]
    pub create_by: Option<String>,
    #[sea_orm_ext(insert)]
    pub create_id: Option<String>,
    #[serde(with = "datetime_format")]
    #[sea_orm_ext(update)]
    pub update_time: DateTimeUtc,
    #[sea_orm_ext(update)]
    pub update_by: Option<String>,
    #[sea_orm_ext(update)]
    pub update_id: Option<String>,
    #[sea_orm(version)]
    #[sea_orm_ext(insert_update)]
    pub version: i32,
    #[soft_delete(default = 0, del = 1)]
    pub delete_flag: i32,
    #[sea_orm_ext(TENANT)]
    pub tenant_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
