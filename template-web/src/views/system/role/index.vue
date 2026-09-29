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
            @change="fetchData"
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
        <a-form-item label="角色名称">
          <a-input
            v-model:value="searchForm.roleName"
            placeholder="请输入角色名称"
            allow-clear
            style="width: 160px"
            @press-enter="fetchData"
          />
        </a-form-item>
        <a-form-item label="角色编码">
          <a-input
            v-model:value="searchForm.roleCode"
            placeholder="请输入角色编码"
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
            新增角色
          </a-button>
          <a-button
            v-permission="'role:export'"
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
      size="middle"
      :scroll="{ x: 1404, y: fitY }"
      :row-selection="rowSelection"
      :default-expand-all-rows="false"
    >
      <template #bodyCell="{ column, record, index }">
        <template v-if="column.key === 'index'">
          {{ index + 1 }}
        </template>
        <template v-if="column.key === 'clientId'">
          <a-tag
            v-if="record.clientId"
            color="blue"
          >
            {{ clientNameMap[record.clientId] || record.clientId }}
          </a-tag>
          <span v-else>-</span>
        </template>
        <template v-if="column.key === 'status'">
          <a-tag :color="record.status === 1 ? 'green' : 'red'">
            {{ record.status === 1 ? '启用' : '禁用' }}
          </a-tag>
        </template>
        <template v-if="column.key === 'action'">
          <a-space>
            <a @click.prevent="openModal(record)">编辑</a>
            <a-divider type="vertical" />
            <a @click.prevent="openModal(undefined, record.id)">新增子角色</a>
            <a-divider type="vertical" />
            <a @click.prevent="openPermModal(record)">分配权限</a>
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
  </div>

  <a-modal
    v-model:open="modalVisible"
    :title="editId ? '编辑角色' : '新增角色'"
    :confirm-loading="submitLoading"
    @ok="onSubmit"
  >
    <a-form
      ref="formRef"
      :model="form"
      :rules="formRules"
      :label-col="{ span: 5 }"
      :wrapper-col="{ span: 18 }"
    >
      <a-form-item
        label="所属客户端"
        name="clientId"
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
      <a-form-item
        label="上级角色"
        name="parentId"
      >
        <a-tree-select
          v-model:value="form.parentId"
          :tree-data="roleTreeData"
          :field-names="{ label: 'roleName', value: 'id', children: 'children' }"
          placeholder="请选择上级角色（留空为顶级）"
          allow-clear
          tree-default-expand-all
          :disabled="!!editId && form.parentId === '0'"
        />
      </a-form-item>
      <a-form-item
        label="角色名称"
        name="roleName"
      >
        <a-input
          v-model:value="form.roleName"
          placeholder="请输入角色名称"
        />
      </a-form-item>
      <a-form-item
        label="角色编码"
        name="roleCode"
      >
        <a-input
          v-model:value="form.roleCode"
          placeholder="请输入角色编码"
          :disabled="!!editId"
        />
      </a-form-item>
      <a-form-item
        label="排序"
        name="roleSort"
      >
        <a-input-number
          v-model:value="form.roleSort"
          :min="0"
          style="width: 100%"
          placeholder="请输入排序"
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

  <a-modal
    v-model:open="permModalVisible"
    title="分配权限"
    :confirm-loading="permSubmitLoading"
    width="520px"
    @ok="onAssignMenus"
  >
    <a-spin :spinning="permLoading">
      <div style="margin-bottom: 12px">
        <a-space>
          <a-button
            size="small"
            @click="onCheckAll"
          >
            全选
          </a-button>
          <a-button
            size="small"
            @click="onUncheckAll"
          >
            取消全选
          </a-button>
          <a-button
            size="small"
            @click="onExpandAll"
          >
            展开全部
          </a-button>
          <a-button
            size="small"
            @click="onCollapseAll"
          >
            折叠全部
          </a-button>
        </a-space>
      </div>
      <a-tree
        ref="permTreeRef"
        v-model:checked-keys="checkedKeys"
        checkable
        :tree-data="menuTreeData"
        :field-names="{ title: 'menuName', key: 'id', children: 'children' }"
        :expanded-keys="expandedKeys"
        @expand="onExpand"
      />
    </a-spin>
  </a-modal>
</template>

<script setup lang="ts">
import { reactive, ref, computed, nextTick } from 'vue'
import { message } from 'ant-design-vue'
import type { FormInstance } from 'ant-design-vue'
import type { TreeProps } from 'ant-design-vue'
import { roleApi, type RoleVo } from '@/api/role'
import { menuApi, type MenuVo } from '@/api/menu'
import { clientApi, type ClientOption } from '@/api/client'
import { buildTree } from '@/utils/tree'
import { cascadeTreeSelection } from '@/utils/treeSelection'
import { useFitTableHeight } from '@/utils/useFitTableHeight'
import { exportList } from '@/api/export'

const loading = ref(false)
const data = ref<RoleVo[]>([])
const clientOptions = ref<ClientOption[]>([])
const clientNameMap = computed<Record<string, string>>(() => {
  const map: Record<string, string> = {}
  clientOptions.value.forEach((item) => {
    map[item.id] = item.clientName
  })
  return map
})

