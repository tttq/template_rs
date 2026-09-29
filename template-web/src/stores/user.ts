import { defineStore } from 'pinia'
import { ref } from 'vue'
import { authApi } from '@/api/auth'
import type { UserInfo, LoginParams } from '@/api/auth'
import type { MenuVo } from '@/api/menu'
import { disconnectPermissionStream } from '@/utils/permissionStream'
import { CLIENT_CODE, CLIENT_SECRET } from '@/config/client'

const TOKEN_KEY = 'token'
const REFRESH_TOKEN_KEY = 'refreshToken'
const EXPIRE_TIME_KEY = 'expireTime'
const REFRESH_EXPIRE_TIME_KEY = 'refreshExpireTime'
const TENANT_ID_KEY = 'tenantId'
const RENEWAL_THRESHOLD_MS = 5 * 60 * 1000

let renewalTimer: ReturnType<typeof setTimeout> | null = null

export const useUserStore = defineStore('user', () => {
  const token = ref<string>(localStorage.getItem(TOKEN_KEY) || '')
  const refreshToken = ref<string>(localStorage.getItem(REFRESH_TOKEN_KEY) || '')
  const expireTime = ref<number>(Number(localStorage.getItem(EXPIRE_TIME_KEY)) || 0)
  const refreshExpireTime = ref<number>(Number(localStorage.getItem(REFRESH_EXPIRE_TIME_KEY)) || 0)
  const tenantId = ref<string>(localStorage.getItem(TENANT_ID_KEY) || '')
  const userInfo = ref<UserInfo | null>(null)
  const roles = ref<string[]>([])
  const permissions = ref<string[]>([])
  const menus = ref<MenuVo[]>([])

  function setTokenData(data: { token: string; refreshToken?: string; expireTime?: number; refreshExpireTime?: number }) {
    token.value = data.token
    localStorage.setItem(TOKEN_KEY, data.token)

    if (data.refreshToken) {
      refreshToken.value = data.refreshToken
      localStorage.setItem(REFRESH_TOKEN_KEY, data.refreshToken)
    } else {
      refreshToken.value = ''
      localStorage.removeItem(REFRESH_TOKEN_KEY)
    }

    if (data.expireTime) {
      expireTime.value = data.expireTime
      localStorage.setItem(EXPIRE_TIME_KEY, String(data.expireTime))
    } else {
      expireTime.value = 0
      localStorage.removeItem(EXPIRE_TIME_KEY)
    }

    if (data.refreshExpireTime) {
      refreshExpireTime.value = data.refreshExpireTime
      localStorage.setItem(REFRESH_EXPIRE_TIME_KEY, String(data.refreshExpireTime))
    } else {
      refreshExpireTime.value = 0
      localStorage.removeItem(REFRESH_EXPIRE_TIME_KEY)
    }

    scheduleRenewal()
  }

  function clearTokenData() {
    token.value = ''
    refreshToken.value = ''
    expireTime.value = 0
    refreshExpireTime.value = 0
    tenantId.value = ''
    localStorage.removeItem(TOKEN_KEY)
    localStorage.removeItem(REFRESH_TOKEN_KEY)
    localStorage.removeItem(EXPIRE_TIME_KEY)
    localStorage.removeItem(REFRESH_EXPIRE_TIME_KEY)
    localStorage.removeItem(TENANT_ID_KEY)
    clearRenewalTimer()
  }

  function clearRenewalTimer() {
    if (renewalTimer) {
      clearTimeout(renewalTimer)
      renewalTimer = null
    }
  }

  function scheduleRenewal() {
    clearRenewalTimer()

    if (!refreshToken.value || !expireTime.value) return

    const now = Date.now()
    const expireMs = expireTime.value * 1000
    const msUntilExpire = expireMs - now

    if (msUntilExpire <= 0) {
      tryAutoRenewal()
      return
    }

    const msUntilRenewal = msUntilExpire - RENEWAL_THRESHOLD_MS
    if (msUntilRenewal <= 0) {
      tryAutoRenewal()
      return
    }

    renewalTimer = setTimeout(() => {
      tryAutoRenewal()
    }, msUntilRenewal)
  }

  async function tryAutoRenewal(): Promise<boolean> {
    if (!refreshToken.value) return false

    const now = Date.now()
    if (refreshExpireTime.value > 0) {
      const refreshExpireMs = refreshExpireTime.value * 1000
      if (now >= refreshExpireMs) {
        clearTokenData()
        return false
      }
    }

    try {
      const res = await authApi.refreshToken({ refreshToken: refreshToken.value })
      setTokenData({
        token: res.token,
        refreshToken: res.refreshToken || refreshToken.value,
        expireTime: res.expireTime,
        refreshExpireTime: res.refreshExpireTime,
      })
      return true
    } catch {
      clearTokenData()
      return false
    }
  }

  async function login(params: LoginParams) {
    // 客户端身份随登录请求提交：后端按客户端加载菜单/功能权限并做登录限制
    const res = await authApi.login({
      clientCode: CLIENT_CODE,
      clientSecret: CLIENT_SECRET,
      ...params,
    })
    setTokenData({
      token: res.token,
      refreshToken: res.refreshToken,
      expireTime: res.expireTime,
      refreshExpireTime: res.refreshExpireTime,
    })
    await getUserInfo()
  }

  async function getUserInfo() {
    const res = await authApi.getUserInfo()
    userInfo.value = res
    roles.value = res.roles || []
    permissions.value = res.permissions || []
    menus.value = res.menus || []
    if (res.tenantId) {
      tenantId.value = res.tenantId
      localStorage.setItem(TENANT_ID_KEY, res.tenantId)
    }
  }

  async function logout() {
    try {
      await authApi.logout()
    } catch {
      // ignore
    }
    disconnectPermissionStream()
    // 动态导入避免与 router/index.ts 形成循环依赖
    // 清理动态路由，确保下一次登录时根据新用户的菜单重新生成
    const { teardownDynamicRoutes } = await import('@/router')
    teardownDynamicRoutes()
    clearTokenData()
    userInfo.value = null
    roles.value = []
    permissions.value = []
    menus.value = []
  }

  function hasPermission(perm: string) {
    return permissions.value.includes(perm)
  }

  if (token.value && refreshToken.value) {
    scheduleRenewal()
  }

  return {
    token, refreshToken, expireTime, refreshExpireTime, tenantId,
    userInfo, roles, permissions, menus,
    login, getUserInfo, logout, hasPermission,
    tryAutoRenewal, setTokenData,
  }
})
