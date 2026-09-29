<template>
  <div>
    <a-card :bordered="false">
      <a-form
        layout="inline"
        :model="searchForm"
        style="margin-bottom: 16px; row-gap: 12px"
        @finish="fetchData"
      >
        <a-form-item label="客户端名称">
          <a-input
            v-model:value="searchForm.clientName"
            placeholder="请输入客户端名称"
            allow-clear
            style="width: 160px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item label="客户端标识">
          <a-input
            v-model:value="searchForm.clientCode"
            placeholder="如 web-admin"
            allow-clear
            style="width: 160px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item label="类型">
          <a-select
            v-model:value="searchForm.clientType"
            placeholder="全部"
            allow-clear
            style="width: 140px"
          >
            <a-select-option
              v-for="item in CLIENT_TYPES"
              :key="item.value"
              :value="item.value"
            >
              {{ item.label }}
            </a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="状态">
          <a-select
            v-model:value="searchForm.status"
            placeholder="全部"
            allow-clear
            style="width: 120px"
          >
            <a-select-option :value="1">
              启用
            </a-select-option>
            <a-select-option :value="0">
              禁用
            </a-select-option>
          </a-select>
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
            v-permission="'client:add'"
            type="primary"
            @click="openModal()"
          >
            新增客户端
          </a-button>
          <a-button
            v-permission="'client:export'"
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
        row-key="id"
        :pagination="false"
        :row-selection="rowSelection"
        :scroll="{ x: 1500, y: fitY }"
        size="middle"
      >
        <template #bodyCell="{ column, record, index }">
          <template v-if="column.key === 'index'">
            {{ index + 1 }}
          </template>
          <template v-if="column.key === 'clientName'">
            <a-space>
              <a-avatar
                v-if="record.logo"
                :src="record.logo"
                shape="square"
                :size="22"
              />
              <span>{{ record.clientName }}</span>
            </a-space>
          </template>
          <template v-if="column.key === 'clientCode'">
            <a-tag color="blue">
              {{ record.clientCode }}
            </a-tag>
          </template>
          <template v-if="column.key === 'clientType'">
            {{ typeLabel(record.clientType) }}
          </template>
          <template v-if="column.key === 'clientSecret'">
            <a-space>
              <span class="secret-text">
                {{ secretVisible[record.id] ? record.clientSecret : maskSecret(record.clientSecret) }}
              </span>
              <a @click.prevent="toggleSecret(record.id)">
                {{ secretVisible[record.id] ? '隐藏' : '查看' }}
              </a>
              <a @click.prevent="copySecret(record.clientSecret)">复制</a>
            </a-space>
          </template>
          <template v-if="column.key === 'status'">
            <a-switch
              :checked="record.status === 1"
              checked-children="启用"
              un-checked-children="禁用"
              size="small"
              :disabled="!canEdit"
              @change="(checked: boolean) => toggleStatus(record, checked)"
            />
          </template>
          <template v-if="column.key === 'action'">
            <a-space>
              <a
                v-permission="'client:edit'"
                @click.prevent="openModal(record)"
              >编辑</a>
              <a-divider
                v-permission="'client:edit'"
                type="vertical"
              />
              <a
                v-permission="'client:list'"
                @click.prevent="gotoClientMenus(record)"
              >菜单权限</a>
              <a-divider
                v-permission="'client:delete'"
                type="vertical"
              />
              <a-popconfirm
                title="删除客户端会同时回收该客户端的菜单与角色，确定删除?"
                @confirm="doDelete(record.id)"
              >
                <a
                  v-permission="'client:delete'"
                  style="color: red"
                >删除</a>
              </a-popconfirm>
            </a-space>
          </template>
        </template>
      </a-table>
    </a-card>
  </div>

  <a-modal
    v-model:open="modalVisible"
    :title="editId ? '编辑客户端' : '新增客户端'"
    width="620px"
    :confirm-loading="submitting"
    :mask-closable="false"
    @ok="onSubmit"
  >
    <a-form
      ref="formRef"
      :model="form"
      :rules="formRules"
      :label-col="{ span: 6 }"
      :wrapper-col="{ span: 16 }"
      style="margin-top: 16px"
    >
      <a-form-item
        label="客户端名称"
        name="clientName"
      >
        <a-input
          v-model:value="form.clientName"
          placeholder="如 管理后台 / 采购小程序"
        />
      </a-form-item>
      <a-form-item
        label="客户端标识"
        name="clientCode"
      >
        <a-input
          v-model:value="form.clientCode"
          placeholder="登录时传递，如 web-admin（创建后不可修改）"
          :disabled="!!editId"
        />
      </a-form-item>
      <a-form-item
        label="客户端密钥"
        name="clientSecret"
      >
        <a-input
          v-model:value="form.clientSecret"
          :placeholder="editId ? '留空表示不修改密钥' : '留空自动生成'"
        />
      </a-form-item>
      <a-form-item
        label="客户端类型"
        name="clientType"
      >
        <a-select v-model:value="form.clientType">
          <a-select-option
            v-for="item in CLIENT_TYPES"
            :key="item.value"
            :value="item.value"
          >
            {{ item.label }}
          </a-select-option>
        </a-select>
      </a-form-item>
      <a-form-item
        label="首页路径"
        name="homePath"
      >
        <a-input
          v-model:value="form.homePath"
          placeholder="登录后默认首页，如 /dashboard"
        />
      </a-form-item>
      <a-form-item
        label="排序"
        name="sortOrder"
      >
        <a-input-number
          v-model:value="form.sortOrder"
          :min="0"
          style="width: 100%"
        />
      </a-form-item>
      <a-form-item
        label="状态"
        name="status"
      >
        <a-radio-group v-model:value="form.status">
          <a-radio :value="1">
            启用
          </a-radio>
          <a-radio :value="0">
            禁用
          </a-radio>
        </a-radio-group>
      </a-form-item>
      <a-form-item
        label="备注"
        name="remark"
      >
        <a-textarea
          v-model:value="form.remark"
          :rows="3"
          placeholder="请输入备注"
        />
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup lang="ts">
import { computed, nextTick, reactive, ref } from 'vue'
import { useRouter } from 'vue-router'
import { message } from 'ant-design-vue'
import type { FormInstance } from 'ant-design-vue'
import { clientApi, type ClientVo } from '@/api/client'
import { useFitTableHeight } from '@/utils/useFitTableHeight'
import { exportList } from '@/api/export'
import { useUserStore } from '@/stores/user'

