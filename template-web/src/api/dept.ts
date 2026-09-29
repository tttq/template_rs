import request from './request'

export interface DeptVo {
  id: string
  parentId: string
  deptName: string
  deptSort: number
  status: number
  leader?: string
  phone?: string
  email?: string
  createTime: string
  createBy?: string
  createId?: string
  updateTime: string
  updateBy?: string
  updateId?: string
  tenantId?: string
  children?: DeptVo[]
}

export const deptApi = {
  list: (params: Record<string, any>) => request.get('/system/depts', { params }),
  listTree: () => request.get<DeptVo[]>('/system/depts/tree'),
  getById: (id: string) => request.get<DeptVo>(`/system/depts/${id}`),
  create: (data: any) => request.post<DeptVo>('/system/depts', data),
  update: (id: string, data: any) => request.put<DeptVo>(`/system/depts/${id}`, data),
  delete: (id: string) => request.delete(`/system/depts/${id}`),
}
