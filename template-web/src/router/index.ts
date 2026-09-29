import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { useUserStore } from '@/stores/user'
import { generateRoutesFromMenus, clearDynamicRoutes } from './dynamic'
import { ensurePermissionStream } from '@/utils/permissionStream'

/**
 * 静态路由
 *
 * 仅包含无需权限的基础页面：
 * - `/login` 登录页
 * - `/403`、`/404` 错误页
 * - `/` 根布局 BasicLayout（dashboard 和 profile 作为静态子路由保留，无需权限）
 *
 * 系统管理菜单（/system/*）不再写死，改为根据后端返回的 menus 数组动态注册。
 * 详见 `setupDynamicRoutes` 与路由守卫中的调用。
 */
const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/login/index.vue'),
    meta: { title: 'route.login' },
  },
  {
    // 完善账户信息（微信自动注册的惰性账号首次进入系统前必填）
    // 用 BlankLayout：未完善时不应看到后台框架
    path: '/profile-setup',
    name: 'ProfileSetup',
    component: () => import('@/layouts/BlankLayout.vue'),
    children: [
      {
        path: '',
        name: 'ProfileSetupPage',
        component: () => import('@/views/profile-setup/index.vue'),
        meta: { title: '完善账户信息' },
      },
    ],
  },
  {
    // Web 微信扫码回调（微信跳转到此页，再交给后端换 token）
    path: '/login/wechat/callback',
    name: 'WechatCallback',
    component: () => import('@/layouts/BlankLayout.vue'),
    children: [
      {
        path: '',
        name: 'WechatCallbackPage',
        component: () => import('@/views/login/wechat-callback.vue'),
        meta: { title: '微信登录' },
      },
    ],
  },
  {
    path: '/',
    name: 'Layout',
    component: () => import('@/layouts/BasicLayout.vue'),
    redirect: '/dashboard',
    children: [
      {
        path: 'dashboard',
        name: 'Dashboard',
        component: () => import('@/views/dashboard/index.vue'),
        meta: { title: 'route.dashboard', icon: 'DashboardOutlined' },
      },
      {
        path: 'profile',
        name: 'Profile',
        component: () => import('@/views/profile/index.vue'),
        meta: { title: 'route.profile' },
      },
      {
        path: 'system/export',
        name: 'SystemExportCenter',
        component: () => import('@/views/system/export/index.vue'),
        meta: { title: '导出中心' },
      },
    ],
  },
  {
    path: '/403',
    name: '403',
    component: () => import('@/views/error/403.vue'),
    meta: { title: '403' },
  },
  {
    path: '/:pathMatch(.*)*',
    name: '404',
    component: () => import('@/views/error/404.vue'),
    meta: { title: '404' },
  },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

/**
 * 标记动态路由是否已注册，避免重复添加
 */
let dynamicRoutesRegistered = false

/**
 * 根据当前用户的 menus 注册动态路由
 *
 * 调用时机：用户登录后首次进入页面、或在路由守卫中检测到 userInfo 已加载但动态路由未注册时。
 * 重复调用安全：会先清理上一次的动态路由再重新注册。
 * 注意：即使 menus 为空也标记已注册——否则守卫中「未注册」分支会对未知路径无限重入。
 */
export function setupDynamicRoutes() {
  const userStore = useUserStore()
  // 先清理旧的动态路由（切换租户或重新登录场景）
  if (dynamicRoutesRegistered) {
    clearDynamicRoutes(router)
    dynamicRoutesRegistered = false
  }
  const menus = userStore.menus
  if (menus.length) {
    for (const route of generateRoutesFromMenus(menus)) {
      // 添加为 Layout 的子路由
      router.addRoute('Layout', route)
    }
  }
  dynamicRoutesRegistered = true
}

/**
 * 清理动态路由并重置标记
 *
 * 用于用户登出时，确保下一次登录时重新根据新用户的菜单生成路由。
 */
export function teardownDynamicRoutes() {
  if (dynamicRoutesRegistered) {
    clearDynamicRoutes(router)
    dynamicRoutesRegistered = false
  }
}

router.beforeEach(async (to, _from, next) => {
  const userStore = useUserStore()

  if (!userStore.token) {
    // 开放页面：登录页 + 微信扫码回调（回调页自行完成登录）
    if (to.path === '/login' || to.path === '/login/wechat/callback') {
      next()
    } else {
      next({ path: '/login', query: { redirect: to.fullPath } })
    }
    return
  }

  if (to.path === '/login') {
    next({ path: '/' })
    return
  }

  // 已登录：建立权限变更通知流（幂等，断线自动重连）
  ensurePermissionStream()

  // 首次进入或刷新页面：拉取用户信息并注册动态路由
  if (!userStore.userInfo) {
    try {
      await userStore.getUserInfo()
      // 根据后端返回的菜单动态注册路由
      setupDynamicRoutes()
      // 重新导航到目标路由，确保动态路由生效。
      // 关键：不能展开整个 `to` —— 刷新时首次解析已命中兜底 404（to.name='404'），
      // 展开会以 name 优先导航到 404；必须仅携带 path/query/hash 触发按路径重新解析。
      next({ path: to.path, query: to.query, hash: to.hash, replace: true })
      return
    } catch {
      try {
        await userStore.logout()
      } catch {
        // ignore logout error, still redirect to login
      }
      next({ path: '/login', query: { redirect: to.fullPath } })
      return
    }
  }

  // 已加载 userInfo 但动态路由未注册（如切换租户后 menus 变化）
  if (!dynamicRoutesRegistered) {
    setupDynamicRoutes()
    next({ path: to.path, query: to.query, hash: to.hash, replace: true })
    return
  }

  // 完善账户信息闸口：微信自动注册的惰性账号（profileCompleted=false）
  // 必须先完善资料才能进入系统；已完善的账号不应停留在完善页
  const profileCompleted = userStore.userInfo.profileCompleted !== false
  if (!profileCompleted && to.path !== '/profile-setup') {
    next({ path: '/profile-setup', replace: true })
    return
  }
  if (profileCompleted && to.path === '/profile-setup') {
    next({ path: '/', replace: true })
    return
  }

  if (to.meta.permission && !userStore.hasPermission(to.meta.permission as string) && !userStore.roles.includes('admin')) {
    next({ path: '/403' })
    return
  }

  next()
})

export default router
