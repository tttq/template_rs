<template>
  <a-card title="菜单管理" :bordered="false">
    <template #extra>
      <a-button type="primary" @click="openModal()">新增菜单</a-button>
    </template>
    <a-table
      :columns="columns"
      :data-source="data"
      :loading="loading"
      row-key="id"
      :pagination="false"
      default-expand-all-rows
      size="middle"
    >
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'menuName'">
          <span>{{ record.menuName }}</span>
        </template>
        <template v-if="column.key === 'icon'">
          <component :is="getIconComponent(record.icon)" v-if="record.icon" style="font-size: 16px" />
          <span v-else style="color: #d9d9d9">-</span>
        </template>
        <template v-if="column.key === 'menuType'">
          <a-tag v-if="record.menuType === 'dir'" color="blue">目录</a-tag>
          <a-tag v-else-if="record.menuType === 'menu'" color="green">菜单</a-tag>
          <a-tag v-else-if="record.menuType === 'button'" color="orange">按钮</a-tag>
        </template>
        <template v-if="column.key === 'status'">
          <a-tag :color="record.status === 1 ? 'green' : 'red'">
            {{ record.status === 1 ? '启用' : '禁用' }}
          </a-tag>
        </template>
        <template v-if="column.key === 'visible'">
          <a-switch
            :checked="record.visible === 1"
            checked-children="显示"
            un-checked-children="隐藏"
            size="small"
            @change="(checked: boolean) => toggleVisible(record, checked)"
          />
        </template>
        <template v-if="column.key === 'action'">
          <a-space>
            <a @click.prevent="openModal(undefined, record.id)">新增子菜单</a>
            <a-divider type="vertical" />
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
    :title="editId ? '编辑菜单' : '新增菜单'"
    @ok="onSubmit"
    width="640px"
    :confirm-loading="submitting"
    :mask-closable="false"
  >
    <a-form :model="form" :label-col="{ span: 6 }" :wrapper-col="{ span: 16 }" style="margin-top: 16px">
      <a-form-item label="上级菜单">
        <div style="display: flex; gap: 8px; align-items: center">
          <a-tree-select
            v-model:value="form.parentId"
            :tree-data="treeSelectData"
            :field-names="{ label: 'menuName', value: 'id', children: 'children' }"
            placeholder="请选择上级菜单"
            allow-clear
            tree-default-expand-all
            style="flex: 1"
          />
          <a-button size="small" @click="toggleParentSelect">
            {{ form.parentId === '0' && parentSelectState !== 'cleared' ? '取消全选' : '全选' }}
          </a-button>
        </div>
      </a-form-item>
      <a-form-item label="菜单类型" name="menuType" :rules="[{ required: true, message: '请选择菜单类型' }]">
        <a-select v-model:value="form.menuType" placeholder="请选择菜单类型">
          <a-select-option value="dir">目录</a-select-option>
          <a-select-option value="menu">菜单</a-select-option>
          <a-select-option value="button">按钮</a-select-option>
        </a-select>
      </a-form-item>
      <a-form-item label="菜单名称" name="menuName" :rules="[{ required: true, message: '请输入菜单名称' }]">
        <a-input v-model:value="form.menuName" placeholder="请输入菜单名称" />
      </a-form-item>
      <template v-if="form.menuType === 'dir' || form.menuType === 'menu'">
        <a-form-item label="路由路径">
          <a-input v-model:value="form.path" placeholder="请输入路由路径" />
        </a-form-item>
        <a-form-item v-if="form.menuType === 'menu'" label="组件路径">
          <a-input v-model:value="form.component" placeholder="请输入组件路径" />
        </a-form-item>
        <a-form-item label="图标">
          <a-input v-model:value="form.icon" placeholder="请输入图标名称，如 SettingOutlined">
            <template #prefix>
              <component :is="getIconComponent(form.icon)" v-if="form.icon" style="font-size: 14px" />
            </template>
          </a-input>
        </a-form-item>
      </template>
      <a-form-item v-if="form.menuType === 'button'" label="权限标识" name="permission" :rules="[{ required: true, message: '请输入权限标识' }]">
        <a-input v-model:value="form.permission" placeholder="请输入权限标识" />
      </a-form-item>
      <a-form-item label="排序">
        <a-input-number v-model:value="form.sortOrder" :min="0" style="width: 100%" />
      </a-form-item>
      <a-form-item label="状态">
        <a-switch v-model:checked="statusChecked" checked-children="启用" un-checked-children="禁用" />
      </a-form-item>
      <a-form-item v-if="form.menuType === 'dir' || form.menuType === 'menu'" label="是否可见">
        <a-switch v-model:checked="visibleChecked" checked-children="显示" un-checked-children="隐藏" />
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup lang="ts">
import { reactive, ref, computed } from 'vue'
import { message } from 'ant-design-vue'
import * as AntdIcons from '@ant-design/icons-vue'
import { menuApi, type MenuVo } from '@/api/menu'

