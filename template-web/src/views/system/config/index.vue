<template>
  <div>
  <a-card :bordered="false">
      <a-form
        layout="inline"
        :model="search"
        style="flex: 1; min-width: 720px; margin-bottom: 16px"
        @finish="fetchData"
      >
        <a-form-item label="配置名称">
          <a-input
            v-model:value="search.configName"
            placeholder="请输入配置名称"
            allow-clear
            style="width: 150px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item label="配置键">
          <a-input
            v-model:value="search.configKey"
            placeholder="请输入配置键"
            allow-clear
            style="width: 150px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item label="创建人">
          <a-input
            v-model:value="search.createBy"
            placeholder="请输入创建人"
            allow-clear
            style="width: 150px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item label="创建时间">
          <a-range-picker
            v-model:value="search.createTimeRange"
            :placeholder="['开始时间', '结束时间']"
            allow-clear
            style="width: 220px"
            @change="onCreateTimeRangeChange"
          />
        </a-form-item>
        <a-form-item>
          <a-space>
            <a-button
              type="primary"
              html-type="submit"
            >
              查询
            </a-button>
            <a-button @click="resetSearch">
              重置
            </a-button>
          </a-space>
        </a-form-item>
      </a-form>
  </a-card>

  <a-card
    ref="fitRef"
    :bordered="false"
    style="margin-top: 12px"
  >
    <template #extra>
      <a-space>
        <a-button
          type="primary"
          @click="openModal()"
        >
          新增配置
        </a-button>
        <a-button
          v-permission="'config:export'"
          :loading="exporting"
          @click="doExport"
        >
          导出 Excel
        </a-button>
      </a-space>
    </template>
    <a-table
      :columns="columns"
      :data-source="data"
      :loading="loading"
      :pagination="pagination"
      :scroll="{ x: 1100, y: fitY }"
      row-key="id"
      size="middle"
      :row-selection="rowSelection"
      @change="onTableChange"
    >
      <template #bodyCell="{ column, record, index }">
        <template v-if="column.key === 'index'">
          {{ (pagination.current - 1) * pagination.pageSize + index + 1 }}
        </template>
        <template v-if="column.key === 'configType'">
          <a-tag>{{ record.configType }}</a-tag>
        </template>
        <template v-if="column.key === 'remark'">
          <span>{{ record.remark || '-' }}</span>
        </template>
        <template v-if="column.key === 'action'">
          <a-space>
            <a @click.prevent="openModal(record)">编辑</a>
            <a-divider type="vertical" />
            <a-popconfirm
              title="确定删除?"
              @confirm="doDelete(record.id)"
            >
              <a style="color: red">删除</a>
            </a-popconfirm>
          </a-space>
        </template>
      </template>
    </a-table>
  </a-card>
  </div>

  <a-modal
    v-model:open="modalVisible"
    :title="editId ? '编辑配置' : '新增配置'"
    :confirm-loading="submitLoading"
    @ok="onSubmit"
  >
    <a-form
      ref="formRef"
      :model="form"
      :rules="formRules"
      :label-col="{ span: 6 }"
      :wrapper-col="{ span: 16 }"
    >
      <a-form-item
        label="配置名称"
        name="configName"
      >
        <a-input
          v-model:value="form.configName"
          placeholder="请输入配置名称"
        />
      </a-form-item>
      <a-form-item
        label="配置键"
        name="configKey"
      >
        <a-input
          v-model:value="form.configKey"
          :disabled="!!editId"
          placeholder="请输入配置键"
        />
      </a-form-item>
      <a-form-item
        label="配置值"
        name="configValue"
      >
        <a-input
          v-model:value="form.configValue"
          placeholder="请输入配置值"
        />
      </a-form-item>
      <a-form-item
        label="配置类型"
        name="configType"
      >
        <a-select
          v-model:value="form.configType"
          placeholder="请选择配置类型"
        >
          <a-select-option value="string">
            string
          </a-select-option>
          <a-select-option value="number">
            number
          </a-select-option>
          <a-select-option value="boolean">
            boolean
          </a-select-option>
          <a-select-option value="json">
            json
          </a-select-option>
        </a-select>
      </a-form-item>
      <a-form-item
        label="备注"
        name="remark"
      >
        <a-textarea
          v-model:value="form.remark"
          placeholder="请输入备注"
          :rows="3"
        />
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { message } from 'ant-design-vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import { configApi, type ConfigVo } from '@/api/config'
import { useFitTableHeight } from '@/utils/useFitTableHeight'
import { exportList } from '@/api/export'

const loading = ref(false)
const submitLoading = ref(false)
const data = ref<ConfigVo[]>([])
const modalVisible = ref(false)
const editId = ref<string | null>(null)
const formRef = ref<FormInstance>()
const exporting = ref(false)

// 行选择（用于勾选导出）
const rowSelection = reactive({
  selectedRowKeys: [] as string[],
  onChange: (selectedRowKeys: string[]) => {
    rowSelection.selectedRowKeys = selectedRowKeys
  },
})

const { fitRef, fitY } = useFitTableHeight()

const pagination = reactive({
  current: 1,
  pageSize: 20,
  total: 0,
  showSizeChanger: true,
  showQuickJumper: true,
  showTotal: (total: number) => `共 ${total} 条`,
  pageSizeOptions: ['20', '40', '60', '100'],
})

