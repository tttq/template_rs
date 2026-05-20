<template>
  <div class="user-picker">
    <a-input
      :value="displayText"
      :placeholder="placeholder"
      :disabled="disabled"
      readonly
      @click="openModal"
    >
      <template #suffix>
        <UserOutlined
          class="user-picker-icon"
          :class="{ disabled }"
          @click="!disabled && openModal()"
        />
      </template>
    </a-input>

    <a-modal
      v-model:open="modalVisible"
      title="选择用户"
      width="900px"
      :footer="null"
      :destroy-on-close="true"
    >
      <div class="user-picker-modal">
        <a-row :gutter="16" class="mb-16">
          <a-col :span="6">
            <a-input
              v-model:value="searchForm.userName"
              placeholder="用户名"
              allow-clear
              @pressEnter="fetchUsers"
            />
          </a-col>
          <a-col :span="6">
            <a-input
              v-model:value="searchForm.email"
              placeholder="邮箱"
              allow-clear
              @pressEnter="fetchUsers"
            />
          </a-col>
          <a-col :span="6">
            <a-input
              v-model:value="searchForm.phone"
              placeholder="手机号"
              allow-clear
              @pressEnter="fetchUsers"
            />
          </a-col>
          <a-col :span="6">
            <a-space>
              <a-button type="primary" @click="fetchUsers">查询</a-button>
              <a-button @click="resetSearch">重置</a-button>
            </a-space>
          </a-col>
        </a-row>

        <a-table
          :columns="columns"
          :data-source="userData"
          :loading="loading"
          :pagination="paginationConfig"
          row-key="id"
          :row-class-name="getRowClassName"
          :custom-row="customRow"
          size="middle"
          :scroll="{ y: 400 }"
          @change="onTableChange"
        >
          <template #bodyCell="{ column, record }">
            <template v-if="column.key === 'status'">
              <a-tag :color="record.status === 1 ? 'green' : 'red'">
                {{ record.status === 1 ? '启用' : '禁用' }}
              </a-tag>
            </template>
          </template>
        </a-table>
      </div>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref, computed, watch } from 'vue'
import { UserOutlined } from '@ant-design/icons-vue'
import { userApi, type UserVo } from '@/api/user'

export type UserPickerDisplayField = 'userName' | 'nickName' | 'email' | 'phone' | 'id'

const props = withDefaults(defineProps<{
  value?: any
  displayField?: UserPickerDisplayField
  placeholder?: string
  disabled?: boolean
}>(), {
  displayField: 'userName',
  placeholder: '请选择用户',
  disabled: false,
})

const emit = defineEmits<{
  'update:value': [value: any]
  'change': [user: UserVo | null]
}>()

const modalVisible = ref(false)
const loading = ref(false)
const userData = ref<UserVo[]>([])
const selectedUser = ref<UserVo | null>(null)

const searchForm = reactive({
  userName: '',
  email: '',
  phone: '',
})

const pagination = reactive({
  current: 1,
  pageSize: 10,
  total: 0,
})

const paginationConfig = computed(() => ({
  current: pagination.current,
  pageSize: pagination.pageSize,
  total: pagination.total,
  showSizeChanger: true,
  showQuickJumper: true,
  pageSizeOptions: ['10', '20', '50'],
  showTotal: (total: number) => `共 ${total} 条`,
}))

const columns = [
  { title: '用户名', dataIndex: 'userName', key: 'userName', width: 120 },
  { title: '昵称', dataIndex: 'nickName', key: 'nickName', width: 120 },
  { title: '邮箱', dataIndex: 'email', key: 'email', width: 180 },
  { title: '手机号', dataIndex: 'phone', key: 'phone', width: 140 },
  { title: '状态', key: 'status', width: 80 },
]

const displayText = computed(() => {
  if (!selectedUser.value) return ''
  const field = props.displayField
  if (field === 'id') {
    return selectedUser.value.userName || ''
  }
  return (selectedUser.value as any)[field] || ''
})

function getRowClassName(record: UserVo) {
  return selectedUser.value?.id === record.id ? 'user-picker-selected-row' : ''
}

function customRow(record: UserVo) {
  return {
    onClick: () => {
      selectUser(record)
    },
    style: { cursor: 'pointer' },
  }
}

function selectUser(user: UserVo) {
  selectedUser.value = user
  const field = props.displayField
  const emitValue = field === 'id' ? user.id : (user as any)[field]
  emit('update:value', emitValue)
  emit('change', user)
  modalVisible.value = false
}

async function fetchUsers() {
  loading.value = true
  try {
    const res = await userApi.list({
      page: pagination.current,
      pageSize: pagination.pageSize,
      userName: searchForm.userName || undefined,
      email: searchForm.email || undefined,
      phone: searchForm.phone || undefined,
    })
    userData.value = res.items
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
  fetchUsers()
}

function onTableChange(pag: any) {
  pagination.current = pag.current
  pagination.pageSize = pag.pageSize
  fetchUsers()
}

function openModal() {
  if (props.disabled) return
  modalVisible.value = true
  fetchUsers()
}

async function resolveUserById(userId: string) {
  try {
    const user = await userApi.getById(userId)
    selectedUser.value = user
  } catch {
    selectedUser.value = null
  }
}

watch(
  () => props.value,
  async (val) => {
    if (!val) {
      selectedUser.value = null
      return
    }
    if (selectedUser.value) {
      const currentFieldValue = props.displayField === 'id'
        ? selectedUser.value.id
        : (selectedUser.value as any)[props.displayField]
      if (currentFieldValue === val) return
    }
    if (props.displayField === 'id') {
      await resolveUserById(val)
    } else if (props.displayField === 'email') {
      selectedUser.value = { email: val } as UserVo
    } else {
      const res = await userApi.list({ page: 1, pageSize: 1, [props.displayField]: val })
      if (res.items.length > 0) {
        selectedUser.value = res.items[0]
      } else {
        selectedUser.value = null
      }
    }
  },
  { immediate: true }
)

defineExpose({ selectedUser, openModal })
</script>

<style scoped>
.user-picker-icon {
  cursor: pointer;
  color: rgba(0, 0, 0, 0.25);
  transition: color 0.3s;
}

.user-picker-icon:hover {
  color: #1890ff;
}

.user-picker-icon.disabled {
  cursor: not-allowed;
  color: rgba(0, 0, 0, 0.15);
}

:deep(.user-picker-selected-row) {
  background-color: #e6f7ff !important;
}

:deep(.user-picker-selected-row:hover > td) {
  background-color: #bae7ff !important;
}
</style>
