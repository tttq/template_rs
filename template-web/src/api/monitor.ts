import request from './request'

export const monitorApi = {
  getRedisInfo: () => request.get('/system/redis/info'),
  getServerInfo: () => request.get('/system/server/info'),
}
