/**
 * 权限变更实时通知（SSE）
 *
 * 订阅后端 GET /api/auth/notify/stream。角色重新分配权限等操作后，
 * 后端通过该流推送 perm-changed 事件，前端收到后重拉用户信息并重建动态路由，
 * 使在线用户的菜单即时更新。
 *
 * 说明：
 * - EventSource 无法自定义请求头，token 以查询参数传递（后端 sa-token 支持）；
 * - 断开后不用浏览器自动重连（URL 里 token 会过期），改为手动延迟重连，
 *   重连时读取 localStorage 中最新 token；
 * - 与 router 形成潜在循环依赖，故 store/router 均在事件回调内动态导入。
 */
import { message } from 'ant-design-vue'

const RECONNECT_DELAY_MS = 5000

let source: EventSource | null = null
let reconnectTimer: ReturnType<typeof setTimeout> | null = null

function buildUrl(): string | null {
  const token = localStorage.getItem('token')
  if (!token) return null
  const base = '/api'
  return `${base}/auth/notify/stream?Authorization=${encodeURIComponent(token)}`
}

async function handlePermChanged() {
  try {
    const [userModule, routerModule] = await Promise.all([
      import('@/stores/user'),
      import('@/router'),
    ])
    const userStore = userModule.useUserStore()
    // 重拉最新菜单/权限并重建动态路由
    await userStore.getUserInfo()
    routerModule.setupDynamicRoutes()

    // 当前页面已被移除（不再有权限访问）时回退到工作台
    const router = routerModule.default
    const resolved = router.resolve(router.currentRoute.value.fullPath)
    if (resolved.matched.length === 0 || resolved.name === '404') {
      await router.replace('/dashboard')
    }
    message.info('权限已更新，菜单已刷新')
  } catch {
    // 刷新失败不打断连接；下次变更或重新进入页面时会再触发
  }
}

function close() {
  if (source) {
    source.close()
    source = null
  }
}

function connect() {
  if (source || reconnectTimer) return

  const url = buildUrl()
  if (!url) return

  const es = new EventSource(url)
  source = es

  es.addEventListener('perm-changed', () => {
    void handlePermChanged()
  })

  es.onerror = () => {
    // 关闭后用最新 token 手动重连，避免浏览器用旧 URL 无限重试
    close()
    if (!localStorage.getItem('token')) return
    reconnectTimer = setTimeout(() => {
      reconnectTimer = null
      connect()
    }, RECONNECT_DELAY_MS)
  }
}

/** 确保通知流已连接（幂等，可在每次路由导航时调用） */
export function ensurePermissionStream() {
  connect()
}

/** 断开通知流并取消重连（登出时调用） */
export function disconnectPermissionStream() {
  close()
  if (reconnectTimer) {
    clearTimeout(reconnectTimer)
    reconnectTimer = null
  }
}
