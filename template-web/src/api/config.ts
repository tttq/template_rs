import request from './request'

export interface ConfigVo {
  id: string
  configName: string
  configKey: string
  configValue: string
  configType: string
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

export const configApi = {
  list: (params: Record<string, any>) =>
    request.get<PageResult<ConfigVo>>('/system/configs', { params }),
  getById: (id: string) => request.get<ConfigVo>(`/system/configs/${id}`),
  getByKey: (key: string) => request.get<ConfigVo>(`/system/configs/key/${key}`),
  create: (data: any) => request.post<ConfigVo>('/system/configs', data),
  update: (id: string, data: any) => request.put<ConfigVo>(`/system/configs/${id}`, data),
  delete: (id: string) => request.delete(`/system/configs/${id}`),
}
