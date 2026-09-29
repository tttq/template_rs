//! 统一 Excel(xlsx) 导出（fast-excel_rs `ExportRunner` 封装）
//!
//! 全平台唯一的表格导出入口：表头 + 行 → xlsx 字节流（内存）。
//! 2026-09 起导出统一为 **xlsx**（不再提供 CSV 下载）：中文不乱码、数值/日期保持单元格类型，
//! 也让「功能页同步导出」与「导出中心异步导出」的文件格式一致。

use crate::error::AppError;
use fast_excel::{CellValue, ColumnDef, DynamicRow, ExportRunner, SheetOptions};

/// xlsx 的响应 MIME
pub const XLSX_MIME: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

/// 导出的同步上限（10 万条）：
/// - ≤ 该条数：功能页同步导出，直接下载（同时在导出中心留一条记录）；
/// - > 该条数：改走「导出中心」异步任务，生成后到导出中心下载。
///
/// 前端同名常量（`SYNC_EXPORT_MAX`）需与这里保持一致。
pub const MAX_SYNC_ROWS: u64 = 100_000;

/// 超过同步上限时的提示文案（前端/后端共用同一口径）
pub fn too_large_message(total: u64) -> String {
    format!(
        "数据量较大（{} 条），已转为异步导出，请稍后到「导出中心」查看",
        total
    )
}

/// 表头 + 行 → xlsx 字节（内存，单 sheet）
///
/// `sheet_name` 为工作表名；`headers` 与每行 `rows[i]` 按位置对齐（不足补空）。
pub fn xlsx_bytes(
    sheet_name: &str,
    headers: &[&str],
    rows: Vec<Vec<String>>,
) -> Result<Vec<u8>, AppError> {
    let columns: Vec<ColumnDef> = headers.iter().map(|h| ColumnDef::new(*h)).collect();
    let models: Vec<DynamicRow> = rows
        .into_iter()
        .map(|row| DynamicRow {
            entries: headers
                .iter()
                .enumerate()
                .map(|(i, h)| {
                    (
                        (*h).to_string(),
                        CellValue::Text(row.get(i).cloned().unwrap_or_default()),
                    )
                })
                .collect(),
        })
        .collect();
    let runner = ExportRunner::new(SheetOptions::new(sheet_name).columns(columns));
    runner
        .export_bytes(&models)
        .map_err(|e| AppError::Internal(format!("生成 Excel 失败: {}", e)))
}

/// 生成带时间戳的导出文件名，如 `users-export-20260101120000.xlsx`
pub fn xlsx_file_name(prefix: &str) -> String {
    format!(
        "{}-{}.xlsx",
        prefix,
        chrono::Utc::now().format("%Y%m%d%H%M%S")
    )
}
