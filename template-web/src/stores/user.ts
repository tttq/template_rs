import { defineStore } from 'pinia'
import { ref } from 'vue'
import { authApi } from '@/api/auth'
import type { UserInfo, LoginParams } from '@/api/auth'
import type { MenuVo } from '@/api/menu'

export const useUserStore = defineStore('user', () => {
  const token = ref<string>(localStorage.getItem('token') || '')
  const userInfo = ref<UserInfo | null>(null)
  const roles = ref<string[]>([])
  const permissions = ref<string[]>([])
  const menus = ref<MenuVo[]>([])

  async function login(params: LoginParams) {
    const res = await authApi.login(params)
    token.value = res.token
    localStorage.setItem('token', res.token)
    await getUserInfo()
  }

  async function getUserInfo() {
    const res = await authApi.getUserInfo()
    userInfo.value = res
    roles.value = res.roles || []
    permissions.value = res.permissions || []
    menus.value = res.menus || []
  }

  async function logout() {
    try {
      await authApi.logout()
    } catch {
      // ignore
    }
    token.value = ''
    userInfo.value = null
    roles.value = []
    permissions.value = []
    menus.value = []
    localStorage.removeItem('token')
  }

  function hasPermission(perm: string) {
    return permissions.value.includes(perm)
  }

  return { token, userInfo, roles, permissions, menus, login, getUserInfo, logout, hasPermission }
})
