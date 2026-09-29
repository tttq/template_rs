<template>
  <div>
    <a-card :bordered="false">
      <a-form
        layout="inline"
        :model="searchForm"
        style="margin-bottom: 16px; row-gap: 12px"
        @finish="fetchData"
      >
        <a-form-item label="用户名">
          <a-input
            v-model:value="searchForm.userName"
            placeholder="请输入用户名"
            allow-clear
            style="width: 160px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item label="邮箱">
          <a-input
            v-model:value="searchForm.email"
            placeholder="请输入邮箱"
            allow-clear
            style="width: 160px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item label="手机号">
          <a-input
            v-model:value="searchForm.phone"
            placeholder="请输入手机号"
            allow-clear
            style="width: 160px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item label="创建人">
          <a-input
            v-model:value="searchForm.createBy"
            placeholder="请输入创建人"
            allow-clear
            style="width: 160px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item label="创建时间">
          <a-range-picker
            v-model:value="searchForm.createTimeRange"
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
            新增用户
          </a-button>
          <a-button
            v-permission="'user:export'"
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
        :scroll="{ x: 1500, y: fitY }"
        size="middle"
        row-key="id"
        :row-selection="rowSelection"
        @change="onTableChange"
      >
        <template #bodyCell="{ column, record, index }">
          <template v-if="column.key === 'index'">
            {{ (pagination.current - 1) * pagination.pageSize + index + 1 }}
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
              <a @click.prevent="openRoleModal(record)">分配角色</a>
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
      :title="editId ? '编辑用户' : '新增用户'"
      :confirm-loading="submitLoading"
      width="600px"
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
          label="用户名"
          name="userName"
        >
          <a-input
            v-model:value="form.userName"
            :disabled="!!editId"
            placeholder="请输入用户名"
          />
        </a-form-item>
        <a-form-item
          label="密码"
          :name="editId ? undefined : 'passWord'"
        >
          <a-input-password
            v-model:value="form.passWord"
            :placeholder="editId ? '留空不修改' : '请输入密码'"
          />
        </a-form-item>
        <a-form-item
          label="昵称"
          name="nickName"
        >
          <a-input
            v-model:value="form.nickName"
            placeholder="请输入昵称"
          />
        </a-form-item>
        <a-form-item
          label="邮箱"
          name="email"
        >
          <a-input
            v-model:value="form.email"
            placeholder="请输入邮箱"
          />
        </a-form-item>
        <a-form-item
          label="手机号"
          name="phone"
        >
          <a-input
            v-model:value="form.phone"
            placeholder="请输入手机号"
          />
        </a-form-item>
        <a-form-item label="状态">
          <a-switch
            v-model:checked="statusChecked"
            checked-children="启用"
            un-checked-children="禁用"
          />
        </a-form-item>
      </a-form>
    </a-modal>

    <a-modal
      v-model:open="roleModalVisible"
      title="分配角色"
      :confirm-loading="roleSubmitLoading"
      width="700px"
      @ok="onAssignRoles"
    >
      <a-spin :spinning="roleLoading">
        <div style="margin-bottom: 12px">
          <a-space>
            <span>客户端：</span>
            <a-select
              v-model:value="roleClientId"
              placeholder="请选择客户端"
              style="width: 280px"
              @change="onRoleClientChange"
            >
              <a-select-option
                v-for="item in clientOptions"
                :key="item.id"
                :value="item.id"
              >
                {{ item.clientName }}（{{ item.clientCode }}）
              </a-select-option>
            </a-select>
            <span style="color: #8c8c8c">角色按客户端隔离，一个账号可在多个客户端拥有不同角色</span>
          </a-space>
        </div>
        <a-transfer
          v-model:target-keys="checkedRoleIds"
          :data-source="transferRoles"
          :render="(item: TransferItem) => item.title"
          :titles="['未分配', '已分配']"
          show-search
          :filter-option="filterTransferOption"
          list-style="width: 300px; height: 400px;"
        />
      </a-spin>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, computed } from 'vue'
import { message } from 'ant-design-vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import type { TransferItem } from 'ant-design-vue/es/transfer'
import { userApi, type UserVo } from '@/api/user'
import { roleApi, type RoleVo } from '@/api/role'
import { clientApi, type ClientOption } from '@/api/client'
import { useFitTableHeight } from '@/utils/useFitTableHeight'
import { exportList } from '@/api/export'

