<template>
  <div>
    <a-card :bordered="false">
      <a-form
        layout="inline"
        :model="search"
        style="margin-bottom: 16px; row-gap: 12px"
        @finish="fetchData"
      >
        <a-form-item label="租户名称">
          <a-input
            v-model:value="search.tenantName"
            placeholder="请输入租户名称"
            allow-clear
            style="width: 160px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item label="租户编码">
          <a-input
            v-model:value="search.tenantCode"
            placeholder="请输入租户编码"
            allow-clear
            style="width: 160px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item label="创建人">
          <a-input
            v-model:value="search.createBy"
            placeholder="请输入创建人"
            allow-clear
            style="width: 160px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item label="创建时间">
          <a-range-picker
            v-model:value="search.createTimeRange"
            :placeholder="['开始时间', '结束时间']"
            allow-clear
            style="width: 240px"
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
      class="mt-3"
    >
      <template #extra>
        <a-space>
          <a-button
            type="primary"
            @click="openModal()"
          >
            新增租户
          </a-button>
          <a-button
            v-permission="'tenant:export'"
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
        :scroll="{ x: 1600, y: fitY }"
        size="middle"
        row-key="id"
        :row-selection="rowSelection"
        @change="onTableChange"
      >
        <template #bodyCell="{ column, record, index }">
          <template v-if="column.key === 'index'">
            {{ (pagination.current - 1) * pagination.pageSize + index + 1 }}
          </template>
          <template v-if="column.key === 'mode'">
            <a-tag :color="record.mode === 'database' ? 'blue' : 'green'">
              {{ record.mode === 'database' ? '数据库隔离' : '表隔离' }}
            </a-tag>
          </template>
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

    <a-modal
      v-model:open="modalVisible"
      :title="editId ? '编辑租户' : '新增租户'"
      :confirm-loading="submitLoading"
      width="680px"
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
          label="租户名称"
          name="tenantName"
        >
          <a-input
            v-model:value="form.tenantName"
            placeholder="请输入租户名称"
          />
        </a-form-item>
        <a-form-item
          label="租户编码"
          name="tenantCode"
        >
          <a-input
            v-model:value="form.tenantCode"
            :disabled="!!editId"
            placeholder="请输入租户编码（英文）"
          />
        </a-form-item>
        <a-form-item
          v-if="!editId"
          label="隔离模式"
          name="mode"
        >
          <a-radio-group v-model:value="form.mode">
            <a-radio value="table">
              表隔离（共享数据库）
            </a-radio>
            <a-radio value="database">
              数据库隔离（独立数据库）
            </a-radio>
          </a-radio-group>
        </a-form-item>

        <template v-if="form.mode === 'database' && !editId">
          <a-divider>数据库配置</a-divider>
          <a-form-item
            label="数据库类型"
            name="databaseType"
          >
            <a-select
              v-model:value="form.databaseType"
              placeholder="选择数据库类型"
            >
              <a-select-option value="postgres">
                PostgreSQL
              </a-select-option>
              <a-select-option value="mysql">
                MySQL
              </a-select-option>
            </a-select>
          </a-form-item>
          <a-form-item
            label="数据库连接"
            name="databaseUrl"
          >
            <a-input
              v-model:value="form.databaseUrl"
              placeholder="如：postgres://postgres:root@localhost:5432"
            />
          </a-form-item>
          <a-form-item
            label="数据库名称"
            name="databaseName"
          >
            <a-input
              v-model:value="form.databaseName"
              placeholder="如：tenant_acme（将自动创建）"
            />
          </a-form-item>
          <a-divider>管理员账号</a-divider>
          <a-form-item
            label="管理员用户名"
            name="adminUserName"
          >
            <a-input
              v-model:value="form.adminUserName"
              placeholder="请输入管理员用户名"
            />
          </a-form-item>
          <a-form-item
            label="管理员密码"
            name="adminPassWord"
          >
            <a-input-password
              v-model:value="form.adminPassWord"
              placeholder="请输入管理员密码"
            />
          </a-form-item>
          <a-form-item
            label="管理员昵称"
            name="adminNickName"
          >
            <a-input
              v-model:value="form.adminNickName"
              placeholder="请输入管理员昵称（选填）"
            />
          </a-form-item>
        </template>

        <a-form-item
          label="联系人"
          name="contactName"
        >
          <a-input
            v-model:value="form.contactName"
            placeholder="请输入联系人"
          />
        </a-form-item>
        <a-form-item
          label="联系电话"
          name="contactPhone"
        >
          <a-input
            v-model:value="form.contactPhone"
            placeholder="请输入联系电话"
          />
        </a-form-item>
        <a-form-item
          label="联系邮箱"
          name="contactEmail"
        >
          <a-input
            v-model:value="form.contactEmail"
            placeholder="请输入联系邮箱"
          />
        </a-form-item>
        <a-form-item label="状态">
          <a-switch
            v-model:checked="statusChecked"
            checked-children="启用"
            un-checked-children="禁用"
          />
        </a-form-item>
        <a-form-item label="备注">
          <a-textarea
            v-model:value="form.remark"
            placeholder="请输入备注"
          />
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
import { useFitTableHeight } from '@/utils/useFitTableHeight'
import { exportList } from '@/api/export'

