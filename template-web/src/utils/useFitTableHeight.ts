import { nextTick, onBeforeUnmount, onMounted, ref } from 'vue'

/**
 * 表格「仅内容滚动」自适应工具（供手写 <a-table> 页面复用）
 *
 * 用法：
 *   1) 给「表格所在 a-card」绑定 ref：`<a-card ref="fitRef"> ... </a-card>`
 *   2) a-table 绑定：`:scroll="{ x: 1100, y: fitY }"`
 *   3) script 中：`const { fitRef, fitY } = useFitTableHeight()`
 *
 * 原理：
 *   - 计算表格卡片相对视口底部剩余高度，写死卡片高度（数据再多也不撑高外层 → 整页无滚动条）；
 *   - 表格 body 可滚动高 = 卡片高 − 卡片头 − body padding − 表头 − 分页，
 *     写入 a-table 的 scroll.y，使仅数据行内部滚动（表头在上、分页在底固定）。
 *   - 内容不足一屏时自动恢复自然高度，避免大段留白。
 *   - 数据刷新后自动重算（配合 nextTick + 延时校准）。
 */
export function useFitTableHeight() {
  /** 表格所在卡片（应为 <a-card> 根节点） */
  const fitRef = ref<HTMLElement | null>(null)
  /**
   * 表格 body 滚动高度。
   * 语义：`undefined` = 不启用表格内部滚动（antd 按内容自然撑高）；
   * 启用时为一个正数像素值。
   * ⚠ 注意：切勿用 `0` 表示「不启用」—— antd 会把 `scroll.y = 0` 当作真实高度
   * 0，导致 `.ant-table-body` 高度塌缩，所有数据行被裁切不可见。
   */
  const fitY = ref<number | undefined>(undefined)

  let fitTimer = 0
  let resizeTimer = 0
  let observer: ResizeObserver | undefined

  /** 恢复卡片自然高度并关闭表格内部滚动 */
  function resetCard(card: HTMLElement) {
    card.style.height = ''
    card.style.overflow = ''
    fitY.value = undefined
  }

  /**
   * 规范化测量目标：`ref="fitRef"` 绑在 <a-card> 组件上时，Vue 3 拿到的是
   * 组件实例而非 DOM 元素，需取其根元素 `$el`；绑在原生元素上则直接返回。
   */
  function toElement(target: unknown): HTMLElement | null {
    if (!target) return null
    if (target instanceof HTMLElement) return target
    if (typeof target === 'object' && '$el' in target) {
      const el = (target as { $el?: unknown }).$el
      if (el instanceof HTMLElement) return el
    }
    return null
  }

  function recompute() {
    requestAnimationFrame(() => {
      nextTick(() => {
        const card = toElement(fitRef.value)
        if (!card || !card.isConnected) return
        const viewportH = window.innerHeight
        const top = card.getBoundingClientRect().top
        // 卡片可达最大高度 = 视口 − 卡片顶部 − 底部留白（含页面 margin + 滚动条安全量）
        const usable = viewportH - top - 32
        // 视口剩余空间不足时不干预，同时复位上一次可能遗留的适配样式
        if (usable < 220) {
          resetCard(card)
          return
        }

        const head = card.querySelector<HTMLElement>('.ant-card-head')
        const headH = head ? head.offsetHeight : 0
        const cardTop = card.getBoundingClientRect().top
        const thead = card.querySelector<HTMLElement>('.ant-table-thead')
        const theadH = thead ? thead.offsetHeight : 0
        // 表头相对卡片顶部的偏移 = 卡片头部 + body padding + 搜索区等前置内容高度，
        // 直接测量天然精确（存在搜索卡片等场景也正确），避免手工累加遗漏
        const topOffset = thead ? thead.getBoundingClientRect().top - cardTop : headH + 48
        const pag = card.querySelector<HTMLElement>('.ant-table-pagination')
        const pagH = pag ? pag.offsetHeight + 16 : 16
        const cap = usable - topOffset - theadH - pagH

        const tbody = card.querySelector<HTMLElement>('.ant-table-tbody')
        const contentH = tbody ? tbody.scrollHeight : 0
        const needFit = contentH > cap && cap >= 60

        if (needFit) {
          card.style.height = `${Math.floor(usable)}px`
          card.style.overflow = 'hidden'
          fitY.value = Math.floor(cap)
        } else {
          resetCard(card)
        }
      })
    })
  }

  function onResize() {
    window.clearTimeout(resizeTimer)
    resizeTimer = window.setTimeout(recompute, 150)
  }

  function setupObserver() {
    const card = toElement(fitRef.value)
    if (!card || typeof ResizeObserver === 'undefined') return
    observer = new ResizeObserver(() => {
      window.clearTimeout(fitTimer)
      fitTimer = window.setTimeout(recompute, 120)
    })
    observer.observe(card)
  }

  onMounted(() => {
    window.addEventListener('resize', onResize)
    nextTick(() => {
      setupObserver()
      recompute()
    })
  })

  onBeforeUnmount(() => {
    window.removeEventListener('resize', onResize)
    window.clearTimeout(fitTimer)
    window.clearTimeout(resizeTimer)
    observer?.disconnect()
    observer = undefined
  })

  return { fitRef, fitY, recompute }
}
