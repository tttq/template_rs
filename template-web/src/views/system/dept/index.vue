<template>
  <div>
    <a-card :bordered="false">
      <a-form
        layout="inline"
        :model="searchForm"
        style="margin-bottom: 16px; row-gap: 12px"
        @finish="fetchData"
      >
        <a-form-item label="部门名称">
          <a-input
            v-model:value="searchForm.deptName"
            placeholder="请输入部门名称"
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
            新增部门
          </a-button>
          <a-button
            v-permission="'dept:export'"
            :loading="exporting"
            @click="doExport"
          >
            导出 Excel
          </a-button>
        </a-space>
      </template>
      <a-table
        :columns="columns"
        :data-source="filteredData"
        :loading="loading"
        row-key="id"
        :pagination="false"
        size="middle"
        :scroll="{ x: 1664, y: fitY }"
        :default-expand-all-rows="false"
        :row-selection="rowSelection"
      >
        <template #bodyCell="{ column, record, index }">
          <template v-if="column.key === 'index'">
            {{ index + 1 }}
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
              <a @click.prevent="openChildModal(record)">新增子部门</a>
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
      :title="editId ? '编辑部门' : '新增部门'"
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
          label="上级部门"
          name="parentId"
        >
          <a-tree-select
            v-model:value="form.parentId"
            :tree-data="deptTreeData"
            :field-names="{ label: 'deptName', value: 'id', children: 'children' }"
            placeholder="请选择上级部门"
            allow-clear
            tree-default-expand-all
          />
        </a-form-item>
        <a-form-item
          label="部门名称"
          name="deptName"
        >
          <a-input
            v-model:value="form.deptName"
            placeholder="请输入部门名称"
          />
        </a-form-item>
        <a-form-item
          label="排序"
          name="deptSort"
        >
          <a-input-number
            v-model:value="form.deptSort"
            :min="0"
            style="width: 100%"
          />
        </a-form-item>
        <a-form-item
          label="负责人"
          name="leader"
        >
          <a-input
            v-model:value="form.leader"
            placeholder="请输入负责人"
          />
        </a-form-item>
        <a-form-item
          label="电话"
          name="phone"
        >
          <a-input
            v-model:value="form.phone"
            placeholder="请输入电话"
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
        <a-form-item label="状态">
          <a-switch
            v-model:checked="statusChecked"
            checked-children="启用"
            un-checked-children="禁用"
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
import { deptApi, type DeptVo } from '@/api/dept'
import { buildTree } from '@/utils/tree'
import { useFitTableHeight } from '@/utils/useFitTableHeight'
import { exportList } from '@/api/export'

const searchForm = reactive({
  // deptName：界面「部门名称」输入框，发送给后端 DeptQuery.deptName，筛选 auth_sys_dept.dept_name 模糊 contains
  deptName: '',
  // createBy：界面「创建人」输入框，发送给后端 DeptQuery.createBy，筛选 auth_sys_dept.create_by 模糊 contains
  createBy: '',
  // createTimeRange：界面「创建时间」范围，拆分发送给后端 DeptQuery.createTimeStart/createTimeEnd，筛选 auth_sys_dept.create_time 区间（>= 开始，<= 结束）
  createTimeRange: [null, null] as [string | null, string | null],
})

const loading = ref(false)
const data = ref<DeptVo[]>([])

const { fitRef, fitY } = useFitTableHeight()

const modalVisible = ref(false)
const editId = ref<string | null>(null)
const submitLoading = ref(false)
const formRef = ref<FormInstance>()
const statusLoadingMap = reactive<Record<string, boolean>>({})
const exporting = ref(false)

// 行选择（用于勾选导出）
const rowSelection = reactive({
  selectedRowKeys: [] as string[],
  onChange: (selectedRowKeys: string[]) => {
    rowSelection.selectedRowKeys = selectedRowKeys
  },
})

const form = reactive({
  parentId: '0',
  deptName: '',
  deptSort: 0,
  status: 1,
  leader: '',
  phone: '',
  email: '',
})

const formRules: Record<string, Rule[]> = {
  deptName: [{ required: true, message: '请输入部门名称', trigger: 'blur' }],
  email: [{ type: 'email', message: '请输入正确的邮箱格式', trigger: 'blur' }],
  phone: [{ pattern: /^1[3-9]\d{9}$/, message: '请输入正确的手机号', trigger: 'blur' }],
}

const statusChecked = computed({
  get: () => form.status === 1,
  set: (v: boolean) => { form.status = v ? 1 : 0 },
})

