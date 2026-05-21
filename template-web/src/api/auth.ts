import request from './request'
import type { MenuVo } from './menu'

export interface LoginParams {
  userName?: string
  passWord: string
  email?: string
  loginType?: string
  rememberMe?: boolean
  tenantCode?: string
  code?: string
  state?: string
}

export interface LocateTenantParams {
  userName: string
}

export interface LocateTenantResult {
  tenantName: string
  tenantCode: string
  tenantLogo?: string
  userName: string
  loginType?: string
}

export interface RegisterParams {
  userName: string
  passWord: string
  nickName?: string
  email?: string
  phone?: string
  registerType?: string
  tenantCode?: string
}

export interface TokenVo {
  token: string
  tokenName: string
  tokenPrefix: string
  refreshToken?: string
  expireTime?: number
  refreshExpireTime?: number
}

export interface RefreshTokenParams {
  refreshToken: string
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
  tenantId?: string
  tenantCode?: string
  tenantName?: string
}

export const authApi = {
  login: (params: LoginParams) => request.post<TokenVo>('/auth/login', params),
  locate: (params: LocateTenantParams) => request.post<LocateTenantResult>('/auth/locate', params),
  register: (params: RegisterParams) => request.post<UserInfo>('/auth/register', params),
  getUserInfo: () => request.get<UserInfo>('/auth/user-info'),
  logout: () => request.post('/auth/logout'),
  refreshToken: (params: RefreshTokenParams) => request.post<TokenVo>('/auth/refresh-token', params),
}