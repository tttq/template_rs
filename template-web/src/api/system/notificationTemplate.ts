import request from '../request'

/** 通知模板（/api/system/notification-template*）：邮件 / 站内消息模板管理 */

export interface NotificationTemplateVo {
  id: string
  /** 模板编码（业务引用，如 kyc_approved） */
  templateCode: string
  templateName: string
  /** 通知类型：system / kyc / recharge / invoice / team ... */
  notifyType: string
  /** 渠道：in_app（站内信）/ email（邮件） */
  channel: string
  /** 标题模板（邮件渠道时作为邮件主题） */
  titleTemplate?: string
  /** 正文模板，支持 ${varName} 占位 */
  contentTemplate: string
  /** 可用变量提示，仅展示 */
  varsHint?: string
  /** 1 启用 / 0 停用 */
  status: number
  remark?: string
  createTime: string
  updateTime: string
}

export interface NotificationTemplateQuery {
  templateCode?: string
  templateName?: string
  notifyType?: string
  channel?: string
  status?: number
  page?: number
  pageSize?: number
}

export interface NotificationTemplateSaveRequest {
  templateCode: string
  templateName: string
  notifyType: string
  channel: string
  titleTemplate?: string
  contentTemplate: string
  varsHint?: string
  remark?: string
}

export interface NotificationTemplatePreviewRequest {
  titleTemplate?: string
  contentTemplate: string
  vars: Record<string, string>
}

export interface NotificationTemplatePreviewVo {
  title: string
  content: string
  /** 模板中出现但变量表未提供的占位符 */
  missingVars: string[]
}

export const notificationTemplateApi = {
  list: (params?: NotificationTemplateQuery) =>
    request.get('/system/notification-template', { params }),
  getById: (id: string) => request.get<NotificationTemplateVo>(`/system/notification-template/${id}`),
  create: (data: NotificationTemplateSaveRequest) =>
    request.post('/system/notification-template', data),
  update: (id: string, data: NotificationTemplateSaveRequest) =>
    request.put(`/system/notification-template/${id}`, data),
  setStatus: (id: string, status: number) =>
    request.put(`/system/notification-template/${id}/status`, { status }),
  remove: (id: string) => request.delete(`/system/notification-template/${id}`),
  /** 试渲染：用草稿内容 + 变量表预览最终文案，并提示缺失变量 */
  preview: (data: NotificationTemplatePreviewRequest) =>
    request.post<NotificationTemplatePreviewVo>('/system/notification-template/preview', data),
}
