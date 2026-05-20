<template>
  <div>
    <a-card :bordered="false">
      <a-row :gutter="16" class="mb-4">
        <a-col :span="6">
          <a-input v-model:value="search.tenantName" placeholder="租户名称" allow-clear @pressEnter="fetchData" />
        </a-col>
        <a-col :span="6">
          <a-input v-model:value="search.tenantCode" placeholder="租户编码" allow-clear @pressEnter="fetchData" />
        </a-col>
        <a-col :span="6">
          <a-space>
            <a-button type="primary" @click="fetchData">查询</a-button>
            <a-button @click="resetSearch">重置</a-button>
          </a-space>
        </a-col>
      </a-row>
    </a-card>

    <a-card :bordered="false" class="mt-3">
      <template #title>租户管理</template>
      <template #extra>
        <a-button type="primary" @click="openModal()">新增租户</a-button>
      </template>
      <a-table
        :columns="columns"
        :data-source="data"
        :loading="loading"
        :pagination="pagination"
        :scroll="{ x: 1100 }"
        size="middle"
        row-key="id"
        @change="onTableChange"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'status'">
            <a-switch
              :checked="record.status === 1"
              checked-children="启用"
              un-checked-children="禁用"
              :loading="statusLoadingMap[record.id]"
              @change="(checked: boolean) => onStatusChange(record, checked)"
            />
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

    <a-modal
      v-model:open="modalVisible"
      :title="editId ? '编辑租户' : '新增租户'"
      :confirm-loading="submitLoading"
      width="600px"
      @ok="onSubmit"
    >
      <a-form ref="formRef" :model="form" :rules="formRules" :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }">
        <a-form-item label="租户名称" name="tenantName">
          <a-input v-model:value="form.tenantName" placeholder="请输入租户名称" />
        </a-form-item>
        <a-form-item label="租户编码" name="tenantCode">
          <a-input v-model:value="form.tenantCode" :disabled="!!editId" placeholder="请输入租户编码" />
        </a-form-item>
        <a-form-item label="联系人" name="contactName">
          <a-input v-model:value="form.contactName" placeholder="请输入联系人" />
        </a-form-item>
        <a-form-item label="联系电话" name="contactPhone">
          <a-input v-model:value="form.contactPhone" placeholder="请输入联系电话" />
        </a-form-item>
        <a-form-item label="联系邮箱" name="contactEmail">
          <a-input v-model:value="form.contactEmail" placeholder="请输入联系邮箱" />
        </a-form-item>
        <a-form-item label="状态">
          <a-switch v-model:checked="statusChecked" checked-children="启用" un-checked-children="禁用" />
        </a-form-item>
        <a-form-item label="备注">
          <a-textarea v-model:value="form.remark" placeholder="请输入备注" />
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, computed } from 'vue'
import { message } from 'ant-design-vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import { tenantApi, type TenantVo } from '@/api/tenant'

const loading = ref(false)
const data = ref<TenantVo[]>([])
const modalVisible = ref(false)
const editId = ref<string | null>(null)
const submitLoading = ref(false)
const formRef = ref<FormInstance>()
const statusLoadingMap = reactive<Record<string, boolean>>({})

const pagination = reactive({
  current: 1,
  pageSize: 10,
  total: 0,
  showSizeChanger: true,
  showQuickJumper: true,
  showTotal: (total: number) => `共 ${total} 条`,
  pageSizeOptions: ['10', '20', '50', '100'],
})

const search = reactive({ tenantName: '', tenantCode: '' })

const form = reactive({
  tenantName: '',
  tenantCode: '',
  status: 1,
  contactName: '',
  contactPhone: '',
  contactEmail: '',
  remark: '',
})

const formRules: Record<string, Rule[]> = {
  tenantName: [{ required: true, message: '请输入租户名称', trigger: 'blur' }],
  tenantCode: [{ required: true, message: '请输入租户编码', trigger: 'blur' }],
  contactEmail: [{ type: 'email', message: '请输入正确的邮箱格式', trigger: 'blur' }],
  contactPhone: [{ pattern: /^1[3-9]\d{9}$/, message: '请输入正确的手机号', trigger: 'blur' }],
}

const statusChecked = computed({
  get: () => form.status === 1,
  set: (v: boolean) => { form.status = v ? 1 : 0 },
})

const columns = [
  { title: 'ID', dataIndex: 'id', key: 'id', width: 80 },
  { title: '租户名称', dataIndex: 'tenantName', key: 'tenantName', width: 150 },
  { title: '租户编码', dataIndex: 'tenantCode', key: 'tenantCode', width: 150 },
  { title: '联系人', dataIndex: 'contactName', key: 'contactName', width: 120 },
  { title: '联系电话', dataIndex: 'contactPhone', key: 'contactPhone', width: 140 },
  { title: '状态', key: 'status', width: 100 },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 180 },
  { title: '操作', key: 'action', width: 150, fixed: 'right' as const },
]

async function fetchData() {
  loading.value = true
  try {
    const res = await tenantApi.list({
      page: pagination.current,
      pageSize: pagination.pageSize,
      tenantName: search.tenantName || undefined,
      tenantCode: search.tenantCode || undefined,
    })
    data.value = res.items
    pagination.total = res.total
  } catch {
    message.error('查询租户列表失败')
  } finally {
    loading.value = false
  }
}

function resetSearch() {
  search.tenantName = ''
  search.tenantCode = ''
  pagination.current = 1
  fetchData()
}

function onTableChange(pag: any) {
  pagination.current = pag.current
  pagination.pageSize = pag.pageSize
  fetchData()
}

async function onStatusChange(record: TenantVo, checked: boolean) {
  statusLoadingMap[record.id] = true
  try {
    await tenantApi.update(record.id, { ...record, status: checked ? 1 : 0 })
    message.success(checked ? '已启用' : '已禁用')
    fetchData()
  } catch {
    message.error('状态更新失败')
  } finally {
    statusLoadingMap[record.id] = false
  }
}

function openModal(record?: TenantVo) {
  editId.value = record?.id || null
  if (record) {
    form.tenantName = record.tenantName
    form.tenantCode = record.tenantCode
    form.status = record.status
    form.contactName = record.contactName || ''
    form.contactPhone = record.contactPhone || ''
    form.contactEmail = record.contactEmail || ''
    form.remark = record.remark || ''
  } else {
    form.tenantName = ''
    form.tenantCode = ''
    form.status = 1
    form.contactName = ''
    form.contactPhone = ''
    form.contactEmail = ''
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
      await tenantApi.update(editId.value, { ...form, version: 1 })
      message.success('更新成功')
    } else {
      await tenantApi.create({ ...form })
      message.success('新增成功')
    }
    modalVisible.value = false
    fetchData()
  } catch {
    message.error(editId.value ? '更新失败' : '新增失败')
  } finally {
    submitLoading.value = false
  }
}

async function doDelete(id: string) {
  try {
    await tenantApi.delete(id)
    message.success('删除成功')
    fetchData()
  } catch {
    message.error('删除失败')
  }
}

fetchData()
</script>

<style scoped>
.mb-4 {
  margin-bottom: 16px;
}
.mt-3 {
  margin-top: 12px;
}
</style>
