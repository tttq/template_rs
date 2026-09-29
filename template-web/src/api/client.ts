import request from './request'

export interface ClientVo {
  id: string
  clientCode: string
  clientSecret: string
  clientName: string
  clientType: string
  logo?: string
  homePath?: string
  sortOrder: number
  status: number
  remark?: string
  createTime: string
  createBy?: string
  createId?: string
  updateTime: string
  updateBy?: string
  updateId?: string
}

/** 下拉项（不含密钥），菜单/角色按客户端筛选时使用 */
export interface ClientOption {
  id: string
  clientCode: string
  clientName: string
  clientType: string
  logo?: string
  homePath?: string
}

export interface PageResult<T> {
  items: T[]
  total: number
  page: number
  pageSize: number
  totalPages: number
}

export const clientApi = {
  list: (params: Record<string, any>) =>
    request.get<PageResult<ClientVo>>('/system/clients', { params }),
  options: () => request.get<ClientOption[]>('/system/clients/options'),
  getById: (id: string) => request.get<ClientVo>(`/system/clients/${id}`),
  create: (data: any) => request.post<ClientVo>('/system/clients', data),
  update: (id: string, data: any) => request.put<ClientVo>(`/system/clients/${id}`, data),
  delete: (id: string) => request.delete(`/system/clients/${id}`),
}
