<template>
  <div>
    <!-- 1. 搜索条件卡 -->
    <a-card :bordered="false">
      <a-form
        layout="inline"
        :model="search"
        style="margin-bottom: 16px; row-gap: 12px"
        @finish="fetchData"
      >
        <a-form-item label="模板编码">
          <a-input
            v-model:value="search.templateCode"
            placeholder="模糊匹配"
            allow-clear
            style="width: 180px"
          />
        </a-form-item>
        <a-form-item label="模板名称">
          <a-input
            v-model:value="search.templateName"
            placeholder="模糊匹配"
            allow-clear
            style="width: 180px"
          />
        </a-form-item>
        <a-form-item label="通知类型">
          <a-select
            v-model:value="search.notifyType"
            placeholder="全部"
            allow-clear
            style="width: 140px"
            :options="notifyTypeOptions"
          />
        </a-form-item>
        <a-form-item label="渠道">
          <a-select
            v-model:value="search.channel"
            placeholder="全部"
            allow-clear
            style="width: 140px"
            :options="channelOptions"
          />
        </a-form-item>
        <a-form-item label="状态">
          <a-select
            v-model:value="search.status"
            placeholder="全部"
            allow-clear
            style="width: 120px"
            :options="statusOptions"
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

    <!-- 2. 表格卡 -->
    <a-card
      ref="fitRef"
      :bordered="false"
      class="mt-3"
    >
      <template #extra>
        <a-space>
          <a-button
            v-permission="'notification:template:add'"
            type="primary"
            @click="openModal()"
          >
            新增模板
          </a-button>
        </a-space>
      </template>

      <a-table
        :columns="columns"
        :data-source="data"
        :loading="loading"
        :pagination="pagination"
        :scroll="{ x: 1300, y: fitY }"
        size="middle"
        row-key="id"
        @change="onTableChange"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.dataIndex === 'templateCode'">
            <a-typography-text code>
              {{ record.templateCode }}
            </a-typography-text>
          </template>
          <template v-else-if="column.dataIndex === 'channel'">
            <a-tag :color="record.channel === 'email' ? 'blue' : 'green'">
              {{ channelLabel(record.channel) }}
            </a-tag>
          </template>
          <template v-else-if="column.dataIndex === 'contentTemplate'">
            <a-tooltip :title="record.contentTemplate">
              <span class="cell-ellipsis">{{ record.contentTemplate }}</span>
            </a-tooltip>
          </template>
          <template v-else-if="column.dataIndex === 'varsHint'">
            <span v-if="record.varsHint">{{ record.varsHint }}</span>
            <span
              v-else
              style="color: #bbb"
            >—</span>
          </template>
          <template v-else-if="column.dataIndex === 'status'">
            <a-switch
              :checked="record.status === 1"
              :disabled="!canEdit"
              checked-children="启用"
              un-checked-children="停用"
              @change="(checked: boolean) => toggleStatus(record, checked)"
            />
          </template>
          <template v-else-if="column.dataIndex === 'action'">
            <a-space>
              <a
                v-permission="'notification:template:edit'"
                @click="openModal(record)"
              >编辑</a>
              <a-popconfirm
                title="确认删除该模板？"
                ok-text="删除"
                cancel-text="取消"
                @confirm="doDelete(record)"
              >
                <a
                  v-permission="'notification:template:delete'"
                  class="danger-link"
                >删除</a>
              </a-popconfirm>
            </a-space>
          </template>
        </template>
      </a-table>
    </a-card>
  </div>

  <!-- 新增 / 编辑弹窗 -->
  <a-modal
    v-model:open="modalVisible"
    :title="form.id ? '编辑通知模板' : '新增通知模板'"
    :width="760"
    :confirm-loading="saving"
    @ok="handleSave"
  >
    <a-form
      ref="formRef"
      :model="form"
      :rules="rules"
      layout="vertical"
    >
      <a-row :gutter="16">
        <a-col :span="12">
          <a-form-item
            label="模板编码"
            name="templateCode"
            extra="业务引用的唯一标识，仅字母/数字/下划线/中划线/点"
          >
            <a-input
              v-model:value="form.templateCode"
              placeholder="如 kyc_approved"
            />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item
            label="模板名称"
            name="templateName"
          >
            <a-input
              v-model:value="form.templateName"
              placeholder="如 实名认证通过"
            />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item
            label="通知类型"
            name="notifyType"
          >
            <a-select
              v-model:value="form.notifyType"
              :options="notifyTypeOptions"
            />
          </a-form-item>
        </a-col>
        <a-col :span="12">
          <a-form-item
            label="渠道"
            name="channel"
            extra="同一编码可分别配置站内信版与邮件版"
          >
            <a-select
              v-model:value="form.channel"
              :options="channelOptions"
            />
          </a-form-item>
        </a-col>
      </a-row>

      <a-form-item
        label="标题模板"
        name="titleTemplate"
        :extra="form.channel === 'email' ? '邮件渠道时作为邮件主题' : '站内消息标题'"
      >
        <a-input
          v-model:value="form.titleTemplate"
          placeholder="如 【Template Admin】注册验证码"
        />
      </a-form-item>

      <a-form-item
        label="正文模板"
        name="contentTemplate"
        extra="使用 ${变量名} 占位，发送时由调用方传入变量表替换"
      >
        <a-textarea
          v-model:value="form.contentTemplate"
          :rows="7"
          placeholder="如 您好：您的验证码为 ${code}，10 分钟内有效。"
        />
      </a-form-item>

      <a-form-item
        label="可用变量提示"
        name="varsHint"
        extra="仅用于提示，多个变量用英文逗号分隔"
      >
        <a-input
          v-model:value="form.varsHint"
          placeholder="如 code 或 applyNo,amount"
        />
      </a-form-item>

      <a-form-item
        label="备注"
        name="remark"
      >
        <a-input
          v-model:value="form.remark"
          placeholder="选填"
        />
      </a-form-item>
    </a-form>

    <!-- 试渲染 -->
    <a-divider orientation="left">
      试渲染
    </a-divider>
    <div
      v-if="placeholderList.length"
      class="preview-vars"
    >
      <div
        v-for="key in placeholderList"
        :key="key"
        class="preview-var-row"
      >
        <span class="preview-var-key">{{ '${' + key + '}' }}</span>
        <a-input
          v-model:value="previewVars[key]"
          :placeholder="'请输入 ' + key + ' 的示例值'"
          size="small"
        />
      </div>
      <a-button
        size="small"
        :loading="previewing"
        @click="doPreview"
      >
        生成预览
      </a-button>
    </div>
    <a-empty
      v-else
      description="正文中尚未使用 ${变量} 占位"
      :image-style="{ height: '48px' }"
    />

    <div
      v-if="previewResult"
      class="preview-result"
    >
      <div class="preview-result-title">
        {{ previewResult.title || '（无标题）' }}
      </div>
      <pre class="preview-result-body">{{ previewResult.content }}</pre>
      <a-alert
        v-if="previewResult.missingVars.length"
        type="warning"
        show-icon
        :message="'以下占位符未提供变量：' + previewResult.missingVars.join('、')"
      />
    </div>
  </a-modal>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { message } from 'ant-design-vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import { useUserStore } from '@/stores/user'
