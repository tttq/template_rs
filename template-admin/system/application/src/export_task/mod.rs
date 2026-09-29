//! 通用导出中心（系统管理内）：异步导出任务台账 + 分发。
//!
//! 具体生成逻辑由业务模块用 [`fast_excel::export_task!`] 注册为异步函数，
//! 导出中心按 `task_type` 直接调用；**没有执行器 / 注册表 / install() 样板**。

pub mod dto;
pub mod service;

pub use fast_excel::{
    ExportTaskContext, ExportTaskEntry, ExportTaskKind, ExportTaskOutput, export_task_entry,
    export_task_registered, registered_export_tasks, run_export_task,
};
pub use service::ExportTaskAppService;
