/**
 * a-tabs 切换时的焦点兜底。
 *
 * 背景：antd Tabs 会把非激活面板设为 `aria-hidden="true"` + `display:none`，而 rc-tabs 在切页时
 * 会把焦点落到新激活的面板上（`tabindex="-1"`）。若被隐藏的面板（或面板内的输入框）仍持有焦点，
 * Chrome 会告警：
 *   Blocked aria-hidden on an element because its descendant retained focus.
 * 切页前先把当前焦点元素 blur 掉，即可避免「隐藏元素仍持有焦点」。
 *
 * 用法（二选一）：
 * - 无额外逻辑：`<a-tabs v-model:active-key="tab" @change="blurActiveFocus">`
 * - 已有 change 处理：在函数开头调用 `blurActiveFocus()`，或 `const onChange = blurOnTabChange(fn)`
 */

/** 让当前获得焦点的元素失焦（body / 无 blur 的元素跳过） */
export function blurActiveFocus(): void {
  const el = document.activeElement as HTMLElement | null
  if (el && el !== document.body && typeof el.blur === 'function') {
    el.blur()
  }
}

/** 包装 tabs 的 change 处理器：先失焦，再执行原逻辑 */
export function blurOnTabChange<T extends string | number>(
  handler?: (key: T) => void,
): (key: T) => void {
  return (key: T) => {
    blurActiveFocus()
    handler?.(key)
  }
}