const CLIENT_TYPES = [
  { value: 'web', label: 'PC 端' },
  { value: 'miniapp', label: '小程序' },
  { value: 'app', label: '移动端' },
  { value: 'server', label: '服务端' },
]

const router = useRouter()
const userStore = useUserStore()
const canEdit = computed(() => userStore.hasPermission('client:edit'))

const loading = ref(false)
const submitting = ref(false)
const exporting = ref(false)
const data = ref<ClientVo[]>([])
const { fitRef, fitY } = useFitTableHeight()

const modalVisible = ref(false)
const editId = ref<string | null>(null)
const formRef = ref<FormInstance>()
const secretVisible = reactive<Record<string, boolean>>({})

const rowSelection = reactive({
  selectedRowKeys: [] as string[],
  onChange: (selectedRowKeys: string[]) => {
    rowSelection.selectedRowKeys = selectedRowKeys
  },
})

const searchForm = reactive({
  clientName: '',
  clientCode: '',
  clientType: undefined as string | undefined,
  status: undefined as number | undefined,
})

const form = reactive({
  clientName: '',
  clientCode: '',
  clientSecret: '',
  clientType: 'web',
  homePath: '',
  sortOrder: 0,
  status: 1,
  remark: '',
  version: 0,
})

const formRules = {
  clientName: [{ required: true, message: '请输入客户端名称', trigger: 'blur' }],
  clientCode: [
    { required: true, message: '请输入客户端标识', trigger: 'blur' },
    {
      pattern: /^[A-Za-z0-9_-]{1,64}$/,
      message: '只能包含字母、数字、- 和 _，最长 64 位',
      trigger: 'blur',
    },
  ],
  clientType: [{ required: true, message: '请选择客户端类型', trigger: 'change' }],
}

const columns = [
  { title: '序号', key: 'index', width: 64 },
  { title: '客户端名称', dataIndex: 'clientName', key: 'clientName', width: 180 },
  { title: '客户端标识', dataIndex: 'clientCode', key: 'clientCode', width: 160 },
  { title: '客户端密钥', key: 'clientSecret', width: 260 },
  { title: '类型', key: 'clientType', width: 100 },
  { title: '首页路径', dataIndex: 'homePath', key: 'homePath', width: 180 },
  { title: '排序', dataIndex: 'sortOrder', key: 'sortOrder', width: 80 },
  { title: '状态', key: 'status', width: 100 },
  { title: '备注', dataIndex: 'remark', key: 'remark', width: 180 },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 180 },
  { title: '操作', key: 'action', width: 260 },
]

