import request from './request'

export interface MenuVo {
  id: string
  parentId: string
  clientId: string
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
  createBy?: string
  createId?: string
  updateTime: string
  updateBy?: string
  updateId?: string
  tenantId?: string
  /** 树形禁用（角色权限分组中非所属客户端只读） */
  disabled?: boolean
  children?: MenuVo[]
}

export const menuApi = {
  list: (params: Record<string, any>) => request.get('/system/menus', { params }),
  /** 菜单树；传 clientId 只返回该客户端的菜单（客户端是权限体系顶级维度） */
  listTree: (clientId?: string) =>
    request.get<MenuVo[]>('/system/menus/tree', { params: { clientId } }),
  getById: (id: string) => request.get<MenuVo>(`/system/menus/${id}`),
  create: (data: any) => request.post<MenuVo>('/system/menus', data),
  update: (id: string, data: any) => request.put<MenuVo>(`/system/menus/${id}`, data),
  delete: (id: string) => request.delete(`/system/menus/${id}`),
}
