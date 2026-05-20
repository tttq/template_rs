import axios, { type InternalAxiosRequestConfig } from 'axios'
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

request.interceptors.request.use((config: InternalAxiosRequestConfig) => {
  NProgress.start()
  const token = localStorage.getItem('token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }
  return config
})

request.interceptors.response.use(
  (response) => {
    NProgress.done()
    const res = response.data
    if (res.code !== 200) {
      message.error(res.message || '请求失败')
      if (res.code === 401) {
        localStorage.removeItem('token')
        window.location.href = '/login'
      }
      return Promise.reject(new Error(res.message || '请求失败'))
    }
    return res.data as any
  },
  (error) => {
    NProgress.done()
    if (error.response?.status === 401) {
      localStorage.removeItem('token')
      window.location.href = '/login'
      return Promise.reject(error)
    }
    message.error(error.message || '网络错误')
    return Promise.reject(error)
  },
)

export default request as {
  get<T = any>(url: string, config?: any): Promise<T>
  post<T = any>(url: string, data?: any, config?: any): Promise<T>
  put<T = any>(url: string, data?: any, config?: any): Promise<T>
  delete<T = any>(url: string, config?: any): Promise<T>
}
