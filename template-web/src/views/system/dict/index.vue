<template>
  <a-card title="字典管理" :bordered="false">
    <a-tabs v-model:activeKey="activeTab">
      <a-tab-pane key="type" tab="字典类型">
        <div style="margin-bottom: 16px; display: flex; justify-content: space-between; align-items: center">
          <a-space>
            <a-input v-model:value="typeSearch.dictName" placeholder="字典名称" allow-clear style="width: 160px" @pressEnter="fetchTypes" />
            <a-input v-model:value="typeSearch.dictType" placeholder="字典类型" allow-clear style="width: 160px" @pressEnter="fetchTypes" />
            <a-button type="primary" @click="fetchTypes">查询</a-button>
            <a-button @click="resetTypeSearch">重置</a-button>
          </a-space>
          <a-button type="primary" @click="openTypeModal()">新增类型</a-button>
        </div>
        <a-table :columns="typeColumns" :data-source="typeData" :loading="typeLoading" :pagination="typePaginationConfig"
          @change="onTypeTableChange" row-key="id" size="middle">
          <template #bodyCell="{ column, record }">
            <template v-if="column.key === 'status'">
              <a-switch :checked="record.status === 1" @change="(checked: boolean) => toggleTypeStatus(record, checked)" checked-children="启" un-checked-children="禁" />
            </template>
            <template v-if="column.key === 'remark'">
              <span>{{ record.remark || '-' }}</span>
            </template>
            <template v-if="column.key === 'action'">
              <a-space>
                <a @click.prevent="selectType(record)">管理字典项</a>
                <a @click.prevent="openTypeModal(record)">编辑</a>
                <a-popconfirm title="确定删除?" @confirm="doDeleteType(record.id)">
                  <a style="color: red">删除</a>
                </a-popconfirm>
              </a-space>
            </template>
          </template>
        </a-table>
      </a-tab-pane>
      <a-tab-pane key="item" tab="字典项" v-if="selectedType">
        <div style="margin-bottom: 16px; display: flex; justify-content: space-between; align-items: center">
          <a-alert :message="`当前字典: ${selectedType.dictName} (${selectedType.dictType})`" type="info" closable />
          <a-button type="primary" @click="openItemModal()">新增字典项</a-button>
        </div>
        <a-table :columns="itemColumns" :data-source="itemData" :loading="itemLoading" :pagination="false" row-key="id" size="middle">
          <template #bodyCell="{ column, record }">
            <template v-if="column.key === 'status'">
              <a-tag :color="record.status === 1 ? 'green' : 'red'">
                {{ record.status === 1 ? '启用' : '禁用' }}
              </a-tag>
            </template>
            <template v-if="column.key === 'action'">
              <a-space>
                <a @click.prevent="openItemModal(record)">编辑</a>
                <a-popconfirm title="确定删除?" @confirm="doDeleteItem(record.id)">
                  <a style="color: red">删除</a>
                </a-popconfirm>
              </a-space>
            </template>
          </template>
        </a-table>
      </a-tab-pane>
    </a-tabs>
  </a-card>

  <a-modal v-model:open="typeModalVisible" :title="editTypeId ? '编辑类型' : '新增类型'" :confirm-loading="typeSubmitting" @ok="onTypeSubmit">
    <a-form ref="typeFormRef" :model="typeForm" :rules="typeFormRules" :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }">
      <a-form-item label="字典名称" name="dictName"><a-input v-model:value="typeForm.dictName" placeholder="请输入字典名称" /></a-form-item>
      <a-form-item label="字典类型" name="dictType"><a-input v-model:value="typeForm.dictType" :disabled="!!editTypeId" placeholder="请输入字典类型" /></a-form-item>
      <a-form-item label="状态"><a-switch v-model:checked="typeStatusChecked" /></a-form-item>
      <a-form-item label="备注"><a-textarea v-model:value="typeForm.remark" placeholder="请输入备注" /></a-form-item>
    </a-form>
  </a-modal>

  <a-modal v-model:open="itemModalVisible" :title="editItemId ? '编辑字典项' : '新增字典项'" :confirm-loading="itemSubmitting" @ok="onItemSubmit">
    <a-form ref="itemFormRef" :model="itemForm" :rules="itemFormRules" :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }">
      <a-form-item label="标签" name="dictLabel"><a-input v-model:value="itemForm.dictLabel" placeholder="请输入标签" /></a-form-item>
      <a-form-item label="键值" name="dictValue"><a-input v-model:value="itemForm.dictValue" placeholder="请输入键值" /></a-form-item>
      <a-form-item label="排序"><a-input-number v-model:value="itemForm.sortOrder" :min="0" style="width: 100%" /></a-form-item>
      <a-form-item label="状态"><a-switch v-model:checked="itemStatusChecked" /></a-form-item>
      <a-form-item label="样式类名"><a-input v-model:value="itemForm.cssClass" placeholder="请输入样式类名" /></a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup lang="ts">
