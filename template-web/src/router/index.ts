import { createRouter, createWebHistory, type RouteRecordRaw } from 'vue-router'
import { useUserStore } from '@/stores/user'

const routes: RouteRecordRaw[] = [
  {
    path: '/login',
    name: 'Login',
    component: () => import('@/views/login/index.vue'),
    meta: { title: 'route.login' },
  },
  {
    path: '/',
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
        path: 'system/user',
        name: 'User',
        component: () => import('@/views/system/user/index.vue'),
        meta: { title: 'route.userManage', permission: 'user:list' },
      },
      {
        path: 'system/role',
        name: 'Role',
        component: () => import('@/views/system/role/index.vue'),
        meta: { title: 'route.roleManage', permission: 'role:list' },
      },
      {
        path: 'system/menu',
        name: 'Menu',
        component: () => import('@/views/system/menu/index.vue'),
        meta: { title: 'route.menuManage', permission: 'menu:list' },
      },
      {
        path: 'system/dept',
        name: 'Dept',
        component: () => import('@/views/system/dept/index.vue'),
        meta: { title: 'route.deptManage', permission: 'dept:list' },
      },
      {
        path: 'system/tenant',
        name: 'Tenant',
        component: () => import('@/views/system/tenant/index.vue'),
        meta: { title: 'route.tenantManage', permission: 'tenant:list' },
      },
      {
        path: 'system/dict',
        name: 'Dict',
        component: () => import('@/views/system/dict/index.vue'),
        meta: { title: 'route.dictManage', permission: 'dict:list' },
      },
      {
        path: 'profile',
        name: 'Profile',
        component: () => import('@/views/profile/index.vue'),
        meta: { title: 'route.profile' },
      },
      {
        path: 'system/config',
        name: 'Config',
        component: () => import('@/views/system/config/index.vue'),
        meta: { title: 'route.configManage', permission: 'config:list' },
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

  if (!userStore.userInfo) {
    try {
      await userStore.getUserInfo()
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

  if (to.meta.permission && !userStore.hasPermission(to.meta.permission as string) && !userStore.roles.includes('admin')) {
    next({ path: '/403' })
    return
  }

  next()
})

export default router
