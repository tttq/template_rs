<template>
  <a-card title="角色管理" :bordered="false">
    <template #extra>
      <a-button type="primary" @click="openModal()">新增角色</a-button>
    </template>
    <a-table
      :columns="columns"
      :data-source="data"
      :loading="loading"
      row-key="id"
      :pagination="false"
      size="middle"
      default-expand-all-rows
    >
      <template #bodyCell="{ column, record }">
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
      <a-form-item label="上级角色" name="parentId">
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
      <a-form-item label="角色名称" name="roleName">
        <a-input v-model:value="form.roleName" placeholder="请输入角色名称" />
      </a-form-item>
      <a-form-item label="角色编码" name="roleCode">
        <a-input v-model:value="form.roleCode" placeholder="请输入角色编码" :disabled="!!editId" />
      </a-form-item>
      <a-form-item label="排序" name="roleSort">
        <a-input-number v-model:value="form.roleSort" :min="0" style="width: 100%" placeholder="请输入排序" />
      </a-form-item>
      <a-form-item label="状态" name="status">
        <a-radio-group v-model:value="form.status">
          <a-radio :value="1">启用</a-radio>
          <a-radio :value="0">禁用</a-radio>
        </a-radio-group>
      </a-form-item>
      <a-form-item label="备注" name="remark">
        <a-textarea v-model:value="form.remark" :rows="3" placeholder="请输入备注" />
      </a-form-item>
    </a-form>
  </a-modal>

  <a-modal
    v-model:open="permModalVisible"
    title="分配权限"
    :confirm-loading="permSubmitLoading"
    @ok="onAssignMenus"
    width="520px"
  >
    <a-spin :spinning="permLoading">
      <div style="margin-bottom: 12px">
        <a-space>
          <a-button size="small" @click="onCheckAll">全选</a-button>
          <a-button size="small" @click="onUncheckAll">取消全选</a-button>
          <a-button size="small" @click="onExpandAll">展开全部</a-button>
          <a-button size="small" @click="onCollapseAll">折叠全部</a-button>
        </a-space>
      </div>
      <a-tree
        ref="permTreeRef"
        v-model:checkedKeys="checkedKeys"
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

const loading = ref(false)
const data = ref<RoleVo[]>([])
const modalVisible = ref(false)
const editId = ref<string | null>(null)
const submitLoading = ref(false)
const formRef = ref<FormInstance>()

const form = reactive({
  parentId: '0' as string | undefined,
  roleName: '',
  roleCode: '',
  roleSort: 0,
  status: 1,
  remark: '',
})

const formRules = {
  roleName: [{ required: true, message: '请输入角色名称', trigger: 'blur' }],
  roleCode: [{ required: true, message: '请输入角色编码', trigger: 'blur' }],
}

const roleTreeData = computed(() => {
  const root: RoleVo = { id: '0', parentId: '0', roleName: '顶级角色', roleCode: '', roleSort: 0, status: 1, createTime: '', updateTime: '', children: data.value }
  return [root]
})

const columns = [
  { title: '角色名称', dataIndex: 'roleName', key: 'roleName' },
  { title: '角色编码', dataIndex: 'roleCode', key: 'roleCode' },
  { title: '排序', dataIndex: 'roleSort', key: 'roleSort', width: 80 },
  { title: '状态', key: 'status', width: 80 },
  { title: '创建时间', dataIndex: 'createTime', key: 'createTime', width: 180 },
  { title: '操作', key: 'action', width: 280 },
]

async function fetchData() {
  loading.value = true
  try {
    data.value = await roleApi.listTree()
  } catch (e: any) {
    message.error(e?.message || '获取角色列表失败')
  } finally {
    loading.value = false
  }
}

function openModal(record?: RoleVo, parentId?: string) {
  editId.value = record?.id || null
  if (record) {
    form.parentId = record.parentId ?? '0'
    form.roleName = record.roleName
    form.roleCode = record.roleCode
    form.roleSort = record.roleSort
    form.status = record.status
    form.remark = record.remark || ''
  } else {
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

function onCheckAll() {
  checkedKeys.value = collectAllKeys(menuTreeData.value)
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
      menuApi.listTree(),
      roleApi.getMenuIds(record.id),
    ])
    menuTreeData.value = menus
    checkedKeys.value = ids
    expandedKeys.value = collectAllKeys(menus)
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
    await roleApi.assignMenus(permRoleId.value, checkedKeys.value)
    message.success('权限分配成功')
    permModalVisible.value = false
  } catch (e: any) {
    message.error(e?.message || '权限分配失败')
  } finally {
    permSubmitLoading.value = false
  }
}

fetchData()
</script>