const { fitRef, fitY } = useFitTableHeight()

const modalVisible = ref(false)
const editId = ref<string | null>(null)
const submitLoading = ref(false)
const formRef = ref<FormInstance>()
const exporting = ref(false)

// 行选择（用于勾选导出）：父子级联——勾选父级自动带上全部子级，
// 取消子级时只要父级下还有子级被勾选，父级就保持勾选
const rowSelection = reactive({
  selectedRowKeys: [] as string[],
  onChange: (selectedRowKeys: string[]) => {
    rowSelection.selectedRowKeys = cascadeTreeSelection<RoleVo>({
      tree: data.value,
      prevKeys: rowSelection.selectedRowKeys,
      nextKeys: selectedRowKeys,
    })
  },
})

const form = reactive({
  clientId: undefined as string | undefined,
  parentId: '0' as string | undefined,
  roleName: '',
  roleCode: '',
  roleSort: 0,
  status: 1,
  remark: '',
})

const formRules = {
  clientId: [{ required: true, message: '请选择所属客户端', trigger: 'change' }],
  roleName: [{ required: true, message: '请输入角色名称', trigger: 'blur' }],
  roleCode: [{ required: true, message: '请输入角色编码', trigger: 'blur' }],
}

const searchForm = reactive({
  // clientId：界面「所属客户端」下拉，发送给后端 RoleQuery.clientId，筛选 auth_sys_role.client_id
  clientId: undefined as string | undefined,
  // roleName：界面「角色名称」输入框，发送给后端 RoleQuery.roleName，筛选 auth_sys_role.role_name 模糊 contains
  roleName: '',
  // roleCode：界面「角色编码」输入框，发送给后端 RoleQuery.roleCode，筛选 auth_sys_role.role_code 模糊 contains
  roleCode: '',
  // createBy：界面「创建人」输入框，发送给后端 RoleQuery.createBy，筛选 auth_sys_role.create_by 模糊 contains
  createBy: '',
  // createTimeRange：界面「创建时间」范围，拆分发送给后端 RoleQuery.createTimeStart/createTimeEnd，筛选 auth_sys_role.create_time 区间（>= 开始，<= 结束）
  createTimeRange: [null, null] as [string | null, string | null],
})

const roleTreeData = computed(() => {
  const root: RoleVo = { id: '0', parentId: '0', clientId: '', roleName: '顶级角色', roleCode: '', roleSort: 0, status: 1, createTime: '', updateTime: '', children: data.value }
  return [root]
})

const columns = [
  { title: '序号', key: 'index', width: 64 },
  { title: '所属客户端', key: 'clientId', width: 150 },
  { title: '角色名称', dataIndex: 'roleName', key: 'roleName', width: 180 },
  { title: '角色编码', dataIndex: 'roleCode', key: 'roleCode', width: 120 },
  { title: '排序', dataIndex: 'roleSort', key: 'roleSort', width: 80 },
  { title: '状态', key: 'status', width: 80 },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 180 },
  { title: '创建人', dataIndex: 'createBy', key: 'createBy', width: 120 },
  { title: '修改时间', dataIndex: 'updateTime', key: 'updateTime', width: 180 },
  { title: '修改人', dataIndex: 'updateBy', key: 'updateBy', width: 120 },
  { title: '操作', key: 'action', width: 280 },
]

async function fetchData() {
  loading.value = true
  try {
    const params: Record<string, any> = {
      page: 1,
      pageSize: 1000,
      clientId: searchForm.clientId || undefined,
      roleName: searchForm.roleName || undefined,
      roleCode: searchForm.roleCode || undefined,
      createBy: searchForm.createBy || undefined,
    }
    // 时间范围拆分为 createTimeStart/createTimeEnd → RoleQuery.createTimeStart/End，筛选 auth_sys_role.create_time 区间（前者 gte，后者 lte）
    if (searchForm.createTimeRange && searchForm.createTimeRange.length === 2) {
      if (searchForm.createTimeRange[0]) {
        params.createTimeStart = searchForm.createTimeRange[0]
      }
      if (searchForm.createTimeRange[1]) {
        params.createTimeEnd = searchForm.createTimeRange[1]
      }
    }
    const res = await roleApi.list(params)
    data.value = buildTree(res.items)
  } catch (e: any) {
    message.error(e?.message || '获取角色列表失败')
  } finally {
    loading.value = false
  }
}

function onCreateTimeRangeChange(value: [string | null, string | null]) {
  searchForm.createTimeRange = value
}

function resetSearch() {
  searchForm.roleName = ''
  searchForm.roleCode = ''
  searchForm.createBy = ''
  searchForm.createTimeRange = [null, null]
  fetchData()
}

