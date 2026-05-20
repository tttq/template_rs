<template>
  <div>
    <a-card :bordered="false">
      <a-row :gutter="16" class="mb-4">
        <a-col :span="6">
          <a-input v-model:value="searchForm.userName" placeholder="用户名" allow-clear @pressEnter="fetchData" />
        </a-col>
        <a-col :span="6">
          <a-input v-model:value="searchForm.email" placeholder="邮箱" allow-clear @pressEnter="fetchData" />
        </a-col>
        <a-col :span="6">
          <a-input v-model:value="searchForm.phone" placeholder="手机号" allow-clear @pressEnter="fetchData" />
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
      <template #title>用户管理</template>
      <template #extra>
        <a-button type="primary" @click="openModal()">新增用户</a-button>
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
              <a @click.prevent="openRoleModal(record)">分配角色</a>
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
      :title="editId ? '编辑用户' : '新增用户'"
      :confirm-loading="submitLoading"
      width="600px"
      @ok="onSubmit"
    >
      <a-form ref="formRef" :model="form" :rules="formRules" :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }">
        <a-form-item label="用户名" name="userName">
          <a-input v-model:value="form.userName" :disabled="!!editId" placeholder="请输入用户名" />
        </a-form-item>
        <a-form-item label="密码" :name="editId ? undefined : 'passWord'">
          <a-input-password v-model:value="form.passWord" :placeholder="editId ? '留空不修改' : '请输入密码'" />
        </a-form-item>
        <a-form-item label="昵称" name="nickName">
          <a-input v-model:value="form.nickName" placeholder="请输入昵称" />
        </a-form-item>
        <a-form-item label="邮箱" name="email">
          <a-input v-model:value="form.email" placeholder="请输入邮箱" />
        </a-form-item>
        <a-form-item label="手机号" name="phone">
          <a-input v-model:value="form.phone" placeholder="请输入手机号" />
        </a-form-item>
        <a-form-item label="状态">
          <a-switch v-model:checked="statusChecked" checked-children="启用" un-checked-children="禁用" />
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

const searchForm = reactive({
  userName: '',
  email: '',
  phone: '',
})

const loading = ref(false)
const data = ref<UserVo[]>([])
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
  { title: 'ID', dataIndex: 'id', key: 'id', width: 80 },
  { title: '用户名', dataIndex: 'userName', key: 'userName', width: 120 },
  { title: '昵称', dataIndex: 'nickName', key: 'nickName', width: 120 },
  { title: '邮箱', dataIndex: 'email', key: 'email', width: 180 },
  { title: '手机号', dataIndex: 'phone', key: 'phone', width: 140 },
  { title: '状态', key: 'status', width: 100 },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 180 },
  { title: '操作', key: 'action', width: 200, fixed: 'right' as const },
]

const roleModalVisible = ref(false)
const roleLoading = ref(false)
const roleSubmitLoading = ref(false)
const roleUserId = ref<string | null>(null)
const allRoles = ref<RoleVo[]>([])
const checkedRoleIds = ref<string[]>([])

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
    const res = await userApi.list({
      page: pagination.current,
      pageSize: pagination.pageSize,
      userName: searchForm.userName || undefined,
      email: searchForm.email || undefined,
      phone: searchForm.phone || undefined,
    })
    data.value = res.items
    pagination.total = res.total
  } finally {
    loading.value = false
  }
}

function resetSearch() {
  searchForm.userName = ''
  searchForm.email = ''
  searchForm.phone = ''
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
  try {
    const [roles, roleIds] = await Promise.all([
      roleApi.listAll(),
      userApi.getRoleIds(record.id),
    ])
    allRoles.value = roles
    checkedRoleIds.value = roleIds
  } finally {
    roleLoading.value = false
  }
}

async function onAssignRoles() {
  if (roleUserId.value === null) return
  roleSubmitLoading.value = true
  try {
    await userApi.assignRoles(roleUserId.value, checkedRoleIds.value)
    message.success('角色分配成功')
    roleModalVisible.value = false
  } finally {
    roleSubmitLoading.value = false
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
