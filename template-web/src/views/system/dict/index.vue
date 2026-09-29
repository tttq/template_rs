<template>
  <div>
      <a-card :bordered="false">
        <a-form
          layout="inline"
          :model="typeSearch"
          style="margin-bottom: 16px; row-gap: 12px"
          @finish="fetchTypes"
        >
            <a-form-item label="字典名称">
              <a-input
                v-model:value="typeSearch.dictName"
                placeholder="请输入字典名称"
                allow-clear
                style="width: 160px"
                @press-enter="fetchTypes"
              />
            </a-form-item>
            <a-form-item label="字典类型">
              <a-input
                v-model:value="typeSearch.dictType"
                placeholder="请输入字典类型"
                allow-clear
                style="width: 160px"
                @press-enter="fetchTypes"
              />
            </a-form-item>
            <a-form-item label="创建人">
              <a-input
                v-model:value="typeSearch.createBy"
                placeholder="请输入创建人"
                allow-clear
                style="width: 160px"
                @press-enter="fetchTypes"
              />
            </a-form-item>
            <a-form-item label="创建时间">
              <a-range-picker
                v-model:value="typeSearch.createTimeRange"
                :placeholder="['开始时间', '结束时间']"
                allow-clear
                style="width: 220px"
                @change="onTypeCreateTimeRangeChange"
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
                <a-button @click="resetTypeSearch">
                  重置
                </a-button>
              </a-space>
            </a-form-item>
          </a-form>
          </a-card>

          <a-card
            ref="typeFitRef"
            :bordered="false"
            class="mt-3"
          >
            <template #extra>
              <a-space>
                <a-button
                  type="primary"
                  @click="openTypeModal()"
                >
                  新增类型
                </a-button>
                <a-button
                  v-permission="'dict:export'"
                  :loading="typeExporting"
                  @click="doExportTypes"
                >
                  导出 Excel
                </a-button>
              </a-space>
            </template>
        <a-table
          :columns="typeColumns"
          :data-source="typeData"
          :loading="typeLoading"
          :pagination="typePaginationConfig"
          row-key="id"
          size="middle"
          :row-selection="typeRowSelection"
          :scroll="{ x: 1350, y: typeFitY }"
          @change="onTypeTableChange"
        >
          <template #bodyCell="{ column, record, index }">
            <template v-if="column.key === 'index'">
              {{ (typePagination.current - 1) * typePagination.pageSize + index + 1 }}
            </template>
            <template v-if="column.key === 'status'">
              <a-switch
                :checked="record.status === 1"
                checked-children="启"
                un-checked-children="禁"
                @change="(checked: boolean) => toggleTypeStatus(record, checked)"
              />
            </template>
            <template v-if="column.key === 'remark'">
              <span>{{ record.remark || '-' }}</span>
            </template>
            <template v-if="column.key === 'action'">
              <a-space>
                <a @click.prevent="selectType(record)">管理字典项</a>
                <a @click.prevent="openTypeModal(record)">编辑</a>
                <a-popconfirm
                  title="确定删除?"
                  @confirm="doDeleteType(record.id)"
                >
                  <a style="color: red">删除</a>
                </a-popconfirm>
              </a-space>
            </template>
          </template>
        </a-table>
        </a-card>

  </div>

  <!-- 管理字典项：弹窗内直接展示字典项表格 -->
  <a-modal
    v-model:open="itemPanelVisible"
    :title="`管理字典项 — ${selectedType?.dictName || ''} (${selectedType?.dictType || ''})`"
    width="820px"
    :footer="null"
    destroy-on-close
  >
    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px">
      <a-alert
        v-if="selectedType"
        :message="`当前字典: ${selectedType.dictName} (${selectedType.dictType})`"
        type="info"
        closable
        style="flex: 1; margin-right: 16px"
      />
      <a-button
        type="primary"
        @click="openItemModal()"
      >
        新增字典项
      </a-button>
    </div>
    <a-table
      :columns="itemColumns"
      :data-source="itemData"
      :loading="itemLoading"
      :pagination="false"
      row-key="id"
      size="middle"
      :scroll="{ x: 700, y: 420 }"
    >
      <template #bodyCell="{ column, record, index }">
        <template v-if="column.key === 'index'">
          {{ index + 1 }}
        </template>
        <template v-if="column.key === 'status'">
          <a-tag :color="record.status === 1 ? 'green' : 'red'">
            {{ record.status === 1 ? '启用' : '禁用' }}
          </a-tag>
        </template>
        <template v-if="column.key === 'action'">
          <a-space>
            <a @click.prevent="openItemModal(record)">编辑</a>
            <a-popconfirm
              title="确定删除?"
              @confirm="doDeleteItem(record.id)"
            >
              <a style="color: red">删除</a>
            </a-popconfirm>
          </a-space>
        </template>
      </template>
    </a-table>
  </a-modal>

  <a-modal
    v-model:open="typeModalVisible"
    :title="editTypeId ? '编辑类型' : '新增类型'"
    :confirm-loading="typeSubmitting"
    @ok="onTypeSubmit"
  >
    <a-form
      ref="typeFormRef"
      :model="typeForm"
      :rules="typeFormRules"
      :label-col="{ span: 6 }"
      :wrapper-col="{ span: 16 }"
    >
      <a-form-item
        label="字典名称"
        name="dictName"
      >
        <a-input
          v-model:value="typeForm.dictName"
          placeholder="请输入字典名称"
        />
      </a-form-item>
      <a-form-item
        label="字典类型"
        name="dictType"
      >
        <a-input
          v-model:value="typeForm.dictType"
          :disabled="!!editTypeId"
          placeholder="请输入字典类型"
        />
      </a-form-item>
      <a-form-item label="状态">
        <a-switch v-model:checked="typeStatusChecked" />
      </a-form-item>
      <a-form-item label="备注">
        <a-textarea
          v-model:value="typeForm.remark"
          placeholder="请输入备注"
        />
      </a-form-item>
    </a-form>
  </a-modal>

  <a-modal
    v-model:open="itemModalVisible"
    :title="editItemId ? '编辑字典项' : '新增字典项'"
    :confirm-loading="itemSubmitting"
    @ok="onItemSubmit"
  >
    <a-form
      ref="itemFormRef"
      :model="itemForm"
      :rules="itemFormRules"
      :label-col="{ span: 6 }"
      :wrapper-col="{ span: 16 }"
    >
      <a-form-item
        label="标签"
        name="dictLabel"
      >
        <a-input
          v-model:value="itemForm.dictLabel"
          placeholder="请输入标签"
        />
      </a-form-item>
      <a-form-item
        label="键值"
        name="dictValue"
      >
        <a-input
          v-model:value="itemForm.dictValue"
          placeholder="请输入键值"
        />
      </a-form-item>
      <a-form-item label="排序">
        <a-input-number
          v-model:value="itemForm.sortOrder"
          :min="0"
          style="width: 100%"
        />
      </a-form-item>
      <a-form-item label="状态">
        <a-switch v-model:checked="itemStatusChecked" />
      </a-form-item>
      <a-form-item label="样式类名">
        <a-input
          v-model:value="itemForm.cssClass"
          placeholder="请输入样式类名"
        />
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup lang="ts">
import { reactive, ref, computed } from 'vue'
import { message } from 'ant-design-vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import { dictTypeApi, dictItemApi, type DictTypeVo, type DictItemVo } from '@/api/dict'
import { useFitTableHeight } from '@/utils/useFitTableHeight'
import { exportList } from '@/api/export'

