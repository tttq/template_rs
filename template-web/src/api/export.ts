/**
 * Excel(xlsx) 导出（blob 下载，绕开 JSON 拦截器）
 */

import request from './request'
import { exportTaskApi } from './system/exportTask'

/** 通用文件下载（fetch blob，绕开 JSON 拦截器）：功能页导出与门户端导出共用 */
export async function downloadCsv(url: string, params: Record<string, any>, filename: string) {
  const token = localStorage.getItem('token')
  const tenantId = localStorage.getItem('tenantId')
  const query = new URLSearchParams()
  Object.entries(params).forEach(([k, v]) => {
    if (v !== undefined && v !== null && v !== '') query.append(k, String(v))
  })
  const resp = await fetch(`${url}?${query.toString()}`, {
    headers: {
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
      ...(tenantId ? { 'x-tenant-id': tenantId } : {}),
    },
  })
  if (!resp.ok) throw new Error(`导出失败（${resp.status}）`)
  const blob = await resp.blob()
  const link = document.createElement('a')
  link.href = URL.createObjectURL(blob)
  link.download = filename
  link.click()
  URL.revokeObjectURL(link.href)
}

/**
 * 导出的同步上限（10 万条，与后端 `common::xlsx::MAX_SYNC_ROWS` 一致）：
 * ≤ 上限同步导出直接下载；> 上限自动转异步导出，完成后到「导出中心」查看。
 */
export const SYNC_EXPORT_MAX = 100000

export interface ExportDispatchResult {
  /** sync=功能页内直接下载；async=已提交「导出中心」异步任务 */
  mode: 'sync' | 'async'
  /** 可导出行数 */
  total: number
}

/**
 * 通用列表导出分流（各功能页导入导出统一走这里）：
 * 1. 先调 `${syncUrl}/count` 拿可导出条数；
 * 2. ≤ SYNC_EXPORT_MAX → 功能页内直接下载（后端同时会在导出中心登记一条记录）；
 * 3. 超过 → 提交「导出中心」异步任务（后台生成后上传附件中心），由调用方提示用户自查导出中心。
 */
export async function exportList(opts: {
  /** 导出中心任务类型（后端已注册的 task_type：user/role/dept/dict/config/tenant/menu/factory/product 等） */
  taskType: string
  /** 与同步导出完全一致的筛选参数 */
  query: Record<string, any>
  /** 功能页同步导出地址（不含 /count），如 `/api/system/users/export` */
  syncUrl: string
  /** 同步下载的文件名 */
  filename: string
}): Promise<ExportDispatchResult> {
  // request 实例 baseURL 已是 /api，这里去掉前缀避免重复
  const countUrl = `${opts.syncUrl.replace(/^\/api/, '')}/count`
  const { total } = await request.get<{ total: number }>(countUrl, { params: opts.query })
  if (total > SYNC_EXPORT_MAX) {
    await exportTaskApi.create(opts.taskType, opts.query)
    return { mode: 'async', total }
  }
  await downloadCsv(opts.syncUrl, opts.query, opts.filename)
  return { mode: 'sync', total }
}
