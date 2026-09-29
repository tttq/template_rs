import request from './request'

export interface RoleVo {
  id: string
  parentId: string
  clientId: string
  roleName: string
  roleCode: string
  roleSort: number
  status: number
  remark?: string
  createTime: string
  createBy?: string
  createId?: string
  updateTime: string
  updateBy?: string
  updateId?: string
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
  listTree: (clientId?: string) =>
    request.get<RoleVo[]>('/system/roles/tree', { params: { clientId } }),
  listAll: (clientId?: string) =>
    request.get<RoleVo[]>('/system/roles/all', { params: { clientId } }),
  assignMenus: (id: string, menuIds: string[]) =>
    request.post(`/system/roles/${id}/menus`, { menuIds }),
  getMenuIds: (id: string) => request.get<string[]>(`/system/roles/${id}/menus`),
}