const loading = ref(false)
const data = ref<TenantVo[]>([])
const modalVisible = ref(false)
const editId = ref<string | null>(null)
const submitLoading = ref(false)
const formRef = ref<FormInstance>()
const statusLoadingMap = reactive<Record<string, boolean>>({})
const exporting = ref(false)

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

// 行选择（用于勾选导出）
const rowSelection = reactive({
  selectedRowKeys: [] as string[],
  onChange: (selectedRowKeys: string[]) => {
    rowSelection.selectedRowKeys = selectedRowKeys
  },
})

const search = reactive({ 
  // tenantName：界面「租户名称」输入框，发送给后端 TenantQuery.tenantName，筛选 auth_sys_tenant.tenant_name 模糊 contains
  tenantName: '', 
  // tenantCode：界面「租户编码」输入框，发送给后端 TenantQuery.tenantCode，筛选 auth_sys_tenant.tenant_code 模糊 contains
  tenantCode: '',
  // createBy：界面「创建人」输入框，发送给后端 TenantQuery.createBy，筛选 auth_sys_tenant.create_by 模糊 contains
  createBy: '',
  // createTimeRange：界面「创建时间」范围，拆分发送给后端 TenantQuery.createTimeStart/createTimeEnd，筛选 auth_sys_tenant.create_time 区间（>= 开始，<= 结束）
  createTimeRange: [null, null] as [string | null, string | null],
})

const form = reactive({
  tenantName: '',
  tenantCode: '',
  mode: 'table',
  databaseType: 'postgres',
  databaseUrl: '',
  databaseName: '',
  adminUserName: '',
  adminPassWord: '',
  adminNickName: '',
  status: 1,
  contactName: '',
  contactPhone: '',
  contactEmail: '',
  remark: '',
})

const formRules: Record<string, Rule[]> = {
  tenantName: [{ required: true, message: '请输入租户名称', trigger: 'blur' }],
  tenantCode: [{ required: true, message: '请输入租户编码', trigger: 'blur' }],
  mode: [{ required: true, message: '请选择隔离模式', trigger: 'change' }],
  databaseType: [{ required: true, message: '请选择数据库类型', trigger: 'change' }],
  databaseUrl: [{ required: true, message: '请输入数据库连接', trigger: 'blur' }],
  databaseName: [{ required: true, message: '请输入数据库名称', trigger: 'blur' }],
  adminUserName: [{ required: true, message: '请输入管理员用户名', trigger: 'blur' }],
  adminPassWord: [{ required: true, message: '请输入管理员密码', trigger: 'blur' }],
  contactEmail: [{ type: 'email', message: '请输入正确的邮箱格式', trigger: 'blur' }],
  contactPhone: [{ pattern: /^1[3-9]\d{9}$/, message: '请输入正确的手机号', trigger: 'blur' }],
}

const statusChecked = computed({
  get: () => form.status === 1,
  set: (v: boolean) => { form.status = v ? 1 : 0 },
})

const columns = [
  { title: '序号', key: 'index', width: 64, fixed: 'left' as const },
  { title: '租户名称', dataIndex: 'tenantName', key: 'tenantName', width: 140 },
  { title: '租户编码', dataIndex: 'tenantCode', key: 'tenantCode', width: 120 },
  { title: '隔离模式', key: 'mode', width: 110 },
  { title: '数据库名', dataIndex: 'databaseName', key: 'databaseName', width: 130 },
  { title: '联系人', dataIndex: 'contactName', key: 'contactName', width: 100 },
  { title: '状态', key: 'status', width: 100 },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 180 },
  { title: '创建人', dataIndex: 'createBy', key: 'createBy', width: 120 },
  { title: '修改时间', dataIndex: 'updateTime', key: 'updateTime', width: 180 },
  { title: '修改人', dataIndex: 'updateBy', key: 'updateBy', width: 120 },
  { title: '操作', key: 'action', width: 150, fixed: 'right' as const },
]

