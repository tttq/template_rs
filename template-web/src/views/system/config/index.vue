<template>
  <a-card title="参数配置" :bordered="false">
    <div style="margin-bottom: 16px; display: flex; justify-content: space-between; align-items: center">
      <a-space>
        <a-input v-model:value="search.configName" placeholder="配置名称" allow-clear style="width: 160px" @pressEnter="fetchData" />
        <a-input v-model:value="search.configKey" placeholder="配置键" allow-clear style="width: 160px" @pressEnter="fetchData" />
        <a-button type="primary" @click="fetchData">查询</a-button>
        <a-button @click="resetSearch">重置</a-button>
      </a-space>
      <a-button type="primary" @click="openModal()">新增配置</a-button>
    </div>
    <a-table :columns="columns" :data-source="data" :loading="loading" :pagination="pagination" @change="onTableChange" row-key="id" size="middle">
      <template #bodyCell="{ column, record }">
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
            <a-popconfirm title="确定删除?" @confirm="doDelete(record.id)">
              <a style="color: red">删除</a>
            </a-popconfirm>
          </a-space>
        </template>
      </template>
    </a-table>
  </a-card>

  <a-modal v-model:open="modalVisible" :title="editId ? '编辑配置' : '新增配置'" :confirm-loading="submitLoading" @ok="onSubmit">
    <a-form ref="formRef" :model="form" :rules="formRules" :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }">
      <a-form-item label="配置名称" name="configName">
        <a-input v-model:value="form.configName" placeholder="请输入配置名称" />
      </a-form-item>
      <a-form-item label="配置键" name="configKey">
        <a-input v-model:value="form.configKey" :disabled="!!editId" placeholder="请输入配置键" />
      </a-form-item>
      <a-form-item label="配置值" name="configValue">
        <a-input v-model:value="form.configValue" placeholder="请输入配置值" />
      </a-form-item>
      <a-form-item label="配置类型" name="configType">
        <a-select v-model:value="form.configType" placeholder="请选择配置类型">
          <a-select-option value="string">string</a-select-option>
          <a-select-option value="number">number</a-select-option>
          <a-select-option value="boolean">boolean</a-select-option>
          <a-select-option value="json">json</a-select-option>
        </a-select>
      </a-form-item>
      <a-form-item label="备注" name="remark">
        <a-textarea v-model:value="form.remark" placeholder="请输入备注" :rows="3" />
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { message } from 'ant-design-vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import { configApi, type ConfigVo } from '@/api/config'

const loading = ref(false)
const submitLoading = ref(false)
const data = ref<ConfigVo[]>([])
const modalVisible = ref(false)
const editId = ref<string | null>(null)
const formRef = ref<FormInstance>()

const pagination = reactive({
  current: 1,
  pageSize: 10,
  total: 0,
  showSizeChanger: true,
  showQuickJumper: true,
  showTotal: (total: number) => `共 ${total} 条`,
  pageSizeOptions: ['10', '20', '50', '100'],
})

const search = reactive({ configName: '', configKey: '' })

const form = reactive({ configName: '', configKey: '', configValue: '', configType: 'string', remark: '' })

const formRules: Record<string, Rule[]> = {
  configName: [{ required: true, message: '请输入配置名称', trigger: 'blur' }],
  configKey: [{ required: true, message: '请输入配置键', trigger: 'blur' }],
  configValue: [{ required: true, message: '请输入配置值', trigger: 'blur' }],
  configType: [{ required: true, message: '请选择配置类型', trigger: 'change' }],
}

const columns = [
  { title: 'ID', dataIndex: 'id', key: 'id', width: 80 },
  { title: '名称', dataIndex: 'configName', key: 'configName', width: 150 },
  { title: '键', dataIndex: 'configKey', key: 'configKey', width: 180 },
  { title: '值', dataIndex: 'configValue', key: 'configValue', ellipsis: true },
  { title: '类型', dataIndex: 'configType', key: 'configType', width: 100 },
  { title: '备注', key: 'remark', ellipsis: true },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 180 },
  { title: '操作', key: 'action', width: 150 },
]

async function fetchData() {
  loading.value = true
  try {
    const res = await configApi.list({
      page: pagination.current,
      pageSize: pagination.pageSize,
      configName: search.configName || undefined,
      configKey: search.configKey || undefined,
    })
    data.value = res.items
    pagination.total = res.total
  } catch {
    message.error('查询配置列表失败')
  } finally {
    loading.value = false
  }
}

function resetSearch() {
  search.configName = ''
  search.configKey = ''
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

fetchData()
</script>
