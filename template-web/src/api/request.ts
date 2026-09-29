import axios, { type InternalAxiosRequestConfig, type AxiosResponse } from 'axios'
import JSONbig from 'json-bigint'
import { message } from 'ant-design-vue'
import NProgress from 'nprogress'
import 'nprogress/nprogress.css'
import i18n from '@/locales'

const JSONbigString = JSONbig({ storeAsString: true })

/** 取当前语言的兜底文案（未定义 key 时回退中文键） */
const tk = (key: string) => i18n.global.t(key)

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

/**
 * 开放认证端点（免登录）：登录 / 注册 / 图形验证码 / 邮箱验证码 / 微信扫码地址。
 * 这类请求不应携带残留的旧 token（过期 token 会触发 401 导致被踢回登录页），
 * 且 401 时不能走刷新/跳转逻辑（见 response 拦截器）。
 */
const OPEN_AUTH_PATHS = [
  '/auth/login',
  '/auth/captcha',
  '/auth/register',
  '/auth/register-roles',
  '/auth/send-code',
  '/auth/wechat/qr-url',
  '/auth/locate',
  '/auth/providers',
  '/auth/refresh-token',
]

function isOpenAuthPath(url?: string) {
  return !!url && OPEN_AUTH_PATHS.some((p) => url.includes(p))
}

function onTokenRefreshed(newToken: string) {
  pendingRequests.forEach((cb) => cb(newToken))
  pendingRequests = []
}

request.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  NProgress.start()
  // 后端按 Accept-Language 返回对应语言的消息（错误提示与操作结果）
  config.headers['Accept-Language'] = localStorage.getItem('locale') || 'zh-CN'
  if (isOpenAuthPath(config.url)) {
    return config
  }
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
      message.error(res.message || tk('common.requestFailed'))
      if (res.code === 401) {
        handleUnauthorized()
      }
      return Promise.reject(new Error(res.message || tk('common.requestFailed')))
    }
    return res.data as any
  },
  async (error) => {
    NProgress.done()
    if (error.response?.status === 401) {
      const config = error.config as InternalAxiosRequestConfig
      // 开放认证端点（登录 / 注册 / 验证码）401 属异常（正常应放行），
      // 不触发 token 刷新 / 跳转登录页，直接带业务码拒绝由页面处理
      if (isOpenAuthPath(config.url)) {
        const backendMessage = error.response?.data?.message
        const err = new Error(backendMessage || tk('common.loginExpired')) as Error & { code?: number }
        err.code = 401
        return Promise.reject(err)
      }
      return handleRefreshToken(config)
    }
    // 后端将 401/403 等错误响应体包装为 ApiResponse JSON：
    // { code: 403, message: "无权限访问", data: null }
    // 优先使用后端返回的消息（已按请求语言返回），避免显示 axios 默认的 "Request failed with status code xxx"
    const backendCode = error.response?.data?.code
    const backendMessage = error.response?.data?.message
    // 非 JSON 响应（如网关/框架层 413 纯文本、nginx 413 HTML、502 空响应）拿不到 message，
    // 这里补一层状态码兜底文案，否则用户只会看到 "Request failed with status code 413" 这类提示
    const status = error.response?.status
    const fallbackByStatus: Record<number, string> = {
      413: tk('common.uploadTooLarge'),
      502: tk('common.serviceUnavailable'),
      504: tk('common.requestTimeout'),
    }
    const displayMessage = backendMessage || fallbackByStatus[status] || error.message || tk('common.networkError')
    // 428（登录两步验证）由登录页自行提示并切换输入步骤，此处不重复弹错
    if (backendCode !== 428) {
      message.error(displayMessage)
    }
    // 附带后端业务码（如登录两步验证 428），供调用方分支处理
    const err = new Error(displayMessage) as Error & { code?: number }
    err.code = backendCode
    return Promise.reject(err)
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
  delete<T = any>(url: string, data?: any, config?: any): Promise<T>
}