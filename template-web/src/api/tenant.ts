import request from './request'

export interface TenantVo {
  id: string
  tenantName: string
  tenantCode: string
  status: number
  contactName?: string
  contactPhone?: string
  contactEmail?: string
  expireTime?: string
  remark?: string
  createTime: string
  updateTime: string
}

export interface PageResult<T> {
  items: T[]
  total: number
  page: number
  pageSize: number
  totalPages: number
}

export const tenantApi = {
  list: (params: Record<string, any>) =>
    request.get<PageResult<TenantVo>>('/system/tenants', { params }),
  getById: (id: string) => request.get<TenantVo>(`/system/tenants/${id}`),
  create: (data: any) => request.post<TenantVo>('/system/tenants', data),
  update: (id: string, data: any) => request.put<TenantVo>(`/system/tenants/${id}`, data),
  delete: (id: string) => request.delete(`/system/tenants/${id}`),
}
