<template>
  <div
    ref="wrapRef"
    class="pro-table-wrap"
  >
    <a-card
      v-if="searchFields.length > 0"
      :bordered="false"
      class="mb-16"
    >
      <a-row :gutter="16">
        <template
          v-for="item in searchFields"
          :key="item.field"
        >
          <a-col
            :span="6"
            :xs="24"
            :sm="12"
            :md="8"
            :lg="6"
          >
            <div class="search-item">
              <span class="search-label">{{ item.label }}:</span>
              <a-input
                v-if="item.type === 'input'"
                :value="searchParams[item.field]"
                :placeholder="item.placeholder || t('common.placeholderInput')"
                allow-clear
                @update:value="onSearchUpdate(item.field, $event)"
                @press-enter="fetchData"
              />
              <a-select
                v-else-if="item.type === 'select'"
                :value="searchParams[item.field]"
                :placeholder="item.placeholder || t('common.placeholderSelect')"
                :options="item.options"
                allow-clear
                @update:value="onSearchUpdate(item.field, $event)"
              />
              <UserPicker
                v-else-if="item.type === 'userPicker'"
                :value="searchParams[item.field]"
                :display-field="item.displayField || 'id'"
                clearable
                :placeholder="item.placeholder || t('common.placeholderUser')"
                @update:value="onSearchUpdate(item.field, $event)"
              />
              <a-range-picker
                v-else-if="item.type === 'timeRange'"
                :value="searchParams[item.field]"
                :placeholder="item.placeholder || [t('common.placeholderStart'), t('common.placeholderEnd')]"
                :disabled="item.disabled"
                show-time
                style="width: 100%"
                v-bind="item.props"
                @update:value="onSearchUpdate(item.field, $event)"
              />
              <DictSelect
                v-else-if="item.type === 'dictSelect'"
                :value="searchParams[item.field]"
                :dict-code="item.dictCode!"
                :placeholder="item.placeholder || t('common.placeholderSelect')"
                allow-clear
                @update:value="onSearchUpdate(item.field, $event)"
              />
            </div>
          </a-col>
        </template>
        <a-col
          :span="6"
          :xs="24"
          :sm="12"
          :md="8"
          :lg="6"
        >
          <a-space>
            <a-button
              type="primary"
              @click="fetchData"
            >
              {{ t('common.search') }}
            </a-button>
            <a-button @click="resetSearch">
              {{ t('common.reset') }}
            </a-button>
          </a-space>
        </a-col>
      </a-row>
    </a-card>

    <a-card :bordered="false">
      <template #extra>
        <slot name="toolbar" />
      </template>
      <a-table
        :columns="finalColumns"
        :data-source="tableData"
        :loading="loading"
        :pagination="paginationConfig"
        :scroll="tableScroll"
        :row-key="rowKey"
        :row-selection="rowSelection"
        size="middle"
        @change="onTableChange"
      >
        <template #bodyCell="{ column, record, index }">
          <template v-if="column.key === '_index'">
            {{ serialNo(index) }}
          </template>
          <slot
            v-else
            name="bodyCell"
            :column="column"
            :record="record"
            :index="index"
          />
        </template>
      </a-table>
    </a-card>
  </div>
</template>

