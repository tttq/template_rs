import request from './request'
import type { MenuVo } from './menu'

export interface LoginParams {
  userName?: string
  passWord: string
  email?: string
  loginType?: string
}

export interface RegisterParams {
  userName: string
  passWord: string
  nickName?: string
  email?: string
  phone?: string
  registerType?: string
}

export interface TokenVo {
  token: string
  tokenName: string
  tokenPrefix: string
}

export interface UserInfo {
  id: string
  userName: string
  nickName?: string
  email?: string
  phone?: string
  avatar?: string
  roles: string[]
  permissions: string[]
  menus: MenuVo[]
}

export const authApi = {
  login: (params: LoginParams) => request.post<TokenVo>('/auth/login', params),
  register: (params: RegisterParams) => request.post<UserInfo>('/auth/register', params),
  getUserInfo: () => request.get<UserInfo>('/auth/user-info'),
  logout: () => request.post('/auth/logout'),
}
