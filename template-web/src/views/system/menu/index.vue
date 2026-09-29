<template>
  <div>
    <a-card :bordered="false">
      <a-form
        layout="inline"
        :model="searchForm"
        style="margin-bottom: 16px; row-gap: 12px"
        @finish="fetchData"
      >
        <a-form-item label="所属客户端">
          <a-select
            v-model:value="searchForm.clientId"
            placeholder="全部客户端"
            allow-clear
            style="width: 180px"
            @change="onClientChange"
          >
            <a-select-option
              v-for="item in clientOptions"
              :key="item.id"
              :value="item.id"
            >
              {{ item.clientName }}（{{ item.clientCode }}）
            </a-select-option>
          </a-select>
        </a-form-item>
        <a-form-item label="菜单名称">
          <a-input
            v-model:value="searchForm.menuName"
            placeholder="请输入菜单名称"
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
            新增菜单
          </a-button>
          <a-button
            v-permission="'menu:export'"
            :loading="exporting"
            @click="doExport"
          >
            导出 Excel
          </a-button>
        </a-space>
      </template>

    <a-table
      v-model:expanded-row-keys="expandedRowKeys"
      :columns="columns"
      :data-source="data"
      :loading="loading"
      row-key="id"
      :pagination="false"
      :row-selection="rowSelection"
      :scroll="{ x: 1824, y: fitY }"
      size="middle"
    >
      <template #bodyCell="{ column, record, index }">
        <template v-if="column.key === 'index'">
          {{ index + 1 }}
        </template>
        <template v-if="column.key === 'clientId'">
          <a-tag
            v-if="!isClientRow(record) && record.clientId"
            color="blue"
          >
            {{ clientNameMap[record.clientId] || record.clientId }}
          </a-tag>
          <span v-else>-</span>
        </template>
        <template v-if="column.key === 'menuName'">
          <span>{{ record.menuName }}</span>
        </template>
        <template v-if="column.key === 'icon'">
          <component
            :is="getIconComponent(record.icon)"
            v-if="record.icon"
            style="font-size: 16px"
          />
          <span
            v-else
            style="color: #d9d9d9"
          >-</span>
        </template>
        <template v-if="column.key === 'menuType'">
          <template v-if="!isClientRow(record)">
            <a-tag
              v-if="record.menuType === 'dir'"
              color="blue"
            >
              目录
            </a-tag>
            <a-tag
              v-else-if="record.menuType === 'menu'"
              color="green"
            >
              菜单
            </a-tag>
            <a-tag
              v-else-if="record.menuType === 'button'"
              color="orange"
            >
              按钮
            </a-tag>
          </template>
          <span
            v-else
            style="color: #d9d9d9"
          >-</span>
        </template>
        <template v-if="column.key === 'status'">
          <template v-if="!isClientRow(record)">
            <a-tag :color="record.status === 1 ? 'green' : 'red'">
              {{ record.status === 1 ? '启用' : '禁用' }}
            </a-tag>
          </template>
          <span
            v-else
            style="color: #d9d9d9"
          >-</span>
        </template>
        <template v-if="column.key === 'visible'">
          <a-switch
            v-if="!isClientRow(record)"
            :checked="record.visible === 1"
            checked-children="显示"
            un-checked-children="隐藏"
            size="small"
            @change="(checked: boolean) => toggleVisible(record, checked)"
          />
          <span
            v-else
            style="color: #d9d9d9"
          >-</span>
        </template>
        <template v-if="column.key === 'action'">
          <a-space>
            <template v-if="isClientRow(record)">
              <a @click.prevent="openModal(undefined, undefined, record.clientId)">新增菜单</a>
            </template>
            <template v-else>
              <a @click.prevent="openModal(undefined, record.id, record.clientId)">新增子菜单</a>
              <a-divider type="vertical" />
              <a @click.prevent="openModal(record)">编辑</a>
              <a-divider type="vertical" />
              <a-popconfirm
                title="确定删除?"
                @confirm="doDelete(record.id)"
              >
                <a style="color: red">删除</a>
              </a-popconfirm>
            </template>
          </a-space>
        </template>
      </template>
    </a-table>
    </a-card>
  </div>

  <a-modal
    v-model:open="modalVisible"
    :title="editId ? '编辑菜单' : '新增菜单'"
    width="640px"
    :confirm-loading="submitting"
    :mask-closable="false"
    @ok="onSubmit"
  >
    <a-form
      :model="form"
      :label-col="{ span: 6 }"
      :wrapper-col="{ span: 16 }"
      style="margin-top: 16px"
    >
      <a-form-item
        label="所属客户端"
        name="clientId"
        :rules="[{ required: true, message: '请选择所属客户端' }]"
      >
        <a-select
          v-model:value="form.clientId"
          placeholder="请选择所属客户端"
          :disabled="!!editId"
        >
          <a-select-option
            v-for="item in clientOptions"
            :key="item.id"
            :value="item.id"
          >
            {{ item.clientName }}（{{ item.clientCode }}）
          </a-select-option>
        </a-select>
      </a-form-item>
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
          <a-button
            size="small"
            @click="toggleParentSelect"
          >
            {{ form.parentId === '0' && parentSelectState !== 'cleared' ? '取消全选' : '全选' }}
          </a-button>
        </div>
      </a-form-item>
      <a-form-item
        label="菜单类型"
        name="menuType"
        :rules="[{ required: true, message: '请选择菜单类型' }]"
      >
        <a-select
          v-model:value="form.menuType"
          placeholder="请选择菜单类型"
        >
          <a-select-option value="dir">
            目录
          </a-select-option>
          <a-select-option value="menu">
            菜单
          </a-select-option>
          <a-select-option value="button">
            按钮
          </a-select-option>
        </a-select>
      </a-form-item>
      <a-form-item
        label="菜单名称"
        name="menuName"
        :rules="[{ required: true, message: '请输入菜单名称' }]"
      >
        <a-input
          v-model:value="form.menuName"
          placeholder="请输入菜单名称"
        />
      </a-form-item>
      <template v-if="form.menuType === 'dir' || form.menuType === 'menu'">
        <a-form-item label="路由路径">
          <a-input
            v-model:value="form.path"
            placeholder="请输入路由路径"
          />
        </a-form-item>
        <a-form-item
          v-if="form.menuType === 'menu'"
          label="组件路径"
        >
          <a-input
            v-model:value="form.component"
            placeholder="请输入组件路径"
          />
        </a-form-item>
        <a-form-item label="图标">
          <a-input
            v-model:value="form.icon"
            placeholder="请输入图标名称，如 SettingOutlined"
          >
            <template #prefix>
              <component
                :is="getIconComponent(form.icon)"
                v-if="form.icon"
                style="font-size: 14px"
              />
            </template>
          </a-input>
        </a-form-item>
      </template>
      <a-form-item
        v-if="form.menuType === 'button'"
        label="权限标识"
        name="permission"
        :rules="[{ required: true, message: '请输入权限标识' }]"
      >
        <a-input
          v-model:value="form.permission"
          placeholder="请输入权限标识"
        />
      </a-form-item>
      <a-form-item label="排序">
        <a-input-number
          v-model:value="form.sortOrder"
          :min="0"
          style="width: 100%"
        />
      </a-form-item>
      <a-form-item label="状态">
        <a-switch
          v-model:checked="statusChecked"
          checked-children="启用"
          un-checked-children="禁用"
        />
      </a-form-item>
      <a-form-item
        v-if="form.menuType === 'dir' || form.menuType === 'menu'"
        label="是否可见"
      >
        <a-switch
          v-model:checked="visibleChecked"
          checked-children="显示"
          un-checked-children="隐藏"
        />
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup lang="ts">
import { reactive, ref, computed } from 'vue'
import { useRoute } from 'vue-router'
import { message } from 'ant-design-vue'
import * as AntdIcons from '@ant-design/icons-vue'
import { menuApi, type MenuVo } from '@/api/menu'
import { clientApi, type ClientOption } from '@/api/client'
import { buildTree } from '@/utils/tree'
import { cascadeTreeSelection } from '@/utils/treeSelection'
import { useFitTableHeight } from '@/utils/useFitTableHeight'
import { exportList } from '@/api/export'