const searchForm = reactive({
  // userName：界面「用户名」输入框，发送给后端 UserQuery.userName，筛选 auth_sys_user.user_name 模糊 contains
  userName: '',
  // email：界面「邮箱」输入框，发送给后端 UserQuery.email，筛选 auth_sys_user.email 模糊 contains
  email: '',
  // phone：界面「手机号」输入框，发送给后端 UserQuery.phone，筛选 auth_sys_user.phone 模糊 contains
  phone: '',
  // createBy：界面「创建人」输入框，发送给后端 UserQuery.createBy，筛选 auth_sys_user.create_by 模糊 contains
  createBy: '',
  // createTimeRange：界面「创建时间」范围，拆分发送给后端 UserQuery.createTimeStart/createTimeEnd，筛选 auth_sys_user.create_time 区间（>= 开始，<= 结束）
  createTimeRange: [null, null] as [string | null, string | null],
})

const loading = ref(false)
const data = ref<UserVo[]>([])
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

const form = reactive({
  userName: '',
  passWord: '',
  nickName: '',
  email: '',
  phone: '',
  status: 1,
})

const formRules: Record<string, Rule[]> = {
  userName: [{ required: true, message: '请输入用户名', trigger: 'blur' }],
  passWord: [{ required: true, message: '请输入密码', trigger: 'blur' }],
  email: [{ type: 'email', message: '请输入正确的邮箱格式', trigger: 'blur' }],
  phone: [{ pattern: /^1[3-9]\d{9}$/, message: '请输入正确的手机号', trigger: 'blur' }],
}

const statusChecked = computed({
  get: () => form.status === 1,
  set: (v: boolean) => { form.status = v ? 1 : 0 },
})

const columns = [
  { title: '序号', key: 'index', width: 64, fixed: 'left' as const },
  { title: '用户名', dataIndex: 'userName', key: 'userName', width: 120 },
  { title: '昵称', dataIndex: 'nickName', key: 'nickName', width: 120 },
  { title: '邮箱', dataIndex: 'email', key: 'email', width: 180 },
  { title: '手机号', dataIndex: 'phone', key: 'phone', width: 140 },
  { title: '状态', key: 'status', width: 100 },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 180 },
  { title: '创建人', dataIndex: 'createBy', key: 'createBy', width: 120 },
  { title: '修改时间', dataIndex: 'updateTime', key: 'updateTime', width: 180 },
  { title: '修改人', dataIndex: 'updateBy', key: 'updateBy', width: 120 },
  { title: '操作', key: 'action', width: 200, fixed: 'right' as const },
]

const roleModalVisible = ref(false)
const roleLoading = ref(false)
const roleSubmitLoading = ref(false)
const roleUserId = ref<string | null>(null)
const allRoles = ref<RoleVo[]>([])
const checkedRoleIds = ref<string[]>([])
/** 已分配角色（跨客户端全量）；transfer 只展示当前客户端的子集 */
const assignedRoleIds = ref<string[]>([])
const roleClientId = ref<string | undefined>(undefined)
const clientOptions = ref<ClientOption[]>([])

const transferRoles = computed<TransferItem[]>(() =>
  allRoles.value.map((role) => ({
    key: role.id,
    title: role.roleName,
    description: role.roleCode,
  }))
)

function filterTransferOption(inputValue: string, item: TransferItem) {
  return (item.title as string).toLowerCase().includes(inputValue.toLowerCase())
}

async function fetchData() {
  loading.value = true
  try {
    const params: Record<string, any> = {
      page: pagination.current,
      pageSize: pagination.pageSize,
      userName: searchForm.userName || undefined,
      email: searchForm.email || undefined,
      phone: searchForm.phone || undefined,
      createBy: searchForm.createBy || undefined,
    }
    // 时间范围拆分为 createTimeStart/createTimeEnd → UserQuery.createTimeStart/End，筛选 auth_sys_user.create_time 区间（前者 gte，后者 lte）
    if (searchForm.createTimeRange && searchForm.createTimeRange.length === 2) {
      if (searchForm.createTimeRange[0]) {
        params.createTimeStart = searchForm.createTimeRange[0]
      }
      if (searchForm.createTimeRange[1]) {
        params.createTimeEnd = searchForm.createTimeRange[1]
      }
    }
    const res = await userApi.list(params)
    data.value = res.items
    pagination.total = res.total
  } finally {
    loading.value = false
  }
}

function onCreateTimeRangeChange(value: [string | null, string | null]) {
  searchForm.createTimeRange = value
}

function resetSearch() {
  searchForm.userName = ''
  searchForm.email = ''
  searchForm.phone = ''
  searchForm.createBy = ''
  searchForm.createTimeRange = [null, null]
  pagination.current = 1
  fetchData()
}