async function fetchData() {
  loading.value = true
  try {
    const params: Record<string, any> = {
      page: pagination.current,
      pageSize: pagination.pageSize,
      tenantName: search.tenantName || undefined,
      tenantCode: search.tenantCode || undefined,
      createBy: search.createBy || undefined,
    }
    // 时间范围拆分为 createTimeStart/createTimeEnd → TenantQuery.createTimeStart/End，筛选 auth_sys_tenant.create_time 区间（前者 gte，后者 lte）
    if (search.createTimeRange && search.createTimeRange.length === 2) {
      if (search.createTimeRange[0]) {
        params.createTimeStart = search.createTimeRange[0]
      }
      if (search.createTimeRange[1]) {
        params.createTimeEnd = search.createTimeRange[1]
      }
    }
    const res = await tenantApi.list(params)
    data.value = res.items
    pagination.total = res.total
  } catch {
    message.error('查询租户列表失败')
  } finally {
    loading.value = false
  }
}

function onCreateTimeRangeChange(value: [string | null, string | null]) {
  search.createTimeRange = value
}

function resetSearch() {
  search.tenantName = ''
  search.tenantCode = ''
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
    form.mode = record.mode || 'table'
    form.databaseType = record.databaseType || 'postgres'
    form.databaseUrl = record.databaseUrl || ''
    form.databaseName = record.databaseName || ''
    form.adminUserName = ''
    form.adminPassWord = ''
    form.adminNickName = ''
    form.status = record.status
    form.contactName = record.contactName || ''
    form.contactPhone = record.contactPhone || ''
    form.contactEmail = record.contactEmail || ''
    form.remark = record.remark || ''
  } else {
    form.tenantName = ''
    form.tenantCode = ''
    form.mode = 'table'
    form.databaseType = 'postgres'
    form.databaseUrl = ''
    form.databaseName = ''
    form.adminUserName = ''
    form.adminPassWord = ''
    form.adminNickName = ''
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
      await tenantApi.update(editId.value, {
        tenantName: form.tenantName,
        tenantCode: form.tenantCode,
        status: form.status,
        contactName: form.contactName,
        contactPhone: form.contactPhone,
        contactEmail: form.contactEmail,
        remark: form.remark,
        version: 1,
      })
      message.success('更新成功')
    } else if (form.mode === 'database') {
      await tenantApi.createFull({
        tenantName: form.tenantName,
        tenantCode: form.tenantCode,
        databaseType: form.databaseType,
        databaseUrl: form.databaseUrl,
        databaseName: form.databaseName,
        adminUserName: form.adminUserName,
        adminPassWord: form.adminPassWord,
        adminNickName: form.adminNickName || undefined,
        contactName: form.contactName || undefined,
        contactPhone: form.contactPhone || undefined,
        contactEmail: form.contactEmail || undefined,
        remark: form.remark || undefined,
      })
      message.success('租户创建成功（数据库隔离模式）')
    } else {
      await tenantApi.create({
        tenantName: form.tenantName,
        tenantCode: form.tenantCode,
        mode: 'table',
        status: form.status,
        contactName: form.contactName || undefined,
        contactPhone: form.contactPhone || undefined,
        contactEmail: form.contactEmail || undefined,
        remark: form.remark || undefined,
      })
      message.success('租户创建成功（表隔离模式）')
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

// 导出功能：支持按时间返回、勾选导出
async function doExport() {
  const selectedKeys = rowSelection.selectedRowKeys
  const params: Record<string, any> = {
    tenantName: search.tenantName || undefined,
    tenantCode: search.tenantCode || undefined,
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
      taskType: 'tenant',
      query: params,
      syncUrl: '/api/system/tenants/export',
      filename: `tenants-export-${Date.now()}.xlsx`,
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

<style scoped>
.mb-4 {
  margin-bottom: 16px;
}
.mt-3 {
  margin-top: 12px;
}
</style>