const loading = ref(false)
const submitting = ref(false)
/** 真实菜单树（后端 list 结果经 buildTree） */
const menuTree = ref<MenuVo[]>([])
const route = useRoute()
const clientOptions = ref<ClientOption[]>([])
const clientNameMap = computed<Record<string, string>>(() => {
  const map: Record<string, string> = {}
  clientOptions.value.forEach((item) => {
    map[item.id] = item.clientName
  })
  return map
})

/** 树形行受控展开（默认全部折叠，点击展开符展开） */
const expandedRowKeys = ref<string[]>([])
const { fitRef, fitY } = useFitTableHeight()

/** 客户端分组展示：每个客户端作为顶层目录节点，其下挂该客户端的菜单树 */
const data = computed<MenuVo[]>(() => {
  // 按 clientId 归组真实菜单树的根节点
  const byClient = new Map<string, MenuVo[]>()
  for (const root of menuTree.value) {
    const cid = root.clientId
    if (!cid) continue
    const list = byClient.get(cid) || []
    list.push(root)
    byClient.set(cid, list)
  }
  // 按 clientOptions 顺序（或出现顺序）生成客户端分组节点
  const seen = new Set<string>()
  const groups: MenuVo[] = []
  const order = clientOptions.value.length
    ? clientOptions.value.map((c) => c.id)
    : Array.from(byClient.keys())
  for (const cid of order) {
    if (seen.has(cid) || !byClient.has(cid)) continue
    seen.add(cid)
    const opt = clientOptions.value.find((c) => c.id === cid)
    groups.push({
      id: `client-${cid}`,
      parentId: '0',
      clientId: cid,
      menuName: opt ? `${opt.clientName}（${opt.clientCode}）` : (clientNameMap.value[cid] || cid),
      menuType: 'dir',
      sortOrder: 0,
      status: 1,
      visible: 1,
      createTime: '',
      updateTime: '',
      children: byClient.get(cid) || [],
    } as MenuVo)
  }
  return groups
})