const deptTreeData = computed(() => {
  const root: DeptVo = { id: '0', parentId: '0', deptName: '顶级部门', deptSort: 0, status: 1, createTime: '', updateTime: '', children: data.value }
  return [root]
})

function filterTree(nodes: DeptVo[], keyword: string): DeptVo[] {
  if (!keyword) return nodes
  const result: DeptVo[] = []
  for (const node of nodes) {
    const children = node.children ? filterTree(node.children, keyword) : []
    if (node.deptName.toLowerCase().includes(keyword.toLowerCase()) || children.length > 0) {
      result.push({ ...node, children: children.length > 0 ? children : node.children })
    }
  }
  return result
}

const filteredData = computed(() => {
  return filterTree(data.value, searchForm.deptName)
})

const columns = [
  { title: '序号', key: 'index', width: 64 },
  { title: '名称', dataIndex: 'deptName', key: 'deptName', width: 200 },
  { title: '排序', dataIndex: 'deptSort', key: 'deptSort', width: 80 },
  { title: '负责人', dataIndex: 'leader', key: 'leader', width: 120 },
  { title: '电话', dataIndex: 'phone', key: 'phone', width: 140 },
  { title: '邮箱', dataIndex: 'email', key: 'email', width: 140 },
  { title: '状态', key: 'status', width: 100 },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 180 },
  { title: '创建人', dataIndex: 'createBy', key: 'createBy', width: 120 },
  { title: '修改时间', dataIndex: 'updateTime', key: 'updateTime', width: 180 },
  { title: '修改人', dataIndex: 'updateBy', key: 'updateBy', width: 120 },
  { title: '操作', key: 'action', width: 220 },
]

async function fetchData() {
  loading.value = true
  try {
    const params: Record<string, any> = {
      page: 1,
      pageSize: 1000,
      deptName: searchForm.deptName || undefined,
      createBy: searchForm.createBy || undefined,
    }
    // 时间范围拆分为 createTimeStart/createTimeEnd → DeptQuery.createTimeStart/End，筛选 auth_sys_dept.create_time 区间（前者 gte，后者 lte）
    if (searchForm.createTimeRange && searchForm.createTimeRange.length === 2) {
      if (searchForm.createTimeRange[0]) {
        params.createTimeStart = searchForm.createTimeRange[0]
      }
      if (searchForm.createTimeRange[1]) {
        params.createTimeEnd = searchForm.createTimeRange[1]
      }
    }
    const res = await deptApi.list(params)
    data.value = Array.isArray(res) ? buildTree(res) : buildTree(res.items)
  } catch {
    message.error('获取部门数据失败')
  } finally {
    loading.value = false
  }
}

function onCreateTimeRangeChange(value: [string | null, string | null]) {
  searchForm.createTimeRange = value
}

function resetSearch() {
  searchForm.deptName = ''
  searchForm.createBy = ''
  searchForm.createTimeRange = [null, null]
}

function openModal(record?: DeptVo) {
  editId.value = record?.id || null
  if (record) {
    form.parentId = record.parentId
    form.deptName = record.deptName
    form.deptSort = record.deptSort
    form.status = record.status
    form.leader = record.leader || ''
    form.phone = record.phone || ''
    form.email = record.email || ''
  } else {
    form.parentId = '0'
    form.deptName = ''
    form.deptSort = 0
    form.status = 1
    form.leader = ''
    form.phone = ''
    form.email = ''
  }
  modalVisible.value = true
  setTimeout(() => formRef.value?.clearValidate(), 0)
}

function openChildModal(record: DeptVo) {
  editId.value = null
  form.parentId = record.id
  form.deptName = ''
  form.deptSort = 0
  form.status = 1
  form.leader = ''
  form.phone = ''
  form.email = ''
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
      await deptApi.update(editId.value, { ...form, version: 1 })
      message.success('更新成功')
    } else {
      await deptApi.create({ ...form })
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

async function onStatusChange(record: DeptVo, checked: boolean) {
  statusLoadingMap[record.id] = true
  try {
    await deptApi.update(record.id, { status: checked ? 1 : 0 })
    message.success(checked ? '已启用' : '已禁用')
    fetchData()
  } catch {
    message.error('状态更新失败')
  } finally {
    statusLoadingMap[record.id] = false
  }
}

async function doDelete(id: string) {
  try {
    await deptApi.delete(id)
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
    deptName: searchForm.deptName || undefined,
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
      taskType: 'dept',
      query: params,
      syncUrl: '/api/system/depts/export',
      filename: `depts-export-${Date.now()}.xlsx`,
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
