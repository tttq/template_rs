<template>
  <a-modal
    :open="open"
    :title="action === 'bind' ? '扫码绑定微信' : '微信扫码登录'"
    :footer="null"
    :width="420"
    :centered="true"
    destroy-on-close
    @cancel="close"
  >
    <div class="qr-body">
      <div
        v-if="loading"
        class="qr-state"
      >
        <a-spin />
        <p class="qr-hint">
          正在生成二维码…
        </p>
      </div>
      <div
        v-else-if="errorMsg"
        class="qr-state"
      >
        <a-alert
          type="warning"
          show-icon
          :message="errorMsg"
        />
        <a-button
          class="qr-retry"
          type="primary"
          ghost
          @click="load"
        >
          重新获取
        </a-button>
      </div>
      <template v-else>
        <!-- qrconnect 返回的是网页（自带二维码），不能直接渲染成二维码图片 -->
        <iframe
          :src="qrUrl"
          class="qr-frame"
          frameborder="0"
          scrolling="no"
          sandbox="allow-scripts allow-same-origin allow-top-navigation allow-popups"
        />
        <p class="qr-hint">
          {{ action === 'bind' ? '请使用微信扫码完成绑定' : '请使用微信扫一扫登录' }}
        </p>
        <a-button
          class="qr-retry"
          type="link"
          size="small"
          @click="load"
        >
          刷新二维码
        </a-button>
      </template>
    </div>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { authApi } from '@/api/auth'

/**
 * 微信扫码弹窗（登录与绑定复用）
 *
 * 打开时向后端要 qrconnect 地址（后端同时把一次性 CSRF state 写入 Redis，300s），
 * 弹窗内 iframe 加载该页面；用户扫码确认后微信跳转到 redirect_uri
 * （`/login/wechat/callback`），由回调页把 code + state 交给后端换 token 或完成绑定。
 */
const props = withDefaults(
  defineProps<{
    open: boolean
    action?: 'login' | 'bind'
  }>(),
  { action: 'login' },
)

const emit = defineEmits<{
  (e: 'update:open', value: boolean): void
}>()

const loading = ref(false)
const errorMsg = ref('')
const qrUrl = ref('')

/** 回调页据此判断是登录还是绑定（redirect_uri 只需登记域名，无需两个地址） */
const ACTION_KEY = 'wx_qr_action'

async function load() {
  loading.value = true
  errorMsg.value = ''
  qrUrl.value = ''
  try {
    sessionStorage.setItem(ACTION_KEY, props.action)
    const res = await authApi.wechatQrUrl()
    qrUrl.value = res.url
  } catch (err: any) {
    // 未配置开放平台时后端返回明确中文错误
    errorMsg.value = err?.message || '二维码获取失败，请稍后重试'
  } finally {
    loading.value = false
  }
}

function close() {
  emit('update:open', false)
}

watch(
  () => props.open,
  (val) => {
    if (val) {
      load()
    } else {
      qrUrl.value = ''
      errorMsg.value = ''
    }
  },
)
</script>

<style scoped>
.qr-body {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  min-height: 340px;
  padding: 8px 0;
}

.qr-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.qr-frame {
  width: 320px;
  height: 320px;
  border: 1px solid var(--border-color, #f0f0f0);
  border-radius: 8px;
  background: #fff;
}

.qr-hint {
  margin: 12px 0 0;
  font-size: 13px;
  color: var(--text-muted, #8a94a6);
}

.qr-retry {
  margin-top: 8px;
}
</style>
