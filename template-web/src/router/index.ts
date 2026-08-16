import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { useUserStore } from '@/stores/user'
import { generateRoutesFromMenus, clearDynamicRoutes } from './dynamic'

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
 */
export function setupDynamicRoutes() {
  const userStore = useUserStore()
  // 先清理旧的动态路由（切换租户或重新登录场景）
  if (dynamicRoutesRegistered) {
    clearDynamicRoutes(router)
    dynamicRoutesRegistered = false
  }
  const menus = userStore.menus
  if (!menus.length) return
  const dynamicRoutes = generateRoutesFromMenus(menus)
  for (const route of dynamicRoutes) {
    // 添加为 Layout 的子路由
    router.addRoute('Layout', route)
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
    if (to.path === '/login') {
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

  // 首次进入或刷新页面：拉取用户信息并注册动态路由
  if (!userStore.userInfo) {
    try {
      await userStore.getUserInfo()
      // 根据后端返回的菜单动态注册路由
      setupDynamicRoutes()
      // 重新导航到目标路由，确保动态路由生效
      // 使用 replace 避免在历史记录中留下重定向前的中间状态
      next({ ...to, replace: true })
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
    next({ ...to, replace: true })
    return
  }

  if (to.meta.permission && !userStore.hasPermission(to.meta.permission as string) && !userStore.roles.includes('admin')) {
    next({ path: '/403' })
    return
  }

  next()
})

export default router