import { useFitTableHeight } from '@/utils/useFitTableHeight'
import {
  notificationTemplateApi,
  type NotificationTemplateVo,
  type NotificationTemplateSaveRequest,
  type NotificationTemplatePreviewVo,
} from '@/api/system/notificationTemplate'

const userStore = useUserStore()
const canEdit = computed(() => userStore.hasPermission('notification:template:edit'))

const { fitRef, fitY, recompute } = useFitTableHeight()

const loading = ref(false)
const data = ref<NotificationTemplateVo[]>([])
const pagination = reactive({
  current: 1,
  pageSize: 10,
  total: 0,
  showSizeChanger: true,
  showTotal: (total: number) => `共 ${total} 条`,
})

const search = reactive<{
  // templateCode：界面「模板编码」输入框，发送给后端 NotificationTemplateQuery.templateCode，筛选 auth_sys_notification_template.template_code 模糊 contains
  templateCode?: string
  // templateName：界面「模板名称」输入框，发送给后端 NotificationTemplateQuery.templateName，筛选 auth_sys_notification_template.template_name 模糊 contains
  templateName?: string
  // notifyType：界面「通知类型」下拉框，发送给后端 NotificationTemplateQuery.notifyType，筛选 auth_sys_notification_template.notify_type 精确 eq
  notifyType?: string
  // channel：界面「渠道」下拉框，发送给后端 NotificationTemplateQuery.channel，筛选 auth_sys_notification_template.channel 精确 eq
  channel?: string
  // status：界面「状态」下拉框，发送给后端 NotificationTemplateQuery.status，筛选 auth_sys_notification_template.status 精确 eq
  status?: number
}>({})

