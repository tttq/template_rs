import request from './request'

export interface UserVo {
  id: string
  userName: string
  nickName?: string
  email?: string
  phone?: string
  avatar?: string
  status: number
  adminFlag: number
  deptId?: string
  createTime: string
  updateTime: string
  tenantId?: string
  roleIds?: string[]
}

export interface PageResult<T> {
  items: T[]
  total: number
  page: number
  pageSize: number
  totalPages: number
}

export const userApi = {
  list: (params: Record<string, any>) =>
    request.get<PageResult<UserVo>>('/system/users', { params }),
  getById: (id: string) => request.get<UserVo>(`/system/users/${id}`),
  create: (data: any) => request.post<UserVo>('/system/users', data),
  update: (id: string, data: any) => request.put<UserVo>(`/system/users/${id}`, data),
  updateStatus: (id: string, status: number) =>
    request.put(`/system/users/${id}/status`, { status }),
  delete: (id: string) => request.delete(`/system/users/${id}`),
  assignRoles: (id: string, roleIds: string[]) =>
    request.post(`/system/users/${id}/roles`, { roleIds }),
  getRoleIds: (id: string) => request.get<string[]>(`/system/users/${id}/roles`),
}
