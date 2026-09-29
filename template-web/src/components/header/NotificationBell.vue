<template>
  <a-popover
    v-model:open="visible"
    trigger="click"
    placement="bottomRight"
  >
    <template #content>
      <div class="notify-panel">
        <div class="notify-head">
          <span class="notify-title">{{ t('notify.title') }}</span>
          <a-button
            v-if="isAdmin"
            type="link"
            size="small"
            @click="broadcastVisible = true"
          >
            {{ t('notify.broadcast') }}
          </a-button>
        </div>
        <a-spin :spinning="loading">
          <div class="notify-list">
            <div
              v-for="item in notifications"
              :key="item.id"
              class="notify-item"
              @click="markRead(item)"
            >
              <a-badge
                :dot="item.readFlag === 0"
                :offset="[4, 0]"
              >
                <span class="notify-item-title">{{ item.title }}</span>
              </a-badge>
              <div class="notify-item-content">
                {{ item.content }}
              </div>
              <div class="notify-item-time">
                {{ item.createTime }}
              </div>
            </div>
            <a-empty
              v-if="!loading && notifications.length === 0"
              :description="t('notify.empty')"
              :image-style="{ height: '48px' }"
            />
          </div>
        </a-spin>
        <div class="notify-foot">
          <a-button
            size="small"
            @click="markAllRead"
          >
            {{ t('notify.markAll') }}
          </a-button>
          <a @click="viewAll">{{ t('notify.viewAll') }}</a>
        </div>
      </div>
    </template>
    <span class="header-action">
      <a-badge
        :count="unreadCount"
        size="small"
        :overflow-count="99"
      >
        <BellOutlined />
      </a-badge>
    </span>
  </a-popover>

  <a-modal
    v-model:open="broadcastVisible"
    :title="t('notify.broadcastTitle')"
    :confirm-loading="broadcastSubmitting"
    @ok="submitBroadcast"
  >
    <a-form layout="vertical">
      <a-form-item
        :label="t('notify.titleLabel')"
        required
      >
        <a-input
          v-model:value="broadcastForm.title"
          :maxlength="120"
        />
      </a-form-item>
      <a-form-item
        :label="t('notify.contentLabel')"
        required
      >
        <a-textarea
          v-model:value="broadcastForm.content"
          :rows="4"
          :maxlength="500"
          show-count
        />
      </a-form-item>
    </a-form>
    <a-alert
      type="warning"
      show-icon
      :message="t('notify.broadcastTip')"
    />
  </a-modal>
</template>

<script setup lang="ts">
import { computed, onUnmounted, reactive, ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import { message } from 'ant-design-vue'
import { BellOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import {
  systemNotificationApi,
  type SystemNotificationVo,
} from '@/api/system/notification'
import { useUserStore } from '@/stores/user'

const { t } = useI18n()
const router = useRouter()
const userStore = useUserStore()

// 仅管理员可发广播
const isAdmin = computed(() => userStore.roles.includes('admin'))

const visible = ref(false)
const loading = ref(false)
const notifications = ref<SystemNotificationVo[]>([])
const unreadCount = ref(0)

async function fetchUnreadCount() {
  try {
    unreadCount.value = await systemNotificationApi.unreadCount()
  } catch {
    // 轮询失败静默忽略
  }
}

async function fetchList() {
  loading.value = true
  try {
    const res = await systemNotificationApi.list({ page: 1, pageSize: 10 })
    notifications.value = res.items
  } catch {
    // 面板内有空态兜底
  } finally {
    loading.value = false
  }
}

watch(visible, (v) => {
  if (v) fetchList()
})

function viewAll() {
  visible.value = false
  router.push('/system/notification')
}

async function markRead(item: SystemNotificationVo) {
  if (item.readFlag !== 0) return
  try {
    await systemNotificationApi.markRead(item.id)
    item.readFlag = 1
    fetchUnreadCount()
  } catch {
    /* handled */
  }
}

async function markAllRead() {
  try {
    await systemNotificationApi.markAllRead()
    fetchList()
    fetchUnreadCount()
  } catch {
    /* handled */
  }
}

const broadcastVisible = ref(false)
const broadcastSubmitting = ref(false)
const broadcastForm = reactive({ title: '', content: '' })

async function submitBroadcast() {
  if (!broadcastForm.title.trim() || !broadcastForm.content.trim()) {
    message.warning(t('notify.broadcastRequired'))
    return
  }
  broadcastSubmitting.value = true
  try {
    await systemNotificationApi.broadcast({
      notifyType: 'system',
      channel: 'in_app',
      title: broadcastForm.title.trim(),
      content: broadcastForm.content.trim(),
    })
    message.success(t('notify.broadcastSent'))
    broadcastVisible.value = false
    broadcastForm.title = ''
    broadcastForm.content = ''
  } catch {
    message.error(t('notify.broadcastFailed'))
  } finally {
    broadcastSubmitting.value = false
  }
}

// 每 60s 轮询未读数，保证角标及时更新
const timer = window.setInterval(fetchUnreadCount, 60_000)
onUnmounted(() => window.clearInterval(timer))
fetchUnreadCount()
</script>

<style scoped>
.notify-panel {
  width: 340px;
}

.notify-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border-color, #f0f0f0);
}

.notify-title {
  font-weight: 600;
  color: var(--header-text, rgba(0, 0, 0, 0.85));
}

.notify-list {
  max-height: 320px;
  overflow-y: auto;
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.notify-item {
  padding: 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.3s;
}

.notify-item:hover {
  background: var(--header-hover-bg, rgba(0, 0, 0, 0.04));
}

.notify-item-title {
  font-size: 13px;
  font-weight: 500;
  color: var(--header-text, rgba(0, 0, 0, 0.85));
}

.notify-item-content {
  color: var(--header-text-secondary, rgba(0, 0, 0, 0.65));
  font-size: 12px;
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.notify-item-time {
  color: var(--header-text-secondary, rgba(0, 0, 0, 0.65));
  font-size: 11px;
  margin-top: 2px;
  opacity: 0.7;
}

.notify-foot {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 24px;
  border-top: 1px solid var(--border-color, #f0f0f0);
  margin-top: 8px;
  padding-top: 8px;
}
</style>