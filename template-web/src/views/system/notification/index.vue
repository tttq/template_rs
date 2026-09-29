<template>
  <a-card :bordered="false">
    <template #extra>
      <a-space>
        <a-badge :count="unreadCount">
          <span />
        </a-badge>
        <a-radio-group
          v-model:value="query.readFlag"
          button-style="solid"
          @change="fetchData"
        >
          <a-radio-button :value="0">
            未读
          </a-radio-button>
          <a-radio-button :value="1">
            已读
          </a-radio-button>
          <a-radio-button :value="undefined">
            全部
          </a-radio-button>
        </a-radio-group>
        <a-button @click="markAll">
          全部已读
        </a-button>
        <a-button
          v-if="userStore.hasPermission('notification:send')"
          @click="openSend"
        >
          发送通知
        </a-button>
        <a-button
          v-if="userStore.hasPermission('notification:broadcast')"
          type="primary"
          @click="openBroadcast"
        >
          发送广播
        </a-button>
      </a-space>
    </template>

    <a-list
      :loading="loading"
      :data-source="notifications"
      item-layout="horizontal"
      :pagination="false"
    >
      <template #renderItem="{ item }">
        <a-list-item>
          <a-list-item-meta>
            <template #avatar>
              <a-badge :dot="item.readFlag === 0">
                <a-avatar style="background-color: #1677ff">
                  {{ typeIcon(item.notifyType) }}
                </a-avatar>
              </a-badge>
            </template>
            <template #title>
              <span :style="{ fontWeight: item.readFlag === 0 ? 600 : 400 }">{{ item.title }}</span>
              <span style="color: #999; font-size: 12px; margin-left: 12px">{{ item.createTime }}</span>
            </template>
            <template #description>
              {{ item.content }}
            </template>
          </a-list-item-meta>
          <template #actions>
            <a
              v-if="item.readFlag === 0"
              @click="markRead(item.id)"
            >标记已读</a>
          </template>
        </a-list-item>
      </template>
      <template #footer>
        <a-pagination
          v-if="total > pageSize"
          :current="page"
          :page-size="pageSize"
          :total="total"
          :show-total="(t: number) => `共 ${t} 条`"
          style="text-align: center"
          @change="onPageChange"
        />
      </template>
    </a-list>

    <!-- 定向发送 -->
    <a-modal
      v-model:open="sendVisible"
      title="发送通知"
      :confirm-loading="submitting"
      @ok="submitSend"
    >
      <a-form
        layout="vertical"
        :model="sendForm"
      >
        <a-form-item
          label="接收用户"
          help="支持多选（可按用户名 / 昵称搜索）；留空则不指定接收人，仅落库"
        >
          <a-select
            v-model:value="sendForm.userIds"
            mode="multiple"
            show-search
            allow-clear
            placeholder="搜索用户名 / 昵称选择接收人"
            :filter-option="false"
            :options="userOptions"
            :loading="userLoading"
            :max-tag-count="4"
            @search="onUserSearch"
          />
        </a-form-item>
        <a-form-item
          label="通知类型"
          required
        >
          <a-select
            v-model:value="sendForm.notifyType"
            :options="NOTIFY_TYPES"
          />
        </a-form-item>
        <a-form-item
          label="推送渠道"
          required
        >
          <a-select
            v-model:value="sendForm.channel"
            :options="CHANNELS"
          />
        </a-form-item>
        <a-form-item
          label="标题"
          required
        >
          <a-input
            v-model:value="sendForm.title"
            :maxlength="120"
            placeholder="通知标题"
          />
        </a-form-item>
        <a-form-item
          label="内容"
          required
        >
          <a-textarea
            v-model:value="sendForm.content"
            :rows="4"
            :maxlength="500"
            show-count
            placeholder="通知内容"
          />
        </a-form-item>
      </a-form>
    </a-modal>

    <!-- 全站广播 -->
    <a-modal
      v-model:open="broadcastVisible"
      title="发送全站广播"
      :confirm-loading="submitting"
      @ok="submitBroadcast"
    >
      <a-form
        layout="vertical"
        :model="broadcastForm"
      >
        <a-form-item
          label="通知类型"
          required
        >
          <a-select
            v-model:value="broadcastForm.notifyType"
            :options="NOTIFY_TYPES"
          />
        </a-form-item>
        <a-form-item
          label="推送渠道"
          required
        >
          <a-select
            v-model:value="broadcastForm.channel"
            :options="CHANNELS"
          />
        </a-form-item>
        <a-form-item
          label="标题"
          required
        >
          <a-input
            v-model:value="broadcastForm.title"
            :maxlength="120"
            placeholder="广播标题"
          />
        </a-form-item>
        <a-form-item
          label="内容"
          required
        >
          <a-textarea
            v-model:value="broadcastForm.content"
            :rows="4"
            :maxlength="500"
            show-count
            placeholder="广播内容（将写入全部用户站内通知）"
          />
        </a-form-item>
      </a-form>
      <a-alert
        type="warning"
        show-icon
        message="广播将写入全部用户的站内通知，请谨慎操作（操作会记录审计日志）"
      />
    </a-modal>
  </a-card>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { message } from 'ant-design-vue'