const selectedType = ref<DictTypeVo | null>(null)
const itemPanelVisible = ref(false)

// 主表格（字典类型）高度自适应
const { fitRef: typeFitRef, fitY: typeFitY } = useFitTableHeight()

const typeLoading = ref(false)
const typeData = ref<DictTypeVo[]>([])
const typeModalVisible = ref(false)
const typeSubmitting = ref(false)
const editTypeId = ref<string | null>(null)
const typePagination = reactive({ current: 1, pageSize: 20, total: 0 })

const itemLoading = ref(false)
const itemData = ref<DictItemVo[]>([])
const itemModalVisible = ref(false)
const itemSubmitting = ref(false)
const editItemId = ref<string | null>(null)

const typeForm = reactive({ dictName: '', dictType: '', status: 1, remark: '' })
const itemForm = reactive({ dictTypeId: '' as string, dictLabel: '', dictValue: '', sortOrder: 0, status: 1, cssClass: '' })

const typeFormRef = ref<FormInstance>()
const itemFormRef = ref<FormInstance>()
const typeExporting = ref(false)

// 行选择（用于勾选导出）
const typeRowSelection = reactive({
  selectedRowKeys: [] as string[],
  onChange: (selectedRowKeys: string[]) => {
    typeRowSelection.selectedRowKeys = selectedRowKeys
  },
})