const notifyTypeOptions = [
  { label: '系统', value: 'system' },
  { label: '实名认证', value: 'kyc' },
  { label: '充值', value: 'recharge' },
  { label: '发票', value: 'invoice' },
  { label: '团队', value: 'team' },
  { label: '用户', value: 'user' },
]

const channelOptions = [
  { label: '站内信', value: 'in_app' },
  { label: '邮件', value: 'email' },
]

const statusOptions = [
  { label: '启用', value: 1 },
  { label: '停用', value: 0 },
]

const columns = [
  { title: '模板编码', dataIndex: 'templateCode', width: 190 },
  { title: '模板名称', dataIndex: 'templateName', width: 150 },
  { title: '通知类型', dataIndex: 'notifyType', width: 110 },
  { title: '渠道', dataIndex: 'channel', width: 90 },
  { title: '正文模板', dataIndex: 'contentTemplate', width: 320, ellipsis: true },
  { title: '可用变量', dataIndex: 'varsHint', width: 150 },
  { title: '状态', dataIndex: 'status', width: 100 },
  { title: '更新时间', dataIndex: 'updateTime', width: 170 },
  { title: '操作', dataIndex: 'action', width: 120, fixed: 'right' as const },
]

function channelLabel(v: string) {
  return v === 'email' ? '邮件' : '站内信'
}

async function fetchData() {
  loading.value = true
  try {
    const res = await notificationTemplateApi.list({
      ...search,
      page: pagination.current,
      pageSize: pagination.pageSize,
    })
    data.value = res.items
    pagination.total = res.total
    recompute()
  } finally {
    loading.value = false
  }
}

function resetSearch() {
  search.templateCode = undefined
  search.templateName = undefined
  search.notifyType = undefined
  search.channel = undefined
  search.status = undefined
  pagination.current = 1
  fetchData()
}

function onTableChange(pag: { current?: number; pageSize?: number }) {
  pagination.current = pag.current || 1
  pagination.pageSize = pag.pageSize || 10
  fetchData()
}

// ---------------- 新增 / 编辑 ----------------

const modalVisible = ref(false)
const saving = ref(false)
const formRef = ref<FormInstance>()

const emptyForm = (): NotificationTemplateSaveRequest & { id?: string } => ({
  id: undefined,
  templateCode: '',
  templateName: '',
  notifyType: 'system',
  channel: 'in_app',
  titleTemplate: '',
  contentTemplate: '',
  varsHint: '',
  remark: '',
})

const form = reactive(emptyForm())

const rules: Record<string, Rule[]> = {
  templateCode: [{ required: true, message: '请输入模板编码', trigger: 'blur' }],
  templateName: [{ required: true, message: '请输入模板名称', trigger: 'blur' }],
  notifyType: [{ required: true, message: '请选择通知类型', trigger: 'change' }],
  channel: [{ required: true, message: '请选择渠道', trigger: 'change' }],
  contentTemplate: [{ required: true, message: '请输入正文模板', trigger: 'blur' }],
}

