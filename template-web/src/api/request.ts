import axios, { type InternalAxiosRequestConfig, type AxiosResponse } from 'axios'
import JSONbig from 'json-bigint'
import { message } from 'ant-design-vue'
import NProgress from 'nprogress'
import 'nprogress/nprogress.css'

const JSONbigString = JSONbig({ storeAsString: true })

const request = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL || '/api',
  timeout: 30000,
  transformResponse: [
    (data) => {
      if (typeof data === 'string') {
        try {
          return JSONbigString.parse(data)
        } catch {
          return data
        }
      }
      return data
    },
  ],
})

let isRefreshing = false
let pendingRequests: Array<(token: string) => void> = []

function onTokenRefreshed(newToken: string) {
  pendingRequests.forEach((cb) => cb(newToken))
  pendingRequests = []
}

request.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  NProgress.start()
  const token = localStorage.getItem('token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  const tenantId = localStorage.getItem('tenantId')
  if (tenantId) {
    config.headers['x-tenant-id'] = tenantId
  }
  return config
})

request.interceptors.response.use(
  (response: AxiosResponse) => {
    NProgress.done()
    const res = response.data
    if (res.code !== 200) {
      message.error(res.message || '请求失败')
      if (res.code === 401) {
        handleUnauthorized()
      }
      return Promise.reject(new Error(res.message || '请求失败'))
    }
    return res.data as any
  },
  async (error) => {
    NProgress.done()
    if (error.response?.status === 401) {
      const config = error.config as InternalAxiosRequestConfig
      return handleRefreshToken(config)
    }
    // 后端将 401/403 等错误响应体包装为 ApiResponse JSON：
    // { code: 403, message: "无权限访问", data: null }
    // 优先使用后端返回的中文消息，避免显示 axios 默认的 "Request failed with status code xxx"
    const backendMessage = error.response?.data?.message
    const displayMessage = backendMessage || error.message || '网络错误'
    message.error(displayMessage)
    return Promise.reject(new Error(displayMessage))
  },
)

async function handleRefreshToken(config: InternalAxiosRequestConfig): Promise<any> {
  const refreshToken = localStorage.getItem('refreshToken')
  if (!refreshToken) {
    handleUnauthorized()
    return Promise.reject(new Error('无刷新令牌'))
  }

  if (isRefreshing) {
    return new Promise((resolve) => {
      pendingRequests.push((newToken: string) => {
        config.headers.Authorization = `Bearer ${newToken}`
        resolve(request(config))
      })
    })
  }

  isRefreshing = true

  try {
    const res = await axios.post(
      `${request.defaults.baseURL}/auth/refresh-token`,
      { refreshToken },
      { timeout: 10000 },
    )

    if (res.data?.code === 200 && res.data?.data) {
      const data = res.data.data
      const newToken = data.token

      localStorage.setItem('token', newToken)
      if (data.refreshToken) {
        localStorage.setItem('refreshToken', data.refreshToken)
      }
      if (data.expireTime) {
        localStorage.setItem('expireTime', String(data.expireTime))
      }
      if (data.refreshExpireTime) {
        localStorage.setItem('refreshExpireTime', String(data.refreshExpireTime))
      }

      onTokenRefreshed(newToken)

      config.headers.Authorization = `Bearer ${newToken}`
      return request(config)
    } else {
      handleUnauthorized()
      return Promise.reject(new Error('刷新令牌失败'))
    }
  } catch {
    handleUnauthorized()
    return Promise.reject(new Error('刷新令牌失败'))
  } finally {
    isRefreshing = false
  }
}

function handleUnauthorized() {
  localStorage.removeItem('token')
  localStorage.removeItem('refreshToken')
  localStorage.removeItem('expireTime')
  localStorage.removeItem('refreshExpireTime')
  localStorage.removeItem('tenantId')
  const currentPath = window.location.pathname
  if (currentPath !== '/login') {
    window.location.href = `/login?redirect=${encodeURIComponent(currentPath)}`
  }
}

export default request as {
  get<T = any>(url: string, config?: any): Promise<T>
  post<T = any>(url: string, data?: any, config?: any): Promise<T>
  put<T = any>(url: string, data?: any, config?: any): Promise<T>
  delete<T = any>(url: string, config?: any): Promise<T>
}