import { useUserStore } from '@/stores/user'
import { systemNotificationApi, type SystemNotificationVo } from '@/api/system/notification'

const userStore = useUserStore()
const loading = ref(false)
const notifications = ref<SystemNotificationVo[]>([])
const unreadCount = ref(0)
const page = ref(1)
const pageSize = 20
const total = ref(0)

// readFlag：界面「未读/已读/全部」单选过滤器，发送给后端 NotificationQuery.readFlag，精确 eq 筛选 auth_sys_notification.read_flag（0 未读 / 1 已读）
const query = reactive<{ readFlag?: number }>({ readFlag: undefined })

const NOTIFY_TYPES = [
  { label: '系统通知', value: 'system' },
  { label: '业务通知', value: 'business' },
  { label: '促销活动', value: 'promotion' },
  { label: '安全提醒', value: 'warning' },
]
const CHANNELS = [
  { label: '站内通知', value: 'system' },
  { label: '邮件', value: 'mail' },
]

const sendVisible = ref(false)
const broadcastVisible = ref(false)
const submitting = ref(false)
const sendForm = reactive<{ userIds: string[]; notifyType: string; channel: string; title: string; content: string }>({
  userIds: [],
  notifyType: 'system',
  channel: 'system',
  title: '',
  content: '',
})

// ---------------- 接收用户多选（远程搜索） ----------------
const userOptions = ref<{ label: string; value: string }[]>([])
const userLoading = ref(false)
let userSearchTimer = 0
/** 仅在首次加载失败时提示，避免搜索过程中反复弹错 */
let userLoadWarned = false

/** 拉取接收人候选：按关键字远程检索（留空取前 20 条） */
async function loadUserOptions(keyword = '') {
  userLoading.value = true
  try {
    const list = await systemNotificationApi.recipients({
      keyword: keyword.trim() || undefined,
      limit: 20,
    })
    userOptions.value = list.map((u) => ({
      label: u.nickName ? `${u.nickName}（${u.userName}）` : u.userName,
      value: u.id,
    }))
  } catch {
    userOptions.value = []
    if (!userLoadWarned) {
      userLoadWarned = true
      message.warning('接收人列表加载失败，请稍后重试')
    }
  } finally {
    userLoading.value = false
  }
}

/** 输入防抖，避免每次按键都发起请求 */
function onUserSearch(keyword: string) {
  window.clearTimeout(userSearchTimer)
  userSearchTimer = window.setTimeout(() => loadUserOptions(keyword), 300)
}
const broadcastForm = reactive({ notifyType: 'system', channel: 'system', title: '', content: '' })

function typeIcon(type: string) {
  const map: Record<string, string> = {
    system: '系',
    business: '业',
    promotion: '促',
    warning: '警',
  }
  return map[type] || '通'
}

async function loadUnread() {
  try {
    unreadCount.value = await systemNotificationApi.unreadCount()
  } catch {
    unreadCount.value = 0
  }
}

async function fetchData() {
  loading.value = true
  try {
    const params: Record<string, unknown> = { page: page.value, pageSize }
    if (query.readFlag !== undefined) params.readFlag = query.readFlag
    const res = await systemNotificationApi.list(params)
    notifications.value = res.items || []
    total.value = res.total || 0
  } finally {
    loading.value = false
  }
}

function onPageChange(p: number) {
  page.value = p
  fetchData()
}

async function markRead(id: string) {
  await systemNotificationApi.markRead(id)
  message.success('已读')
  loadUnread()
  fetchData()
}

async function markAll() {
  await systemNotificationApi.markAllRead()
  message.success('全部已读')
  loadUnread()
  fetchData()
}

function openSend() {
  sendVisible.value = true
  // 首次打开预加载用户选项（之后按关键字远程搜索）
  if (!userOptions.value.length) loadUserOptions()
}

function openBroadcast() {
  broadcastVisible.value = true
}

async function submitSend() {
  if (!sendForm.title.trim() || !sendForm.content.trim()) {
    message.warning('请填写标题与内容')
    return
  }
  submitting.value = true
  try {
    const res = await systemNotificationApi.send({
      userIds: sendForm.userIds.length ? sendForm.userIds : undefined,
      notifyType: sendForm.notifyType,
      channel: sendForm.channel,
      title: sendForm.title,
      content: sendForm.content,
    })
    // 后端返回 "已发送 N 条"，部分失败时走错误分支由拦截器提示
    message.success(typeof res === 'string' ? res : '发送成功')
    sendVisible.value = false
    sendForm.userIds = []
    sendForm.title = ''
    sendForm.content = ''
    fetchData()
  } finally {
    submitting.value = false
  }
}

async function submitBroadcast() {
  if (!broadcastForm.title.trim() || !broadcastForm.content.trim()) {
    message.warning('请填写标题与内容')
    return
  }
  submitting.value = true
  try {
    await systemNotificationApi.broadcast({
      notifyType: broadcastForm.notifyType,
      channel: broadcastForm.channel,
      title: broadcastForm.title,
      content: broadcastForm.content,
    })
    message.success('广播成功')
    broadcastVisible.value = false
    broadcastForm.title = ''
    broadcastForm.content = ''
  } finally {
    submitting.value = false
  }
}

onMounted(() => {
  loadUnread()
  fetchData()
})
</script>