import request from './request'

export interface MenuVo {
  id: string
  parentId: string
  menuName: string
  menuType: string
  path?: string
  component?: string
  icon?: string
  sortOrder: number
  permission?: string
  status: number
  visible: number
  createTime: string
  updateTime: string
  tenantId?: string
  children?: MenuVo[]
}

export const menuApi = {
  list: () => request.get<MenuVo[]>('/system/menus'),
  listTree: () => request.get<MenuVo[]>('/system/menus/tree'),
  getById: (id: string) => request.get<MenuVo>(`/system/menus/${id}`),
  create: (data: any) => request.post<MenuVo>('/system/menus', data),
  update: (id: string, data: any) => request.put<MenuVo>(`/system/menus/${id}`, data),
  delete: (id: string) => request.delete(`/system/menus/${id}`),
}
