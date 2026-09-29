import request from './request'
import type { MenuVo } from './menu'

export interface LoginParams {
  userName?: string
  passWord: string
  email?: string
  loginType?: string
  rememberMe?: boolean
  tenantCode?: string
  /** 客户端标识（OAuth2 的 client_id）：后端据此加载该客户端的菜单与功能权限 */
  clientCode?: string
  /** 客户端密钥：客户端配置了密钥时必填 */
  clientSecret?: string
  code?: string
  state?: string
  /** Web 扫码登录的 CSRF state（由 /auth/wechat/qr-url 下发，一次性消费） */
  qrState?: string
  /** 邮箱验证码登录时必填（loginType = "email_code"） */
  emailCode?: string
  /** 两步验证动态码（TOTP 6 位；账号开启 2FA 时必填） */
  totpCode?: string
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
  /** 邮箱（必填）：注册前必须通过邮箱验证码校验 */
  email: string
  /** 邮箱验证码（必填，一次性消费） */
  emailCode: string
  phone?: string
  tenantCode?: string
  /** 注册身份（必填）：全端共用的 register_role 字典启用项，服务端会再校验 */
  roleCode: string
}

/** 完善账户信息（微信自动注册后的惰性账号首次进入系统前必填） */
export interface CompleteProfileParams {
  userName: string
  passWord: string
  phone: string
  nickName?: string
  /** 身份（必填）：全端共用的 register_role 字典启用项 */
  roleCode: string
  /** 邮箱（必填）：与注册同口径，必须通过邮箱验证码校验 */
  email: string
  /** 邮箱验证码（必填，一次性消费） */
  emailCode: string
}

/** 注册身份选项（免登录公开接口） */
export interface RegisterRoleVo {
  label: string
  value: string
}

/** Web 微信扫码登录：二维码地址 + 一次性 CSRF state */
export interface WechatQrVo {
  url: string
  state: string
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
  /** 账号信息是否已完善；false 时前端强制跳「完善账户信息」页 */
  profileCompleted: boolean
}

/** 第三方绑定（wechat_mp / wechat / github） */
export interface BindParams {
  provider: string
  code?: string
  /** Web 扫码绑定的 CSRF state（provider=wechat 时必填） */
  qrState?: string
}

export interface ProviderBindingVo {
  provider: string
}

/** 图形验证码：captchaId 供发码时回传；image 为 PNG data URI，可直接用于 <img> */
export interface CaptchaResult {
  captchaId: string
  image: string
}

/** 发送邮箱验证码：scene = register / reset / login / change_password / change_email */
export interface SendCodeParams {
  email: string
  scene: string
  captchaId: string
  captchaCode: string
}

export const authApi = {
  login: (params: LoginParams) => request.post<TokenVo>('/auth/login', params),
  locate: (params: LocateTenantParams) => request.post<LocateTenantResult>('/auth/locate', params),
  register: (params: RegisterParams) => request.post<UserInfo>('/auth/register', params),
  getUserInfo: () => request.get<UserInfo>('/auth/user-info'),
  logout: () => request.post('/auth/logout'),
  refreshToken: (params: RefreshTokenParams) => request.post<TokenVo>('/auth/refresh-token', params),
  /** 完善账户信息（微信自动注册后的惰性账号） */
  completeProfile: (params: CompleteProfileParams) =>
    request.post<UserInfo>('/auth/complete-profile', params),
  /** 注册身份选项（管理后台注册弹窗 / 完善资料页共用；免登录） */
  registerRoles: () => request.get<RegisterRoleVo[]>('/auth/register-roles'),
  /** Web 微信扫码登录二维码（免登录） */
  wechatQrUrl: () => request.get<WechatQrVo>('/auth/wechat/qr-url'),
  /** 绑定第三方身份 */
  bind: (params: BindParams) => request.post('/auth/bind', params),
  /** 当前账号已绑定的第三方身份 */
  bindings: () => request.get<ProviderBindingVo[]>('/auth/bindings'),
  /** 解绑第三方身份 */
  unbind: (provider: string) => request.post('/auth/unbind', { provider }),
  /** 图形验证码（免登录）：注册 / 邮箱验证码登录 / 账户安全操作的发码前置校验 */
  captcha: () => request.get<CaptchaResult>('/auth/captcha'),
  /** 发送邮箱验证码（免登录） */
  sendCode: (params: SendCodeParams) => request.post('/auth/send-code', params),
  /** 修改个人信息（昵称 / 手机号） */
  updateProfile: (params: { nickName?: string; phone?: string }) =>
    request.put<UserInfo>('/auth/profile', params),
  /** 修改密码（验证码发到账号当前绑定邮箱，scene=change_password） */
  changePassword: (params: { emailCode: string; newPassword: string }) =>
    request.post('/auth/change-password', params),
  /** 更换绑定邮箱（验证码发到新邮箱，scene=change_email） */
  changeEmail: (params: { newEmail: string; emailCode: string }) =>
    request.post<UserInfo>('/auth/change-email', params),
}