import { reactive, ref, computed } from 'vue'
import { message } from 'ant-design-vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import { dictTypeApi, dictItemApi, type DictTypeVo, type DictItemVo } from '@/api/dict'

const activeTab = ref('type')
const selectedType = ref<DictTypeVo | null>(null)

const typeLoading = ref(false)
const typeData = ref<DictTypeVo[]>([])
const typeModalVisible = ref(false)
const typeSubmitting = ref(false)
const editTypeId = ref<string | null>(null)
const typePagination = reactive({ current: 1, pageSize: 10, total: 0 })

const typeSearch = reactive({ dictName: '', dictType: '' })

const itemLoading = ref(false)
const itemData = ref<DictItemVo[]>([])
const itemModalVisible = ref(false)
const itemSubmitting = ref(false)
const editItemId = ref<string | null>(null)

const typeForm = reactive({ dictName: '', dictType: '', status: 1, remark: '' })
const itemForm = reactive({ dictTypeId: '' as string, dictLabel: '', dictValue: '', sortOrder: 0, status: 1, cssClass: '' })

const typeFormRef = ref<FormInstance>()
const itemFormRef = ref<FormInstance>()

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
  pageSizeOptions: ['10', '20', '50', '100'],
  showTotal: (total: number) => `共 ${total} 条`,
}))

const typeColumns = [
  { title: 'ID', dataIndex: 'id', key: 'id', width: 80 },
  { title: '名称', dataIndex: 'dictName', key: 'dictName', width: 150 },
  { title: '类型', dataIndex: 'dictType', key: 'dictType', width: 150 },
  { title: '状态', key: 'status', width: 100 },
  { title: '备注', key: 'remark', ellipsis: true },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 180 },
  { title: '操作', key: 'action', width: 220 },
]

const itemColumns = [
  { title: 'ID', dataIndex: 'id', key: 'id', width: 80 },
  { title: '标签', dataIndex: 'dictLabel', key: 'dictLabel', width: 150 },
  { title: '键值', dataIndex: 'dictValue', key: 'dictValue', width: 150 },
  { title: '排序', dataIndex: 'sortOrder', key: 'sortOrder', width: 80 },
  { title: '状态', key: 'status', width: 80 },
  { title: '操作', key: 'action', width: 150 },
]

async function fetchTypes() {
  typeLoading.value = true
  try {
    const res = await dictTypeApi.list({
      page: typePagination.current,
      pageSize: typePagination.pageSize,
      dictName: typeSearch.dictName || undefined,
      dictType: typeSearch.dictType || undefined,
    })
    typeData.value = res.items
    typePagination.total = res.total
  } catch (e: any) {
    message.error(e?.message || '查询字典类型失败')
  } finally { typeLoading.value = false }
}

function resetTypeSearch() {
  typeSearch.dictName = ''
  typeSearch.dictType = ''
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
  activeTab.value = 'item'
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

fetchTypes()
</script>
