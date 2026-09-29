//! 通用导出任务 DTO

use common::datetime_format;
use serde::{Deserialize, Serialize};
use system_entity::export_task;

/// 创建导出任务（通用中心：taskType + 筛选参数，立即返回任务状态）
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExportTaskDto {
    /// 任务类型（须已注册，如 `product` 选品带图导出）
    pub task_type: String,
    /// 筛选参数（JSON 对象，原样保存，由注册函数按自己约定的键解析）
    pub query: Option<serde_json::Value>,
}

/// 导出任务出参
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportTaskVo {
    pub id: String,
    pub task_no: String,
    pub task_type: String,
    /// pending / processing / success / failed
    pub status: String,
    pub total_rows: i32,
    pub success_rows: i32,
    pub error_rows: i32,
    pub error_msg: Option<String>,
    pub file_name: Option<String>,
    pub file_size: Option<i64>,
    pub query: Option<String>,
    #[serde(with = "datetime_format")]
    pub create_time: chrono::DateTime<chrono::Utc>,
}

impl From<export_task::Model> for ExportTaskVo {
    fn from(m: export_task::Model) -> Self {
        Self {
            id: m.id,
            task_no: m.task_no,
            task_type: m.task_type,
            status: m.status,
            total_rows: m.total_rows,
            success_rows: m.success_rows,
            error_rows: m.error_rows,
            error_msg: m.error_msg,
            file_name: m.file_name,
            file_size: m.file_size,
            query: m.query,
            create_time: m.create_time,
        }
    }
}