const typeSearch = reactive({
  // dictName：界面「字典名称」输入框，发送给后端 DictTypeQuery.dictName，筛选 auth_sys_dict_type.dict_name 模糊 contains
  dictName: '',
  // dictType：界面「字典类型」输入框，发送给后端 DictTypeQuery.dictType，筛选 auth_sys_dict_type.dict_type 模糊 contains
  dictType: '',
  // createBy：界面「创建人」输入框，发送给后端 DictTypeQuery.createBy，筛选 auth_sys_dict_type.create_by 模糊 contains
  createBy: '',
  // createTimeRange：界面「创建时间」范围，拆分发送给后端 DictTypeQuery.createTimeStart/createTimeEnd，筛选 auth_sys_dict_type.create_time 区间（>= 开始，<= 结束）
  createTimeRange: [null, null] as [string | null, string | null],
})

const typeFormRules: Record<string, Rule[]> = {
  dictName: [{ required: true, message: '请输入字典名称', trigger: 'blur' }],
  dictType: [{ required: true, message: '请输入字典类型', trigger: 'blur' }],
}

const itemFormRules: Record<string, Rule[]> = {
  dictLabel: [{ required: true, message: '请输入标签', trigger: 'blur' }],
  dictValue: [{ required: true, message: '请输入键值', trigger: 'blur' }],
}

const typeStatusChecked = computed({
  get: () => typeForm.status === 1,
  set: (v) => { typeForm.status = v ? 1 : 0 },
})

const itemStatusChecked = computed({
  get: () => itemForm.status === 1,
  set: (v) => { itemForm.status = v ? 1 : 0 },
})

const typePaginationConfig = computed(() => ({
  current: typePagination.current,
  pageSize: typePagination.pageSize,
  total: typePagination.total,
  showSizeChanger: true,
  showQuickJumper: true,
  pageSizeOptions: ['20', '40', '60', '100'],
  showTotal: (total: number) => `共 ${total} 条`,
}))

const typeColumns = [
  { title: '序号', key: 'index', width: 64 },
  { title: '名称', dataIndex: 'dictName', key: 'dictName', width: 150 },
  { title: '类型', dataIndex: 'dictType', key: 'dictType', width: 150 },
  { title: '状态', key: 'status', width: 100 },
  { title: '备注', key: 'remark', ellipsis: true },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 180 },
  { title: '创建人', dataIndex: 'createBy', key: 'createBy', width: 120 },
  { title: '修改时间', dataIndex: 'updateTime', key: 'updateTime', width: 180 },
  { title: '修改人', dataIndex: 'updateBy', key: 'updateBy', width: 120 },
  { title: '操作', key: 'action', width: 220 },
]

const itemColumns = [
  { title: '序号', key: 'index', width: 64 },
  { title: '标签', dataIndex: 'dictLabel', key: 'dictLabel', width: 150 },
  { title: '键值', dataIndex: 'dictValue', key: 'dictValue', width: 150 },
  { title: '排序', dataIndex: 'sortOrder', key: 'sortOrder', width: 80 },
  { title: '状态', key: 'status', width: 80 },
  { title: '操作', key: 'action', width: 150 },
]

async function fetchTypes() {
  typeLoading.value = true
  try {
    const params: Record<string, any> = {
      page: typePagination.current,
      pageSize: typePagination.pageSize,
      dictName: typeSearch.dictName || undefined,
      dictType: typeSearch.dictType || undefined,
      createBy: typeSearch.createBy || undefined,
    }
    // 时间范围拆分为 createTimeStart/createTimeEnd → DictTypeQuery.createTimeStart/End，筛选 auth_sys_dict_type.create_time 区间（前者 gte，后者 lte）
    if (typeSearch.createTimeRange && typeSearch.createTimeRange.length === 2) {
      if (typeSearch.createTimeRange[0]) {
        params.createTimeStart = typeSearch.createTimeRange[0]
      }
      if (typeSearch.createTimeRange[1]) {
        params.createTimeEnd = typeSearch.createTimeRange[1]
      }
    }
    const res = await dictTypeApi.list(params)
    typeData.value = res.items
    typePagination.total = res.total
  } catch (e: any) {
    message.error(e?.message || '查询字典类型失败')
  } finally { typeLoading.value = false }
}

function onTypeCreateTimeRangeChange(value: [string | null, string | null]) {
  typeSearch.createTimeRange = value
}

function resetTypeSearch() {
  typeSearch.dictName = ''
  typeSearch.dictType = ''
  typeSearch.createBy = ''
  typeSearch.createTimeRange = [null, null]
  typePagination.current = 1
  fetchTypes()
}

function onTypeTableChange(pag: any) { typePagination.current = pag.current; typePagination.pageSize = pag.pageSize; fetchTypes() }