function typeLabel(type: string) {
  return CLIENT_TYPES.find((item) => item.value === type)?.label || type
}

function maskSecret(secret?: string) {
  if (!secret) return '-'
  if (secret.length <= 8) return '********'
  return `${secret.slice(0, 4)}****${secret.slice(-4)}`
}

function toggleSecret(id: string) {
  secretVisible[id] = !secretVisible[id]
}

async function copySecret(secret: string) {
  try {
    await navigator.clipboard.writeText(secret)
    message.success('客户端密钥已复制')
  } catch {
    message.error('复制失败，请手动选择复制')
  }
}

async function fetchData() {
  loading.value = true
  try {
    const res = await clientApi.list({
      page: 1,
      pageSize: 200,
      clientName: searchForm.clientName || undefined,
      clientCode: searchForm.clientCode || undefined,
      clientType: searchForm.clientType || undefined,
      status: searchForm.status,
    })
    data.value = res.items
  } catch (e: any) {
    message.error(e?.message || '获取客户端列表失败')
  } finally {
    loading.value = false
  }
}

function resetSearch() {
  searchForm.clientName = ''
  searchForm.clientCode = ''
  searchForm.clientType = undefined
  searchForm.status = undefined
  fetchData()
}

function openModal(record?: ClientVo) {
  editId.value = record?.id || null
  form.clientName = record?.clientName || ''
  form.clientCode = record?.clientCode || ''
  form.clientSecret = ''
  form.clientType = record?.clientType || 'web'
  form.homePath = record?.homePath || ''
  form.sortOrder = record?.sortOrder ?? 0
  form.status = record?.status ?? 1
  form.remark = record?.remark || ''
  form.version = (record as any)?.version ?? 0
  modalVisible.value = true
  nextTick(() => formRef.value?.clearValidate())
}

async function onSubmit() {
  try {
    await formRef.value?.validate()
  } catch {
    return
  }
  submitting.value = true
  try {
    const payload: Record<string, any> = {
      clientName: form.clientName,
      clientType: form.clientType,
      homePath: form.homePath || undefined,
      sortOrder: form.sortOrder,
      status: form.status,
      remark: form.remark || undefined,
      clientSecret: form.clientSecret || undefined,
    }
    if (editId.value) {
      await clientApi.update(editId.value, payload)
      message.success('更新成功')
    } else {
      await clientApi.create({ ...payload, clientCode: form.clientCode })
      message.success('新增成功')
    }
    modalVisible.value = false
    fetchData()
  } catch (e: any) {
    message.error(e?.message || '操作失败')
  } finally {
    submitting.value = false
  }
}

async function toggleStatus(record: ClientVo, checked: boolean) {
  try {
    await clientApi.update(record.id, { status: checked ? 1 : 0 })
    record.status = checked ? 1 : 0
    message.success(checked ? '已启用' : '已禁用（该客户端在线会话已回收）')
  } catch (e: any) {
    message.error(e?.message || '状态更新失败')
    fetchData()
  }
}

async function doDelete(id: string) {
  try {
    await clientApi.delete(id)
    message.success('删除成功')
    fetchData()
  } catch (e: any) {
    message.error(e?.message || '删除失败')
  }
}

/** 菜单/功能权限按客户端维护：跳到菜单管理并带上客户端筛选 */
function gotoClientMenus(record: ClientVo) {
  router.push({ path: '/system/menu', query: { clientId: record.id } })
}

async function doExport() {
  const params: Record<string, any> = {
    clientName: searchForm.clientName || undefined,
    clientCode: searchForm.clientCode || undefined,
    clientType: searchForm.clientType || undefined,
    status: searchForm.status,
  }
  if (rowSelection.selectedRowKeys.length > 0) {
    params.ids = rowSelection.selectedRowKeys.join(',')
  }
  exporting.value = true
  try {
    const { mode } = await exportList({
      taskType: 'client',
      query: params,
      syncUrl: '/api/system/clients/export',
      filename: `clients-export-${Date.now()}.xlsx`,
    })
    message.success(mode === 'async' ? '导出成功，请自行到导出中心查看' : '导出成功')
  } catch (e: any) {
    message.error(e?.message || '导出失败')
  } finally {
    exporting.value = false
  }
}

fetchData()
</script>

<style scoped>
.secret-text {
  font-family: Consolas, Monaco, monospace;
  font-size: 12px;
}
</style>
