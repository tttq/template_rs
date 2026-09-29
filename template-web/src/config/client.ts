/**
 * 当前前端所属客户端（对应「系统管理 → 客户端管理」里配置的客户端标识/密钥）。
 *
 * 登录时随请求提交，后端据此识别是哪个客户端在登录（OAuth2 的 client_id / client_secret 模式），
 * 只返回该客户端下的菜单与功能权限，并限制没有该客户端角色的账号登录。
 *
 * 生产环境请在 .env.production 覆盖 VITE_CLIENT_CODE / VITE_CLIENT_SECRET。
 */
const env = import.meta.env

export const CLIENT_CODE: string = env.VITE_CLIENT_CODE || 'web-admin'
export const CLIENT_SECRET: string = env.VITE_CLIENT_SECRET || 'web-admin-secret'