const search = reactive({ 
  // configName：界面「配置名称」输入框，发送给后端 ConfigQuery.configName，筛选 auth_sys_config.config_name 模糊 contains
  configName: '', 
  // configKey：界面「配置键」输入框，发送给后端 ConfigQuery.configKey，筛选 auth_sys_config.config_key 模糊 contains
  configKey: '',
  // createBy：界面「创建人」输入框，发送给后端 ConfigQuery.createBy，筛选 auth_sys_config.create_by 模糊 contains
  createBy: '',
  // createTimeRange：界面「创建时间」范围，拆分发送给后端 ConfigQuery.createTimeStart/createTimeEnd，筛选 auth_sys_config.create_time 区间（>= 开始，<= 结束）
  createTimeRange: [null, null] as [string | null, string | null],
})

const form = reactive({ configName: '', configKey: '', configValue: '', configType: 'string', remark: '' })

const formRules: Record<string, Rule[]> = {
  configName: [{ required: true, message: '请输入配置名称', trigger: 'blur' }],
  configKey: [{ required: true, message: '请输入配置键', trigger: 'blur' }],
  configValue: [{ required: true, message: '请输入配置值', trigger: 'blur' }],
  configType: [{ required: true, message: '请选择配置类型', trigger: 'change' }],
}

const columns = [
  { title: '序号', key: 'index', width: 64 },
  { title: '名称', dataIndex: 'configName', key: 'configName', width: 150 },
  { title: '键', dataIndex: 'configKey', key: 'configKey', width: 180 },
  { title: '值', dataIndex: 'configValue', key: 'configValue', ellipsis: true },
  { title: '类型', dataIndex: 'configType', key: 'configType', width: 100 },
  { title: '备注', key: 'remark', ellipsis: true },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 180 },
  { title: '创建人', dataIndex: 'createBy', key: 'createBy', width: 120 },
  { title: '修改时间', dataIndex: 'updateTime', key: 'updateTime', width: 180 },
  { title: '修改人', dataIndex: 'updateBy', key: 'updateBy', width: 120 },
  { title: '操作', key: 'action', width: 150 },
]

async function fetchData() {
  loading.value = true
  try {
    const params: Record<string, any> = {
      page: pagination.current,
      pageSize: pagination.pageSize,
      configName: search.configName || undefined,
      configKey: search.configKey || undefined,
      createBy: search.createBy || undefined,
    }
    // 时间范围拆分为 createTimeStart/createTimeEnd → ConfigQuery.createTimeStart/End，筛选 auth_sys_config.create_time 区间（前者 gte，后者 lte）
    if (search.createTimeRange && search.createTimeRange.length === 2) {
      if (search.createTimeRange[0]) {
        params.createTimeStart = search.createTimeRange[0]
      }
      if (search.createTimeRange[1]) {
        params.createTimeEnd = search.createTimeRange[1]
      }
    }
    const res = await configApi.list(params)
    data.value = res.items
    pagination.total = res.total
  } catch {
    message.error('查询配置列表失败')
  } finally {
    loading.value = false
  }
}

function onCreateTimeRangeChange(value: [string | null, string | null]) {
  search.createTimeRange = value
}

function resetSearch() {
  search.configName = ''
  search.configKey = ''
  search.createBy = ''
  search.createTimeRange = [null, null]
  pagination.current = 1
  fetchData()
}

function onTableChange(pag: any) {
  pagination.current = pag.current
  pagination.pageSize = pag.pageSize
  fetchData()
}

function openModal(record?: ConfigVo) {
  editId.value = record?.id || null
  if (record) {
    form.configName = record.configName
    form.configKey = record.configKey
    form.configValue = record.configValue
    form.configType = record.configType
    form.remark = record.remark || ''
  } else {
    form.configName = ''
    form.configKey = ''
    form.configValue = ''
    form.configType = 'string'
    form.remark = ''
  }
  modalVisible.value = true
  setTimeout(() => formRef.value?.clearValidate(), 0)
}

async function onSubmit() {
  try {
    await formRef.value?.validate()
  } catch {
    return
  }
  submitLoading.value = true
  try {
    if (editId.value) {
      await configApi.update(editId.value, { ...form, version: 1 })
      message.success('更新成功')
    } else {
      await configApi.create({ ...form })
      message.success('新增成功')
    }
    modalVisible.value = false
    fetchData()
  } catch {
    message.error(editId.value ? '更新配置失败' : '新增配置失败')
  } finally {
    submitLoading.value = false
  }
}

async function doDelete(id: string) {
  try {
    await configApi.delete(id)
    message.success('删除成功')
    fetchData()
  } catch {
    message.error('删除配置失败')
  }
}

// 导出功能：支持按时间返回、勾选导出
async function doExport() {
  const selectedKeys = rowSelection.selectedRowKeys
  const params: Record<string, any> = {
    configName: search.configName || undefined,
    configKey: search.configKey || undefined,
    createBy: search.createBy || undefined,
  }
  if (search.createTimeRange && search.createTimeRange.length === 2) {
    if (search.createTimeRange[0]) {
      params.createTimeStart = search.createTimeRange[0]
    }
    if (search.createTimeRange[1]) {
      params.createTimeEnd = search.createTimeRange[1]
    }
  }
  // 如果有勾选行，只导出勾选的行
  if (selectedKeys.length > 0) {
    params.ids = selectedKeys.join(',')
  }
  exporting.value = true
  try {
    const { mode } = await exportList({
      taskType: 'config',
      query: params,
      syncUrl: '/api/system/configs/export',
      filename: `configs-export-${Date.now()}.xlsx`,
    })
    if (mode === 'async') {
      // 超过 10 万条：自动转异步导出，完成后到「导出中心」查看/下载
      message.success('导出成功，请自行到导出中心查看')
    } else {
      message.success('导出成功')
    }
  } catch (e: any) {
    message.error(e.message || '导出失败')
  } finally {
    exporting.value = false
  }
}

fetchData()
</script>