function openModal(record?: RoleVo, parentId?: string) {
  editId.value = record?.id || null
  if (record) {
    form.clientId = record.clientId
    form.parentId = record.parentId ?? '0'
    form.roleName = record.roleName
    form.roleCode = record.roleCode
    form.roleSort = record.roleSort
    form.status = record.status
    form.remark = record.remark || ''
  } else {
    form.clientId = searchForm.clientId || undefined
    form.parentId = parentId ?? '0'
    form.roleName = ''
    form.roleCode = ''
    form.roleSort = 0
    form.status = 1
    form.remark = ''
  }
  modalVisible.value = true
  nextTick(() => {
    formRef.value?.clearValidate()
  })
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
      await roleApi.update(editId.value, { ...form })
      message.success('更新成功')
    } else {
      await roleApi.create({ ...form })
      message.success('新增成功')
    }
    modalVisible.value = false
    fetchData()
  } catch (e: any) {
    message.error(e?.message || '操作失败')
  } finally {
    submitLoading.value = false
  }
}

async function doDelete(id: string) {
  try {
    await roleApi.delete(id)
    message.success('删除成功')
    fetchData()
  } catch (e: any) {
    message.error(e?.message || '删除失败')
  }
}

const permModalVisible = ref(false)
const permLoading = ref(false)
const permSubmitLoading = ref(false)
const permRoleId = ref<string | null>(null)
const checkedKeys = ref<string[]>([])
const menuTreeData = ref<MenuVo[]>([])
const expandedKeys = ref<string[]>([])
const permTreeRef = ref()

function collectAllKeys(tree: MenuVo[], keyField: 'id' = 'id'): string[] {
  const keys: string[] = []
  function walk(nodes: MenuVo[]) {
    for (const node of nodes) {
      keys.push(node[keyField])
      if (node.children?.length) walk(node.children)
    }
  }
  walk(tree)
  return keys
}

/** 全量菜单 → 「客户端→菜单」分组树；所有客户端分组均可勾选（角色跨客户端授权） */
function groupMenusByClient(roots: MenuVo[]): MenuVo[] {
  const byClient = new Map<string, MenuVo[]>()
  for (const root of roots) {
    const cid = root.clientId || ''
    if (!byClient.has(cid)) byClient.set(cid, [])
    byClient.get(cid)!.push(root)
  }
  const order = clientOptions.value
    .map((c) => c.id)
    .concat(Array.from(byClient.keys()).filter((id) => !clientOptions.value.some((c) => c.id === id)))
  const result: MenuVo[] = []
  for (const cid of order) {
    if (!byClient.has(cid)) continue
    const opt = clientOptions.value.find((c) => c.id === cid)
    result.push({
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
  return result
}

/** 收集所有可勾选的真实菜单 id（跳过 disabled 与客户端分组节点），供「全选」使用 */
function collectCheckableKeys(tree: MenuVo[]): string[] {
  const keys: string[] = []
  function walk(nodes: MenuVo[]) {
    for (const node of nodes) {
      if (node.disabled) continue
      if (!String(node.id).startsWith('client-')) keys.push(node.id)
      if (node.children?.length) walk(node.children)
    }
  }
  walk(tree)
  return keys
}

function onCheckAll() {
  checkedKeys.value = collectCheckableKeys(menuTreeData.value)
}

function onUncheckAll() {
  checkedKeys.value = []
}

function onExpandAll() {
  expandedKeys.value = collectAllKeys(menuTreeData.value)
}

function onCollapseAll() {
  expandedKeys.value = []
}

const onExpand: TreeProps['onExpand'] = (keys) => {
  expandedKeys.value = keys as string[]
}

async function openPermModal(record: RoleVo) {
  permRoleId.value = record.id
  permModalVisible.value = true
  permLoading.value = true
  try {
    const [menus, ids] = await Promise.all([
      // 取全量客户端菜单，前端按「客户端→菜单」分组；所有分组均可勾选（角色跨客户端授权）
      menuApi.listTree(),
      roleApi.getMenuIds(record.id),
    ])
    menuTreeData.value = groupMenusByClient(menus)
    checkedKeys.value = ids
    expandedKeys.value = collectAllKeys(menuTreeData.value)
  } catch (e: any) {
    message.error(e?.message || '获取权限数据失败')
  } finally {
    permLoading.value = false
  }
}

async function onAssignMenus() {
  if (permRoleId.value === null) return
  permSubmitLoading.value = true
  try {
    // 过滤掉「client-xxx」合成分组节点 id，只提交真实菜单 id
    const realIds = checkedKeys.value.filter((k) => !String(k).startsWith('client-'))
    await roleApi.assignMenus(permRoleId.value, realIds)
    message.success('权限分配成功')
    permModalVisible.value = false
  } catch (e: any) {
    message.error(e?.message || '权限分配失败')
  } finally {
    permSubmitLoading.value = false
  }
}

// 导出功能：支持按时间返回、勾选导出
async function doExport() {
  const selectedKeys = rowSelection.selectedRowKeys
  const params: Record<string, any> = {
    clientId: searchForm.clientId || undefined,
    roleName: searchForm.roleName || undefined,
    roleCode: searchForm.roleCode || undefined,
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
      taskType: 'role',
      query: params,
      syncUrl: '/api/system/roles/export',
      filename: `roles-export-${Date.now()}.xlsx`,
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
