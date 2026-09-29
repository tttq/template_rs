import request from '../request'

/** 系统通知中心（/api/system/notification*）：统一通知中心管理端接口 */
export interface SystemNotificationVo {
  id: string
  userId?: string
  notifyType: string
  channel: string
  title: string
  content: string
  /** 来源模板编码（按模板发送时写入） */
  templateCode?: string
  refType?: string
  refId?: string
  readFlag: number
  readAt?: string
  sendStatus: string
  createTime: string
}

export interface SystemNotificationQuery {
  notifyType?: string
  readFlag?: number
  page?: number
  pageSize?: number
}

/** 接收人候选（仅含选择框展示所需字段） */
export interface RecipientOptionVo {
  id: string
  userName: string
  nickName?: string
}

/**
 * 发送请求：二选一
 * - 传 `templateCode` + `vars` → 按通知模板渲染后发送（模板 channel 决定落库 / 投递邮件）
 * - 否则使用 `title` / `content` 原文
 */
export interface SystemNotificationSendRequest {
  userId?: string
  /** 接收用户 ID 列表（支持多选；与 userId 同时存在时以本字段为准） */
  userIds?: string[]
  notifyType: string
  channel: string
  title?: string
  content?: string
  templateCode?: string
  vars?: Record<string, string>
  refType?: string
  refId?: string
}

export const systemNotificationApi = {
  list: (params?: SystemNotificationQuery) => request.get('/system/notification', { params }),
  unreadCount: () => request.get('/system/notification/unread-count'),
  markRead: (id: string) => request.put(`/system/notification/${id}/read`),
  markAllRead: () => request.put('/system/notification/read-all'),
  /**
   * 接收人候选（供用户选择框检索）。
   * 权限跟随 notification:send —— 能发通知即可选接收人，无需系统用户管理的 user:list。
   */
  recipients: (params?: { keyword?: string; limit?: number }) =>
    request.get<RecipientOptionVo[]>('/system/notification/recipients', { params }),
  /** 定向发送给指定用户（支持多选；userId 为空时仅落库） */
  send: (data: SystemNotificationSendRequest) => request.post('/system/notification', data),
  /** 全站广播（写入全部用户站内通知） */
  broadcast: (data: SystemNotificationSendRequest) => request.post('/system/notification/broadcast', data),
}