const loading = ref(false)
const submitting = ref(false)
const data = ref<MenuVo[]>([])
const modalVisible = ref(false)
const editId = ref<string | null>(null)
const parentSelectState = ref<'root' | 'cleared'>('root')

const form = reactive({
  parentId: '0' as string,
  menuName: '',
  menuType: 'menu',
  path: '',
  component: '',
  icon: '',
  sortOrder: 0,
  permission: '',
  status: 1,
  visible: 1,
})

const statusChecked = computed({
  get: () => form.status === 1,
  set: (v: boolean) => { form.status = v ? 1 : 0 },
})

const visibleChecked = computed({
  get: () => form.visible === 1,
  set: (v: boolean) => { form.visible = v ? 1 : 0 },
})

const treeSelectData = computed(() => {
  return [{ id: '0', menuName: '根目录', children: data.value }]
})

function getIconComponent(iconName: string) {
  if (!iconName) return null
  const icons = AntdIcons as any
  if (icons[iconName]) return icons[iconName]
  const pascalName = iconName
    .split('-')
    .map((s: string) => s.charAt(0).toUpperCase() + s.slice(1))
    .join('')
  if (icons[pascalName]) return icons[pascalName]
  const outlinedName = pascalName + 'Outlined'
  if (icons[outlinedName]) return icons[outlinedName]
  return null
}

function toggleParentSelect() {
  if (form.parentId === '0' && parentSelectState.value !== 'cleared') {
    form.parentId = undefined as unknown as string
    parentSelectState.value = 'cleared'
  } else {
    form.parentId = '0'
    parentSelectState.value = 'root'
  }
}

const columns = [
  { title: '菜单名称', dataIndex: 'menuName', key: 'menuName', width: 200 },
  { title: '图标', dataIndex: 'icon', key: 'icon', width: 80, align: 'center' as const },
  { title: '类型', key: 'menuType', width: 80, align: 'center' as const },
  { title: '路由路径', dataIndex: 'path', key: 'path', width: 160 },
  { title: '权限标识', dataIndex: 'permission', key: 'permission', width: 160 },
  { title: '排序', dataIndex: 'sortOrder', key: 'sortOrder', width: 80, align: 'center' as const },
  { title: '状态', key: 'status', width: 80, align: 'center' as const },
  { title: '可见', key: 'visible', width: 100, align: 'center' as const },
  { title: '操作', key: 'action', width: 220, align: 'center' as const },
]

function resetForm() {
  form.parentId = '0'
  form.menuName = ''
  form.menuType = 'menu'
  form.path = ''
  form.component = ''
  form.icon = ''
  form.sortOrder = 0
  form.permission = ''
  form.status = 1
  form.visible = 1
  parentSelectState.value = 'root'
}

async function fetchData() {
  loading.value = true
  try {
    data.value = await menuApi.listTree()
  } catch (e: any) {
    message.error(e?.message || '获取菜单列表失败')
  } finally {
    loading.value = false
  }
}

function openModal(record?: MenuVo, parentId?: string) {
  editId.value = null
  resetForm()

  if (record) {
    editId.value = record.id
    form.parentId = record.parentId
    form.menuName = record.menuName
    form.menuType = record.menuType
    form.path = record.path || ''
    form.component = record.component || ''
    form.icon = record.icon || ''
    form.sortOrder = record.sortOrder
    form.permission = record.permission || ''
    form.status = record.status
    form.visible = record.visible
    parentSelectState.value = record.parentId === '0' ? 'root' : 'cleared'
  } else if (parentId !== undefined) {
    form.parentId = parentId
    parentSelectState.value = parentId === '0' ? 'root' : 'cleared'
  }

  modalVisible.value = true
}

async function onSubmit() {
  if (!form.menuName) {
    message.warning('请输入菜单名称')
    return
  }
  if (form.menuType === 'button' && !form.permission) {
    message.warning('请输入权限标识')
    return
  }

  submitting.value = true
  try {
    if (editId.value) {
      await menuApi.update(editId.value, { ...form })
      message.success('更新成功')
    } else {
      await menuApi.create({ ...form })
      message.success('新增成功')
    }
    modalVisible.value = false
    fetchData()
  } catch (e: any) {
    message.error(e?.message || (editId.value ? '更新失败' : '新增失败'))
  } finally {
    submitting.value = false
  }
}

async function toggleVisible(record: MenuVo, checked: boolean) {
  try {
    await menuApi.update(record.id, { ...record, visible: checked ? 1 : 0 })
    message.success(checked ? '已显示' : '已隐藏')
    fetchData()
  } catch (e: any) {
    message.error(e?.message || '更新可见状态失败')
  }
}

async function doDelete(id: string) {
  try {
    await menuApi.delete(id)
    message.success('删除成功')
    fetchData()
  } catch (e: any) {
    message.error(e?.message || '删除失败')
  }
}

fetchData()
</script>
