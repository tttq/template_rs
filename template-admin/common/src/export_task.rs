//! 异步导出任务兼容门面：实现已下沉到 fast-excel_rs（`fast_excel::task`）。
//!
//! 业务侧注册直接使用 [`fast_excel::export_task!`] 宏；这里仅保留类型/函数转发，
//! 避免既有的 `common::export_task::...` 引用路径失效。
//! 不再有 `ExportTaskExecutor` trait、注册表、安装器或 `install()` 样板。

pub use fast_excel::{
    ExportTaskContext, ExportTaskEntry, ExportTaskFn, ExportTaskFuture, ExportTaskKind,
    ExportTaskOutput, XLSX_MIME, export_task_entry, export_task_registered,
    registered_export_tasks, run_export_task,
};
