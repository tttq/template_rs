import type { RouteRecordRaw } from 'vue-router'
import type { MenuVo } from '@/api/menu'

/**
 * 动态路由生成模块
 *
 * 根据后端返回的菜单树（GET /auth/user-info 中的 menus 字段）动态生成 Vue Router 路由记录。
 *
 * 菜单类型与路由生成规则：
 * - `dir`（目录）：仅作为菜单分组容器，不生成路由；递归处理其 children
 * - `menu`（菜单）：生成路由记录，component 字段映射到 views 目录下的 vue 组件
 * - `button`（按钮）：仅用于权限校验（sa_check_permission），不渲染菜单也不生成路由
 *   后端 `get_user_info` 已经过滤掉 button 类型，前端无需特殊处理
 *
 * 组件加载机制：
 * - 使用 Vite 的 import.meta.glob 在构建时收集 views 目录下所有 vue 模块
 * - 运行时根据 menu.component 字段（如 "system/user/index"）查表获取懒加载组件
 * - 若组件未找到，使用 fallback 组件避免白屏
 */

// 预加载所有页面组件（懒加载，构建时按需分块）
const viewModules = import.meta.glob('@/views/**/*.vue')

/**
 * 根据 menu.component 字段（如 "system/user/index"）解析出对应的组件加载函数
 *
 * @param component 后端菜单的 component 字段，如 "system/user/index"
 * @returns 组件懒加载函数，找不到时返回 undefined
 */
function resolveComponent(component?: string | null): (() => Promise<unknown>) | undefined {
  if (!component) return undefined
  // 拼接成完整路径，与 import.meta.glob 的 key 对应
  const fullPath = `/src/views/${component}.vue`
  return viewModules[fullPath] as (() => Promise<unknown>) | undefined
}

/**
 * 将后端菜单的 path（如 "/system/user"）转换为相对路径（如 "system/user"）
 *
 * Vue Router 4 中，子路由的 path 若以 `/` 开头会被视为绝对路径（不拼接父路径），
 * 但仍可作为 children 注册。这里统一去除前导 `/`，让路径相对于父路由（BasicLayout 的 `/`）。
 */
function toRelativePath(path: string): string {
  return path.startsWith('/') ? path.substring(1) : path
}

/**
 * 递归遍历菜单树，生成扁平化的路由记录数组
 *
 * 所有 `menu` 类型的菜单项都会被生成为 BasicLayout 的子路由（扁平结构），
 * `dir` 类型不生成路由但会递归处理其 children。
 *
 * @param menus 后端返回的菜单树
 * @returns 路由记录数组（可直接传给 router.addRoute）
 */
export function generateRoutesFromMenus(menus: MenuVo[]): RouteRecordRaw[] {
  const routes: RouteRecordRaw[] = []

  function walk(items: MenuVo[]) {
    for (const item of items) {
      // button 类型不生成路由
      if (item.menuType === 'button') continue

      // dir 类型：递归处理 children（dir 本身不生成路由）
      if (item.menuType === 'dir') {
        if (item.children?.length) {
          walk(item.children)
        }
        continue
      }

      // menu 类型：生成路由记录
      if (item.menuType === 'menu' && item.path && item.component) {
        const component = resolveComponent(item.component)
        if (!component) {
          console.warn(`[dynamic-router] Component not found for menu "${item.menuName}" (path: ${item.path}, component: ${item.component})`)
          continue
        }
        routes.push({
          path: toRelativePath(item.path),
          // 路由 name 用 menu id 保证唯一，便于后续 removeRoute
          name: `menu_${item.id}`,
          component,
          meta: {
            title: item.menuName,
            permission: item.permission,
            icon: item.icon,
            // 标记为动态路由，便于登出时清理
            dynamic: true,
          },
        })
      }
    }
  }

  walk(menus)
  return routes
}

/**
 * 清理动态路由
 *
 * 遍历 router 中所有 name 以 `menu_` 开头的路由并移除。
 * 用于用户登出或切换租户时清理上一次的动态路由。
 */
export function clearDynamicRoutes(router: { getRoutes: () => Array<{ name?: string | symbol | null }>; removeRoute: (name: string | symbol) => void }) {
  const allRoutes = router.getRoutes()
  for (const route of allRoutes) {
    if (typeof route.name === 'string' && route.name.startsWith('menu_')) {
      router.removeRoute(route.name)
    }
  }
}