function openTypeModal(record?: DictTypeVo) {
  editTypeId.value = record?.id || null
  if (record) {
    typeForm.dictName = record.dictName; typeForm.dictType = record.dictType
    typeForm.status = record.status; typeForm.remark = record.remark || ''
  } else {
    typeForm.dictName = ''; typeForm.dictType = ''; typeForm.status = 1; typeForm.remark = ''
  }
  typeFormRef.value?.clearValidate()
  typeModalVisible.value = true
}

async function onTypeSubmit() {
  try {
    await typeFormRef.value?.validate()
  } catch { return }
  typeSubmitting.value = true
  try {
    if (editTypeId.value) {
      await dictTypeApi.update(editTypeId.value, { ...typeForm, version: 1 })
    } else {
      await dictTypeApi.create({ ...typeForm })
    }
    message.success('操作成功')
    typeModalVisible.value = false
    fetchTypes()
  } catch (e: any) {
    message.error(e?.message || '操作失败')
  } finally { typeSubmitting.value = false }
}

async function toggleTypeStatus(record: DictTypeVo, checked: boolean) {
  const newStatus = checked ? 1 : 0
  try {
    await dictTypeApi.update(record.id, { ...record, status: newStatus, version: 1 })
    record.status = newStatus
    message.success('状态更新成功')
  } catch (e: any) {
    message.error(e?.message || '状态更新失败')
  }
}

async function doDeleteType(id: string) {
  try {
    await dictTypeApi.delete(id)
    message.success('删除成功')
    fetchTypes()
  } catch (e: any) {
    message.error(e?.message || '删除失败')
  }
}

async function selectType(record: DictTypeVo) {
  selectedType.value = record
  itemPanelVisible.value = true
  await fetchItems()
}

async function fetchItems() {
  if (!selectedType.value) return
  itemLoading.value = true
  try {
    const res = await dictItemApi.getByTypeId(selectedType.value.id)
    itemData.value = res
  } catch (e: any) {
    message.error(e?.message || '查询字典项失败')
  } finally { itemLoading.value = false }
}

function openItemModal(record?: DictItemVo) {
  editItemId.value = record?.id || null
  if (record) {
    itemForm.dictTypeId = record.dictTypeId; itemForm.dictLabel = record.dictLabel
    itemForm.dictValue = record.dictValue; itemForm.sortOrder = record.sortOrder
    itemForm.status = record.status; itemForm.cssClass = record.cssClass || ''
  } else {
    itemForm.dictTypeId = selectedType.value?.id || ''
    itemForm.dictLabel = ''; itemForm.dictValue = ''; itemForm.sortOrder = 0
    itemForm.status = 1; itemForm.cssClass = ''
  }
  itemFormRef.value?.clearValidate()
  itemModalVisible.value = true
}

async function onItemSubmit() {
  try {
    await itemFormRef.value?.validate()
  } catch { return }
  itemSubmitting.value = true
  try {
    if (editItemId.value) {
      await dictItemApi.update(editItemId.value, { ...itemForm, version: 1 })
    } else {
      await dictItemApi.create({ ...itemForm })
    }
    message.success('操作成功')
    itemModalVisible.value = false
    fetchItems()
  } catch (e: any) {
    message.error(e?.message || '操作失败')
  } finally { itemSubmitting.value = false }
}

async function doDeleteItem(id: string) {
  try {
    await dictItemApi.delete(id)
    message.success('删除成功')
    fetchItems()
  } catch (e: any) {
    message.error(e?.message || '删除失败')
  }
}

// 导出字典类型
async function doExportTypes() {
  const selectedKeys = typeRowSelection.selectedRowKeys
  const params: Record<string, any> = {
    dictName: typeSearch.dictName || undefined,
    dictType: typeSearch.dictType || undefined,
    createBy: typeSearch.createBy || undefined,
  }
  if (typeSearch.createTimeRange && typeSearch.createTimeRange.length === 2) {
    if (typeSearch.createTimeRange[0]) {
      params.createTimeStart = typeSearch.createTimeRange[0]
    }
    if (typeSearch.createTimeRange[1]) {
      params.createTimeEnd = typeSearch.createTimeRange[1]
    }
  }
  if (selectedKeys.length > 0) {
    params.ids = selectedKeys.join(',')
  }
  typeExporting.value = true
  try {
    const { mode } = await exportList({
      taskType: 'dict',
      query: params,
      syncUrl: '/api/system/dicts/types/export',
      filename: `dict-types-export-${Date.now()}.xlsx`,
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
    typeExporting.value = false
  }
}

fetchTypes()
</script>