function onTableChange(pag: any) {
  pagination.current = pag.current
  pagination.pageSize = pag.pageSize
  fetchData()
}

async function onStatusChange(record: UserVo, checked: boolean) {
  statusLoadingMap[record.id] = true
  try {
    await userApi.updateStatus(record.id, checked ? 1 : 0)
    message.success(checked ? '已启用' : '已禁用')
    fetchData()
  } finally {
    statusLoadingMap[record.id] = false
  }
}

function openModal(record?: UserVo) {
  editId.value = record?.id || null
  if (record) {
    form.userName = record.userName
    form.nickName = record.nickName || ''
    form.email = record.email || ''
    form.phone = record.phone || ''
    form.status = record.status
    form.passWord = ''
  } else {
    form.userName = ''
    form.nickName = ''
    form.email = ''
    form.phone = ''
    form.status = 1
    form.passWord = ''
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
      await userApi.update(editId.value, { ...form })
      message.success('更新成功')
    } else {
      await userApi.create({ ...form })
      message.success('新增成功')
    }
    modalVisible.value = false
    fetchData()
  } finally {
    submitLoading.value = false
  }
}

async function doDelete(id: string) {
  try {
    await userApi.delete(id)
    message.success('删除成功')
    fetchData()
  } catch {
    message.error('删除失败')
  }
}

async function openRoleModal(record: UserVo) {
  roleUserId.value = record.id
  roleModalVisible.value = true
  roleLoading.value = true
  checkedRoleIds.value = []
  assignedRoleIds.value = []
  try {
    const [roleIds, clients] = await Promise.all([
      userApi.getRoleIds(record.id),
      clientApi.options(),
    ])
    clientOptions.value = clients || []
    assignedRoleIds.value = roleIds || []
    if (!roleClientId.value || !clientOptions.value.some((c) => c.id === roleClientId.value)) {
      roleClientId.value = clientOptions.value[0]?.id
    }
    await loadRoleOptionsForClient()
  } finally {
    roleLoading.value = false
  }
}

/** 按当前客户端加载角色，并回显该客户端下已分配的角色 */
async function loadRoleOptionsForClient() {
  if (!roleClientId.value) {
    allRoles.value = []
    checkedRoleIds.value = []
    return
  }
  allRoles.value = (await roleApi.listAll(roleClientId.value)) || []
  const ids = new Set(allRoles.value.map((r) => r.id))
  checkedRoleIds.value = assignedRoleIds.value.filter((id) => ids.has(id))
}

/** 把当前客户端的勾选结果合并回全量（未勾选的本客户端角色会被移除） */
function mergeCurrentClientSelection() {
  const currentIds = new Set(allRoles.value.map((r) => r.id))
  const others = assignedRoleIds.value.filter((id) => !currentIds.has(id))
  assignedRoleIds.value = Array.from(new Set([...others, ...checkedRoleIds.value]))
}

async function onRoleClientChange() {
  mergeCurrentClientSelection()
  roleLoading.value = true
  try {
    await loadRoleOptionsForClient()
  } finally {
    roleLoading.value = false
  }
}

async function onAssignRoles() {
  if (roleUserId.value === null) return
  mergeCurrentClientSelection()
  roleSubmitLoading.value = true
  try {
    await userApi.assignRoles(roleUserId.value, assignedRoleIds.value)
    message.success('角色分配成功')
    roleModalVisible.value = false
  } finally {
    roleSubmitLoading.value = false
  }
}

// 导出功能：支持按时间返回、勾选导出
async function doExport() {
  const selectedKeys = rowSelection.selectedRowKeys
  const params: Record<string, any> = {
    userName: searchForm.userName || undefined,
    email: searchForm.email || undefined,
    phone: searchForm.phone || undefined,
    createBy: searchForm.createBy || undefined,
  }
  if (searchForm.createTimeRange && searchForm.createTimeRange.length === 2) {
    if (searchForm.createTimeRange[0]) {
      params.createTimeStart = searchForm.createTimeRange[0]
    }
    if (searchForm.createTimeRange[1]) {
      params.createTimeEnd = searchForm.createTimeRange[1]
    }
  }
  // 如果有勾选行，只导出勾选的行
  if (selectedKeys.length > 0) {
    params.ids = selectedKeys.join(',')
  }
  exporting.value = true
  try {
    const { mode } = await exportList({
      taskType: 'user',
      query: params,
      syncUrl: '/api/system/users/export',
      filename: `users-export-${Date.now()}.xlsx`,
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
