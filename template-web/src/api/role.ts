import request from './request'

export interface RoleVo {
  id: string
  parentId: string
  roleName: string
  roleCode: string
  roleSort: number
  status: number
  remark?: string
  createTime: string
  updateTime: string
  tenantId?: string
  menuIds?: string[]
  children?: RoleVo[]
}

export const roleApi = {
  list: (params: Record<string, any>) =>
    request.get('/system/roles', { params }),
  getById: (id: string) => request.get<RoleVo>(`/system/roles/${id}`),
  create: (data: any) => request.post<RoleVo>('/system/roles', data),
  update: (id: string, data: any) => request.put<RoleVo>(`/system/roles/${id}`, data),
  delete: (id: string) => request.delete(`/system/roles/${id}`),
  listTree: () => request.get<RoleVo[]>('/system/roles/tree'),
  listAll: () => request.get<RoleVo[]>('/system/roles/all'),
  assignMenus: (id: string, menuIds: string[]) =>
    request.post(`/system/roles/${id}/menus`, { menuIds }),
  getMenuIds: (id: string) => request.get<string[]>(`/system/roles/${id}/menus`),
}
