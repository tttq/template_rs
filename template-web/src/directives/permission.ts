import type { Directive } from 'vue'
import { useUserStore } from '@/stores/user'

/**
 * 按钮级权限指令：无权限时直接移除元素
 *
 * 用法：
 *   <a-button v-permission="'ai:channel:add'">新增</a-button>
 *   <a v-permission="['ai:model:add', 'ai:model:edit']">任一权限即显示</a>
 *
 * 规则：admin 角色直通；否则需持有列表中任一权限码（与后端 sa_check_permission 对应）。
 * 权限在会话内不变，故 mounted 时一次性判定即可（无需响应式 watch）。
 */
function check(perm: string | string[]): boolean {
  const userStore = useUserStore()
  if (userStore.roles.includes('admin')) return true
  const list = Array.isArray(perm) ? perm : [perm]
  return list.some((p) => userStore.hasPermission(p))
}

export const permission: Directive<HTMLElement, string | string[]> = {
  mounted(el, binding) {
    if (!check(binding.value)) {
      el.parentNode?.removeChild(el)
    }
  },
}