function openModal(record?: NotificationTemplateVo) {
  Object.assign(form, emptyForm())
  previewVars.value = {}
  previewResult.value = undefined
  if (record) {
    Object.assign(form, {
      id: record.id,
      templateCode: record.templateCode,
      templateName: record.templateName,
      notifyType: record.notifyType,
      channel: record.channel,
      titleTemplate: record.titleTemplate || '',
      contentTemplate: record.contentTemplate,
      varsHint: record.varsHint || '',
      remark: record.remark || '',
    })
  }
  modalVisible.value = true
}

async function handleSave() {
  try {
    await formRef.value?.validate()
  } catch {
    return
  }
  saving.value = true
  try {
    const payload: NotificationTemplateSaveRequest = {
      templateCode: form.templateCode,
      templateName: form.templateName,
      notifyType: form.notifyType,
      channel: form.channel,
      titleTemplate: form.titleTemplate || undefined,
      contentTemplate: form.contentTemplate,
      varsHint: form.varsHint || undefined,
      remark: form.remark || undefined,
    }
    if (form.id) {
      await notificationTemplateApi.update(form.id, payload)
      message.success('已保存')
    } else {
      await notificationTemplateApi.create(payload)
      message.success('已新增')
    }
    modalVisible.value = false
    fetchData()
  } finally {
    saving.value = false
  }
}

async function toggleStatus(record: NotificationTemplateVo, checked: boolean) {
  const next = checked ? 1 : 0
  try {
    await notificationTemplateApi.setStatus(record.id, next)
    record.status = next
    message.success(next === 1 ? '已启用' : '已停用')
  } catch {
    // 失败保持原状态
  }
}

async function doDelete(record: NotificationTemplateVo) {
  await notificationTemplateApi.remove(record.id)
  message.success('已删除')
  fetchData()
}

// ---------------- 试渲染 ----------------

const previewVars = ref<Record<string, string>>({})
const previewing = ref(false)
const previewResult = ref<NotificationTemplatePreviewVo>()

/** 从模板文本中提取 ${xxx} 占位符（去重、保持出现顺序） */
function extractPlaceholders(tpl?: string): string[] {
  const out: string[] = []
  if (!tpl) return out
  const re = /\$\{([^}]+)\}/g
  let m: RegExpExecArray | null
  while ((m = re.exec(tpl)) !== null) {
    if (!out.includes(m[1])) out.push(m[1])
  }
  return out
}

/** 标题 + 正文中出现的全部占位符 */
const placeholderList = computed(() => {
  const list = extractPlaceholders(form.titleTemplate)
  for (const key of extractPlaceholders(form.contentTemplate)) {
    if (!list.includes(key)) list.push(key)
  }
  return list
})

async function doPreview() {
  previewing.value = true
  try {
    previewResult.value = await notificationTemplateApi.preview({
      titleTemplate: form.titleTemplate || undefined,
      contentTemplate: form.contentTemplate,
      vars: { ...previewVars.value },
    })
  } finally {
    previewing.value = false
  }
}

onMounted(fetchData)
</script>

<style scoped>
.cell-ellipsis {
  display: inline-block;
  max-width: 300px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  vertical-align: bottom;
}

.danger-link {
  color: #ff4d4f;
}

.preview-vars {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 12px;
}

.preview-var-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.preview-var-key {
  flex: 0 0 160px;
  font-family: monospace;
  color: #1890ff;
}

.preview-result {
  margin-top: 12px;
  padding: 12px;
  background: #fafafa;
  border: 1px solid #f0f0f0;
  border-radius: 6px;
}

.preview-result-title {
  font-weight: 600;
  margin-bottom: 8px;
}

.preview-result-body {
  margin: 0 0 8px;
  white-space: pre-wrap;
  word-break: break-all;
  font-family: inherit;
}
</style>
