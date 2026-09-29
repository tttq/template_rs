import request from './request'

export interface DictTypeVo {
  id: string
  dictName: string
  dictType: string
  status: number
  remark?: string
  createTime: string
  createBy?: string
  createId?: string
  updateTime: string
  updateBy?: string
  updateId?: string
}

export interface DictItemVo {
  id: string
  dictTypeId: string
  dictLabel: string
  dictValue: string
  sortOrder: number
  status: number
  remark?: string
  cssClass?: string
  listClass?: string
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

export const dictTypeApi = {
  list: (params: Record<string, any>) =>
    request.get<PageResult<DictTypeVo>>('/system/dicts/types', { params }),
  getById: (id: string) => request.get<DictTypeVo>(`/system/dicts/types/${id}`),
  create: (data: any) => request.post('/system/dicts/types', data),
  update: (id: string, data: any) => request.put(`/system/dicts/types/${id}`, data),
  delete: (id: string) => request.delete(`/system/dicts/types/${id}`),
}

/** 字典项选项（仅 label/value，供业务页面下拉使用） */
export interface DictOptionVo {
  label: string
  value: string
}

export const dictItemApi = {
  getByTypeId: (typeId: string) =>
    request.get<DictItemVo[]>(`/system/dicts/items/by-type/${typeId}`),
  getByTypeCode: (typeCode: string) =>
    request.get<DictItemVo[]>(`/system/dicts/types/${typeCode}/items`),
  /**
   * 按字典类型编码取**启用中**的字典项选项（仅 label/value）。
   *
   * 走仅需登录的 `/system/dicts/options/{typeCode}`：
   * 业务页面（如采购的工厂类型）无需 `dict:list` 权限即可渲染下拉。
   */
  getOptionsByTypeCode: (typeCode: string) =>
    request.get<DictOptionVo[]>(`/system/dicts/options/${typeCode}`),
  getById: (id: string) => request.get<DictItemVo>(`/system/dicts/items/${id}`),
  create: (data: any) => request.post('/system/dicts/items', data),
  update: (id: string, data: any) => request.put(`/system/dicts/items/${id}`, data),
  delete: (id: string) => request.delete(`/system/dicts/items/${id}`),
}
