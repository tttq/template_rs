import request from './request'

export interface GeneratorConfig {
  tableName: string
  moduleName: string
  businessName: string
  entityName: string
  generateFrontend: boolean
  generateBackend: boolean
  parentMenuId?: number
}

export const generatorApi = {
  listTables: () => request.get('/system/tables'),
  getTableColumns: (tableName: string) => request.get(`/system/tables/${tableName}/columns`),
  preview: (config: GeneratorConfig) => request.post('/system/preview', config),
  download: (config: GeneratorConfig) => request.post('/system/download', config, { responseType: 'blob' }),
}
