<template>
  <div>
    <a-card v-if="searchFields.length > 0" :bordered="false" class="mb-16">
      <a-row :gutter="16">
        <template v-for="item in searchFields" :key="item.field">
          <a-col :span="6" :xs="24" :sm="12" :md="8" :lg="6">
            <div class="search-item">
              <span class="search-label">{{ item.label }}:</span>
              <a-input
                v-if="item.type === 'input'"
                :value="searchParams[item.field]"
                :placeholder="item.placeholder || `请输入`"
                allow-clear
                @update:value="onSearchUpdate(item.field, $event)"
                @pressEnter="fetchData"
              />
              <a-select
                v-else-if="item.type === 'select'"
                :value="searchParams[item.field]"
                :placeholder="item.placeholder || `请选择`"
                :options="item.options"
                allow-clear
                @update:value="onSearchUpdate(item.field, $event)"
              />
              <DictSelect
                v-else-if="item.type === 'dictSelect'"
                :value="searchParams[item.field]"
                :dict-code="item.dictCode!"
                :placeholder="item.placeholder || `请选择`"
                allow-clear
                @update:value="onSearchUpdate(item.field, $event)"
              />
            </div>
          </a-col>
        </template>
        <a-col :span="6" :xs="24" :sm="12" :md="8" :lg="6">
          <a-space>
            <a-button type="primary" @click="fetchData">{{ t('common.search') }}</a-button>
            <a-button @click="resetSearch">{{ t('common.reset') }}</a-button>
          </a-space>
        </a-col>
      </a-row>
    </a-card>

    <a-card :bordered="false">
      <template v-if="title" #title>{{ title }}</template>
      <template #extra>
        <slot name="toolbar" />
      </template>
      <a-table
        :columns="columns"
        :data-source="tableData"
        :loading="loading"
        :pagination="paginationConfig"
        :scroll="scroll"
        :row-key="rowKey"
        :row-selection="rowSelection"
        size="middle"
        @change="onTableChange"
      >
        <template #bodyCell="{ column, record, index }">
          <slot name="bodyCell" :column="column" :record="record" :index="index" />
        </template>
      </a-table>
    </a-card>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type { FormField, TableColumn, PageResult } from './types'
import DictSelect from '../DictSelect.vue'

const props = withDefaults(defineProps<{
  columns: TableColumn[]
  searchFields?: FormField[]
  apiFn: (params: Record<string, any>) => Promise<PageResult<any>>
  title?: string
  rowKey?: string
  scroll?: Record<string, number | string>
  rowSelection?: any
  pageSize?: number
  immediate?: boolean
}>(), {
  searchFields: () => [],
  rowKey: 'id',
  scroll: () => ({ x: 1100 }),
  pageSize: 10,
  immediate: true,
})

const emit = defineEmits<{
  'loaded': [data: any[], total: number]
}>()

const { t } = useI18n()

const loading = ref(false)
const tableData = ref<any[]>([])
const searchParams = ref<Record<string, any>>({})

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
  pageSizeOptions: ['10', '20', '50', '100'],
  showTotal: (total: number) => `共 ${total} 条`,
}))

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
        params[k] = v
      }
    }
    const res = await props.apiFn(params)
    tableData.value = res.items
    pagination.total = res.total
    emit('loaded', res.items, res.total)
  } finally {
    loading.value = false
  }
}

function resetSearch() {
  searchParams.value = {}
  pagination.current = 1
  fetchData()
}

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

if (props.immediate) {
  fetchData()
}

defineExpose({ fetchData, refresh, resetSearch, getCurrentPage, tableData })
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
  color: rgba(0, 0, 0, 0.85);
  font-size: 14px;
}
</style>
