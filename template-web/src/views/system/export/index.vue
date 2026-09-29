<template>
  <div>
    <ProTable
      ref="tableRef"
      :columns="columns"
      :search-fields="[]"
      :api-fn="apiFn"
      :show-index="false"
      :scroll="{ x: 900 }"
    >
      <template #toolbar>
        <a-space>
          <a-button @click="router.push('/dashboard')">
            返回首页
          </a-button>
          <a-tag color="blue">
            处理中的任务会自动刷新
          </a-tag>
        </a-space>
      </template>
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'status'">
          <a-tag :color="statusColor(record.status)">
            {{ statusText(record.status) }}
          </a-tag>
        </template>
        <template v-if="column.key === 'errorMsg'">
          <a-tooltip
            v-if="record.errorMsg"
            :title="record.errorMsg"
            placement="top"
          >
            <span class="cell-ellipsis">{{ record.errorMsg }}</span>
          </a-tooltip>
          <span v-else>-</span>
        </template>
        <template v-if="column.key === 'action'">
          <template v-if="record.status === 'success'">
            <a
              :loading="downloadingId === record.id"
              @click.prevent="doDownload(record)"
            >下载</a>
          </template>
          <span v-else>-</span>
        </template>
      </template>
    </ProTable>
  </div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import { message } from 'ant-design-vue'
import { useRouter } from 'vue-router'
import ProTable from '@/components/common/ProTable.vue'
import { exportTaskApi, type ExportTaskVo } from '@/api/system/exportTask'

const router = useRouter()
const tableRef = ref<InstanceType<typeof ProTable>>()
const downloadingId = ref<string>('')

const columns = [
  { title: '任务编号', dataIndex: 'taskNo', key: 'taskNo', width: 200 },
  { title: '类型', dataIndex: 'taskType', key: 'taskType', width: 140, customRender: ({ text }: any) => TASK_TYPE_TEXT[text] || text },
  { title: '状态', key: 'status', width: 100 },
  { title: '总行数', dataIndex: 'totalRows', key: 'totalRows', width: 90 },
  { title: '成功行数', dataIndex: 'successRows', key: 'successRows', width: 90 },
  { title: '文件大小(KB)', key: 'fileSize', width: 110, customRender: ({ text }: any) => (text ? (text / 1024).toFixed(1) : '-') },
  { title: '失败原因', key: 'errorMsg', width: 220 },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 170 },
  { title: '操作', key: 'action', width: 80 },
]

/** 导出中心任务类型 → 中文名（与后端注册的 task_type 一一对应） */
const TASK_TYPE_TEXT: Record<string, string> = {
  product: '选品带图导出',
  factory: '工厂导出',
  packaging: '包装规格导出',
  user: '用户导出',
  role: '角色导出',
  dept: '部门导出',
  dict: '字典类型导出',
  config: '参数配置导出',
  tenant: '租户导出',
  menu: '菜单导出',
}

const STATUS_TEXT: Record<string, string> = {
  pending: '排队中',
  processing: '生成中',
  success: '已完成',
  failed: '失败',
}
const STATUS_COLORS: Record<string, string> = {
  pending: 'orange',
  processing: 'blue',
  success: 'green',
  failed: 'red',
}

function statusText(s: string) {
  return STATUS_TEXT[s] || s
}
function statusColor(s: string) {
  return STATUS_COLORS[s] || 'default'
}

async function apiFn(params: Record<string, any>) {
  return exportTaskApi.list(params)
}

async function doDownload(record: ExportTaskVo) {
  downloadingId.value = record.id
  try {
    await exportTaskApi.download(record.id)
  } catch (e: any) {
    message.error(e.message || '下载失败')
  } finally {
    downloadingId.value = ''
  }
}

/** 有进行中任务时每 30s 自动刷新 */
function isRunning(record: any) {
  return record?.status === 'pending' || record?.status === 'processing'
}
function hasRunningTask(): boolean {
  const rows = tableRef.value?.tableData || []
  return rows.some(isRunning)
}
let timer: ReturnType<typeof setInterval> | null = null
onMounted(() => {
  timer = setInterval(() => {
    if (hasRunningTask()) tableRef.value?.refresh()
  }, 30000)
})
onUnmounted(() => {
  if (timer) clearInterval(timer)
})
</script>
