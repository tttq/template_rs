import request from './request'

export interface TenantVo {
  id: string
  tenantName: string
  tenantCode: string
  mode: string
  databaseType?: string
  databaseUrl?: string
  databaseName?: string
  status: number
  contactName?: string
  contactPhone?: string
  contactEmail?: string
  expireTime?: string
  remark?: string
  createTime: string
  createBy?: string
  createId?: string
  updateTime: string
  updateBy?: string
  updateId?: string
}

export interface CreateTenantFullParams {
  tenantName: string
  tenantCode: string
  databaseType?: string
  databaseUrl?: string
  databaseName?: string
  contactName?: string
  contactPhone?: string
  contactEmail?: string
  remark?: string
  adminUserName: string
  adminPassWord: string
  adminNickName?: string
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
  createFull: (data: CreateTenantFullParams) => request.post<TenantVo>('/system/tenants/create-full', data),
  update: (id: string, data: any) => request.post<TenantVo>(`/system/tenants/${id}`, data),
  delete: (id: string) => request.post(`/system/tenants/${id}/delete`),
  testConnection: (data: { databaseType: string; databaseUrl: string }) =>
    request.post<boolean>('/system/tenants/test-connection', data),
  createDatabase: (data: { databaseType: string; databaseUrl: string; databaseName: string }) =>
    request.post<string>('/system/tenants/create-database', data),
  initDatabase: (data: { databaseType: string; databaseUrl: string; databaseName: string }) =>
    request.post<string>('/system/tenants/init-database', data),
}