<script setup lang="ts">
import { nextTick, onBeforeUnmount, onMounted, reactive, ref, computed, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import type { FormField, TableColumn, PageResult } from './types'
import DictSelect from '../DictSelect.vue'
import UserPicker from '../UserPicker.vue'

const props = withDefaults(defineProps<{
  columns: TableColumn[]
  searchFields?: FormField[]
  apiFn: (params: Record<string, any>) => Promise<PageResult<any>>
  rowKey?: string
  scroll?: Record<string, number | string>
  rowSelection?: any
  pageSize?: number
  /** 首列序号（跨页连续编号），默认开启 */
  showIndex?: boolean
  immediate?: boolean
  /** 关闭自适应高度：表格不内部滚动，交由页面/外层滚动 */
  noFit?: boolean
  /** 搜索条件初始值（如 { status: 'pool' }）；变化时自动重置条件并重新查询 */
  searchDefaults?: Record<string, any>
}>(), {
  searchFields: () => [],
  searchDefaults: () => ({}),
  rowKey: 'id',
  scroll: () => ({ x: 1100 }),
  pageSize: 20,
  showIndex: true,
  immediate: true,
  noFit: false,
})

const emit = defineEmits<{
  'loaded': [data: any[], total: number]
}>()

const { t } = useI18n()

const loading = ref(false)
const tableData = ref<any[]>([])
const searchParams = ref<Record<string, any>>({ ...props.searchDefaults })
const wrapRef = ref<HTMLElement>()
/** 表格数据区高度（内容超高时内部滚动；0=不限制） */
const fitScrollY = ref<number>(0)

const pagination = reactive({
  current: 1,
  pageSize: props.pageSize,
  total: 0,
})

const paginationConfig = computed(() => ({
  current: pagination.current,
  pageSize: pagination.pageSize,
  total: pagination.total,
  showSizeChanger: true,
  showQuickJumper: true,
  pageSizeOptions: [ '20','40', '60', '100'],
  showTotal: (total: number) => t('common.totalItems', { total }),
}))

/** 合并外部 scroll 与自适应高度（外部显式 y 优先） */
const tableScroll = computed(() => ({
  x: props.scroll.x,
  y: props.scroll.y ?? (props.noFit ? undefined : fitScrollY.value || undefined),
}))

/** 首列序号（跨页连续：第 N 页第 i 条 = (N-1)×pageSize + i + 1） */
const finalColumns = computed<TableColumn[]>(() => {
  if (!props.showIndex) return props.columns
  return [{ title: t('common.index'), key: '_index', width: 64, fixed: 'left' }, ...props.columns]
})

function serialNo(index: number): number {
  return (pagination.current - 1) * pagination.pageSize + index + 1
}

function onSearchUpdate(field: string, value: any) {
  searchParams.value = { ...searchParams.value, [field]: value }
}

async function fetchData() {
  loading.value = true
  try {
    const params: Record<string, any> = {
      page: pagination.current,
      pageSize: pagination.pageSize,
    }
    for (const [k, v] of Object.entries(searchParams.value)) {
      if (v !== undefined && v !== null && v !== '') {
        // timeRange 控件返回数组 [start, end]，拆分为 createTimeStart / createTimeEnd
        if (Array.isArray(v)) {
          if (v.length === 2) {
            params[k + 'Start'] = v[0]
            params[k + 'End'] = v[1]
          }
        } else {
          params[k] = v
        }
      }
    }
    const res = await props.apiFn(params)
    tableData.value = res.items
    pagination.total = res.total
    emit('loaded', res.items, res.total)
  } finally {
    loading.value = false
    await nextTick()
    recomputeFit()
    scheduleFit()
  }
}

function resetSearch() {
  // 重置回页面配置的默认条件（如品类池页默认只看「品类池」状态）
  searchParams.value = { ...props.searchDefaults }
  pagination.current = 1
  fetchData()
}

/**
 * 外部默认条件变化（如顶部状态页签切换）→ 覆盖搜索条件并重新查询。
 * deep 监听 + 对象身份比较，父组件用 computed 返回新对象即可触发。
 */
watch(
  () => props.searchDefaults,
  (value) => {
    searchParams.value = { ...(value || {}) }
    pagination.current = 1
    fetchData()
  },
  { deep: true },
)

function onTableChange(pag: any) {
  pagination.current = pag.current
  pagination.pageSize = pag.pageSize
  fetchData()
}

function refresh() {
  fetchData()
}

function getCurrentPage() {
  return pagination.current
}

/**
 * 自适应高度核心：
 * 1) 测量 wrap 相对视口顶部 → 整页可用总高；
 * 2) 计算「表格卡片」可达到的目标高度（写死卡片高度，数据再多也不撑高外层）；
 * 3) 若表格内容(tbody)真实高度超出卡片可用空间 → 固定卡片高度 + 表格 body 内部滚动
 *    （表头在上、分页在底、仅内容滚动，外层不出现滚动条）；
 * 4) 若内容不足一屏 → 保持自然高度（不产生大段留白），页面同样不会超高。
 * 仅在数据渲染完成后测量（thead/pagination/tbody 才有真实高度），窗口缩放时重算。
 */
function recomputeFit() {
  if (props.noFit) return
  requestAnimationFrame(() => {
    nextTick(() => {
      const wrap = wrapRef.value
      if (!wrap) return
      const viewportH = window.innerHeight
      const wrapTop = wrap.getBoundingClientRect().top
      const cards = Array.from(wrap.querySelectorAll(':scope > .ant-card')) as HTMLElement[]
      const searchCard = cards.length > 1 ? cards[0] : null
      const tableCard = cards[cards.length - 1]
      if (!tableCard) return

      // 整页可用总高：视口 - wrap顶部 - 底部留白（含 content margin + 滚动条安全余量）
      const bottomReserve = 32
      const usable = viewportH - wrapTop - bottomReserve
      // 查询卡占高（含 mb-16）
      const searchH = searchCard ? searchCard.offsetHeight + 16 : 0
      // 表格卡片目标高度（数据超高时才写死）
      const target = usable - searchH
      if (target < 200) return // 高度异常(如隐藏/未挂载)不处理

      const head = tableCard.querySelector<HTMLElement>('.ant-card-head')
      const headH = head ? head.offsetHeight : 0
      const cardBodyPad = 24 * 2 // antd card body padding 上下各 24
      const thead = tableCard.querySelector<HTMLElement>('.ant-table-thead')
      const theadH = thead ? thead.offsetHeight : 0
      const pagWrap = tableCard.querySelector<HTMLElement>('.ant-table-pagination')
      const pagH = pagWrap ? pagWrap.offsetHeight + 16 : 16
      // 表格 body 可滚动高度上限
      const scrollCap = target - headH - cardBodyPad - theadH - pagH
      // tbody.scrollHeight = 表格内容真实总高（不受 scroll.y 影响）
      const tbody = tableCard.querySelector<HTMLElement>('.ant-table-tbody')
      const contentH = tbody ? tbody.scrollHeight : 0
      const needFit = contentH > scrollCap && scrollCap >= 60

      if (needFit) {
        tableCard.style.height = `${Math.floor(target)}px`
        tableCard.style.overflow = 'hidden'
        fitScrollY.value = Math.floor(scrollCap)
      } else {
        // 内容不足一屏：恢复自然高度
        tableCard.style.height = ''
        tableCard.style.overflow = ''
        fitScrollY.value = 0
      }
    })
  })
}

/** 渲染稳定后再次校准（分页/滚动条出现后布局微变） */
let fitTimer = 0
function scheduleFit() {
  window.clearTimeout(fitTimer)
  fitTimer = window.setTimeout(recomputeFit, 80)
}

let resizeTimer = 0
function onResize() {
  window.clearTimeout(resizeTimer)
  resizeTimer = window.setTimeout(() => recomputeFit(), 150)
}

let wrapObserver: ResizeObserver | undefined
let observerTimer = 0
function setupWrapObserver() {
  const wrap = wrapRef.value
  if (!wrap || typeof ResizeObserver === 'undefined') return
  wrapObserver = new ResizeObserver(() => {
    window.clearTimeout(observerTimer)
    // 自身尺寸变化（含卡片被写死后 antd 重排）再校准一次并收敛
    observerTimer = window.setTimeout(() => {
      // 仅当当前非固定态或高度与目标不一致时才重算，避免自激
      recomputeFit()
    }, 120)
  })
  wrapObserver.observe(wrap)
}

onMounted(() => {
  window.addEventListener('resize', onResize)
  setupWrapObserver()
  if (props.immediate) fetchData()
  else recomputeFit()
})

onBeforeUnmount(() => {
  window.removeEventListener('resize', onResize)
  window.clearTimeout(resizeTimer)
  window.clearTimeout(fitTimer)
  window.clearTimeout(observerTimer)
  wrapObserver?.disconnect()
  wrapObserver = undefined
})

defineExpose({ fetchData, refresh, resetSearch, getCurrentPage, tableData, searchParams })
</script>

<style scoped>
.search-item {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
}

.search-label {
  flex-shrink: 0;
  margin-right: 8px;
  /* 主题自适应：暗色下由 --header-text 提供浅色文字（硬编码深色会导致 label 不可见） */
  color: var(--header-text);
  font-size: 14px;
}

/* 查询条件输入控件统一宽度：占满标签之外的剩余空间（列宽内自适应，各页面一致） */
.search-item :deep(.ant-input),
.search-item :deep(.ant-select),
.search-item :deep(.ant-select-selector),
.search-item :deep(.ant-input-affix-wrapper) {
  flex: 1;
  min-width: 160px;
  width: auto;
}

/* userPicker 自带 display:flex/width:100%，此处仅确保作为 flex item 可拉伸且不与内部宽度冲突 */
.search-item :deep(.user-picker) {
  flex: 1;
  min-width: 0;
  width: auto;
}

.search-item :deep(.user-picker .ant-input) {
  width: 100%;
}
</style>
