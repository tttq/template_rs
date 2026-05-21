<template>
  <div>
    <a-row :gutter="16">
      <a-col :span="8">
        <a-card title="数据库表" :bordered="false" style="height: calc(100vh - 180px); overflow: auto">
          <a-input-search v-model:value="tableSearch" placeholder="搜索表名" class="mb-4" allow-clear />
          <a-table
            :columns="[{ title: '表名', dataIndex: 'tableName' }, { title: '注释', dataIndex: 'tableComment', width: 120 }]"
            :data-source="filteredTables"
            :loading="tablesLoading"
            :pagination="false"
            size="small"
            row-key="tableName"
            :custom-row="(record: any) => ({ onClick: () => selectTable(record) })"
            :row-class-name="(record: any) => record.tableName === selectedTable ? 'ant-table-row-selected' : ''"
          />
        </a-card>
      </a-col>
      <a-col :span="16">
        <a-card title="表字段信息" :bordered="false" class="mb-4">
          <a-table
            :columns="columnColumns"
            :data-source="columns"
            :loading="columnsLoading"
            :pagination="false"
            size="small"
            row-key="columnName"
          />
        </a-card>
        <a-card title="生成配置" :bordered="false">
          <a-form :model="config" layout="inline" class="mb-4">
            <a-form-item label="模块名"><a-input v-model:value="config.moduleName" /></a-form-item>
            <a-form-item label="业务名"><a-input v-model:value="config.businessName" /></a-form-item>
            <a-form-item label="实体名"><a-input v-model:value="config.entityName" /></a-form-item>
            <a-form-item label="生成后端"><a-switch v-model:checked="config.generateBackend" /></a-form-item>
            <a-form-item label="生成前端"><a-switch v-model:checked="config.generateFrontend" /></a-form-item>
          </a-form>
          <a-space>
            <a-button type="primary" @click="handlePreview" :disabled="!selectedTable">预览代码</a-button>
            <a-button @click="handleDownload" :disabled="!selectedTable">下载代码</a-button>
          </a-space>
        </a-card>
      </a-col>
    </a-row>
    <a-modal v-model:open="previewVisible" title="代码预览" width="80%" :footer="null">
      <a-tabs v-if="previewFiles.length" v-model:activeKey="activeFile">
        <a-tab-pane v-for="file in previewFiles" :key="file.filePath" :tab="file.fileName">
          <a-typography-paragraph :copyable="{ text: file.content }" :content="file.content" style="white-space: pre; font-family: monospace; font-size: 12px; max-height: 60vh; overflow: auto" />
        </a-tab-pane>
      </a-tabs>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { message } from 'ant-design-vue'
import { generatorApi } from '@/api/generator'
import type { GeneratorConfig } from '@/api/generator'

const tablesLoading = ref(false)
const columnsLoading = ref(false)
const tables = ref<any[]>([])
const columns = ref<any[]>([])
const selectedTable = ref('')
const tableSearch = ref('')
const previewVisible = ref(false)
const previewFiles = ref<any[]>([])
const activeFile = ref('')

const config = ref<GeneratorConfig>({
  tableName: '',
  moduleName: '',
  businessName: '',
  entityName: '',
  generateFrontend: true,
  generateBackend: true,
})

const filteredTables = computed(() => {
  if (!tableSearch.value) return tables.value
  return tables.value.filter(t => t.tableName.toLowerCase().includes(tableSearch.value.toLowerCase()))
})

const columnColumns = [
  { title: '字段名', dataIndex: 'columnName', width: 150 },
  { title: '类型', dataIndex: 'columnType', width: 120 },
  { title: '注释', dataIndex: 'columnComment', width: 150 },
  { title: '可空', dataIndex: 'isNullable', width: 80 },
  { title: '主键', dataIndex: 'isPrimaryKey', width: 80, customRender: ({ text }: any) => text ? '是' : '否' },
  { title: '默认值', dataIndex: 'columnDefault', width: 120 },
]

async function fetchTables() {
  tablesLoading.value = true
  try {
    const res = await generatorApi.listTables()
    tables.value = res
  } finally {
    tablesLoading.value = false
  }
}

async function selectTable(record: any) {
  selectedTable.value = record.tableName
  config.value.tableName = record.tableName
  config.value.moduleName = record.tableName.replace(/_/g, '')
  config.value.businessName = record.tableComment || record.tableName
  config.value.entityName = record.tableName.split('_').map((w: string) => w.charAt(0).toUpperCase() + w.slice(1)).join('')
  columnsLoading.value = true
  try {
    const res = await generatorApi.getTableColumns(record.tableName)
    columns.value = res
  } finally {
    columnsLoading.value = false
  }
}

async function handlePreview() {
  try {
    const res = await generatorApi.preview(config.value)
    previewFiles.value = res.files || []
    if (previewFiles.value.length) activeFile.value = previewFiles.value[0].filePath
    previewVisible.value = true
  } catch {
    message.error('预览失败')
  }
}

async function handleDownload() {
  try {
    const res = await generatorApi.download(config.value)
    const blob = new Blob([res as any], { type: 'application/zip' })
    const url = window.URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'generated-code.zip'
    a.click()
    window.URL.revokeObjectURL(url)
    message.success('下载成功')
  } catch {
    message.error('下载失败')
  }
}

onMounted(() => fetchTables())
</script>
