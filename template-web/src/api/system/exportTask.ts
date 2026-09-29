import request from '../request'

/** 导出中心（系统管理内）：提交/列表/下载通用异步导出任务（下载走 fetch blob，绕开 JSON 拦截器） */

export interface ExportTaskVo {
  id: string
  taskNo: string
  taskType: string
  /** pending / processing / success / failed */
  status: string
  totalRows: number
  successRows: number
  errorRows: number
  errorMsg: string | null
  fileName: string | null
  fileSize: number | null
  createTime: string
}

const API_BASE = '/api'

export const exportTaskApi = {
  /** 提交导出任务（通用中心：taskType + 筛选参数 query） */
  create: (taskType: string, query: Record<string, any>) =>
    request.post<ExportTaskVo>('/system/exports', { taskType, query }),

  /** 我的导出任务列表（分页） */
  list: (params: Record<string, any>) => request.get('/system/exports', { params }),

  /** 任务详情（轮询状态用） */
  get: (id: string) => request.get<ExportTaskVo>(`/system/exports/${id}`),

  /** 下载已生成的文件 */
  download: async (id: string) => {
    const token = localStorage.getItem('token')
    const tenantId = localStorage.getItem('tenantId')
    const resp = await fetch(`${API_BASE}/system/exports/${id}/download`, {
      headers: {
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
        ...(tenantId ? { 'x-tenant-id': tenantId } : {}),
      },
    })
    if (!resp.ok) throw new Error(`下载失败（${resp.status}）`)
    const blob = await resp.blob()
    const cd = resp.headers.get('content-disposition') || ''
    const m = /filename="?([^"]+)"?/.exec(cd)
    const fileName = m ? m[1] : 'export.xlsx'
    const link = document.createElement('a')
    link.href = URL.createObjectURL(blob)
    link.download = fileName
    link.click()
    URL.revokeObjectURL(link.href)
  },
}