/** 是否为客户端分组节点（非真实菜单） */
function isClientRow(record: any): boolean {
  return typeof record?.id === 'string' && record.id.startsWith('client-')
}

const modalVisible = ref(false)
const editId = ref<string | null>(null)
const parentSelectState = ref<'root' | 'cleared'>('root')
const exporting = ref(false)

// 行选择（用于勾选导出）：父子级联——勾选父级自动带上全部子级，
// 取消子级时只要父级下还有子级被勾选，父级就保持勾选
const rowSelection = reactive({
  selectedRowKeys: [] as string[],
  onChange: (selectedRowKeys: string[]) => {
    rowSelection.selectedRowKeys = cascadeTreeSelection<MenuVo>({
      tree: data.value,
      prevKeys: rowSelection.selectedRowKeys,
      nextKeys: selectedRowKeys,
      isSelectable: (record) => !isClientRow(record),
    })
  },
  getCheckboxProps: (record: any) => ({
    disabled: isClientRow(record),
  }),
})

const form = reactive({
  clientId: undefined as string | undefined,
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

const searchForm = reactive({
  // clientId：界面「所属客户端」下拉，发送给后端 MenuQuery.clientId，筛选 auth_sys_menu.client_id
  clientId: (route.query.clientId as string) || undefined,
  // menuName：界面「菜单名称」输入框，发送给后端 MenuQuery.menuName，筛选 auth_sys_menu.menu_name 模糊 contains
  menuName: '',
  // createBy：界面「创建人」输入框，发送给后端 MenuQuery.createBy，筛选 auth_sys_menu.create_by 模糊 contains
  createBy: '',
  // createTimeRange：界面「创建时间」范围，拆分发送给后端 MenuQuery.createTimeStart/createTimeEnd，筛选 auth_sys_menu.create_time 区间（>= 开始，<= 结束）
  createTimeRange: [null, null] as [string | null, string | null],
})

/** 上级菜单（仅展示当前客户端下的真实菜单，避免跨客户端选父级） */
const treeSelectData = computed(() => {
  const cid = form.clientId || searchForm.clientId
  const roots = cid
    ? menuTree.value.filter((m) => m.clientId === cid)
    : menuTree.value
  return [{ id: '0', menuName: '根目录', children: roots }]
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
  { title: '序号', key: 'index', width: 64 },
  { title: '所属客户端', key: 'clientId', width: 150 },
  { title: '菜单名称', dataIndex: 'menuName', key: 'menuName', width: 200 },
  { title: '图标', dataIndex: 'icon', key: 'icon', width: 80, align: 'center' as const },
  { title: '类型', key: 'menuType', width: 80, align: 'center' as const },
  { title: '路由路径', dataIndex: 'path', key: 'path', width: 160 },
  { title: '权限标识', dataIndex: 'permission', key: 'permission', width: 160 },
  { title: '排序', dataIndex: 'sortOrder', key: 'sortOrder', width: 80, align: 'center' as const },
  { title: '状态', key: 'status', width: 80, align: 'center' as const },
  { title: '可见', key: 'visible', width: 100, align: 'center' as const },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 180 },
  { title: '创建人', dataIndex: 'createBy', key: 'createBy', width: 120 },
  { title: '修改时间', dataIndex: 'updateTime', key: 'updateTime', width: 180 },
  { title: '修改人', dataIndex: 'updateBy', key: 'updateBy', width: 120 },
  { title: '操作', key: 'action', width: 220, align: 'center' as const },
]

function resetForm(clientId?: string) {
  form.clientId = clientId || searchForm.clientId || undefined
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
    const params: Record<string, any> = {
      page: 1,
      pageSize: 1000,
      clientId: searchForm.clientId || undefined,
      menuName: searchForm.menuName || undefined,
      createBy: searchForm.createBy || undefined,
    }
    // 时间范围拆分为 createTimeStart/createTimeEnd → MenuQuery.createTimeStart/End，筛选 auth_sys_menu.create_time 区间（前者 gte，后者 lte）
    if (searchForm.createTimeRange && searchForm.createTimeRange.length === 2) {
      if (searchForm.createTimeRange[0]) {
        params.createTimeStart = searchForm.createTimeRange[0]
      }
      if (searchForm.createTimeRange[1]) {
        params.createTimeEnd = searchForm.createTimeRange[1]
      }
    }
    const res = await menuApi.list(params)
    const items = Array.isArray(res) ? res : res.items
    menuTree.value = buildTree(items)
    // 默认展开「客户端分组」节点（用户看客户端→菜单结构），具体菜单仍收起
    expandedRowKeys.value = data.value.map((g) => g.id)
  } catch (e: any) {
    message.error(e?.message || '获取菜单列表失败')
  } finally {
    loading.value = false
  }
}

function onCreateTimeRangeChange(value: [string | null, string | null]) {
  searchForm.createTimeRange = value
}

/** 切换客户端后重拉该客户端的菜单树 */
function onClientChange() {
  expandedRowKeys.value = []
  fetchData()
}

function resetSearch() {
  searchForm.menuName = ''
  searchForm.createBy = ''
  searchForm.createTimeRange = [null, null]
  fetchData()
}

/** 打开新增/编辑弹窗；clientId 用于「客户端分组节点」上的新增菜单，锁定所属客户端 */
function openModal(record?: MenuVo, parentId?: string, clientId?: string) {
  editId.value = null
  resetForm(clientId)

  if (record) {
    editId.value = record.id
    form.clientId = record.clientId
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

  if (!editId.value && !form.clientId) {
    form.clientId = searchForm.clientId || undefined
  }
  modalVisible.value = true
}

async function onSubmit() {
  if (!form.clientId) {
    message.warning('请选择所属客户端')
    return
  }
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

// 导出功能：支持按时间返回、勾选导出
async function doExport() {
  const selectedKeys = rowSelection.selectedRowKeys
  const params: Record<string, any> = {
    clientId: searchForm.clientId || undefined,
    menuName: searchForm.menuName || undefined,
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
      taskType: 'menu',
      query: params,
      syncUrl: '/api/system/menus/export',
      filename: `menus-export-${Date.now()}.xlsx`,
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

async function loadClientOptions() {
  try {
    clientOptions.value = (await clientApi.options()) || []
  } catch {
    clientOptions.value = []
  }
}

loadClientOptions()
fetchData()
</script>
