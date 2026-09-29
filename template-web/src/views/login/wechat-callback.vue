<template>
  <div class="wx-callback">
    <a-spin
      v-if="status === 'working'"
      size="large"
    />
    <a-result
      v-else-if="status === 'error'"
      status="warning"
      title="微信登录失败"
      :sub-title="errorMsg"
    >
      <template #extra>
        <a-button
          type="primary"
          @click="goLogin"
        >
          返回登录
        </a-button>
      </template>
    </a-result>
    <a-result
      v-else
      status="success"
      title="操作成功"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { message } from 'ant-design-vue'
import { authApi } from '@/api/auth'
import { useUserStore } from '@/stores/user'

/**
 * Web 微信扫码回调页
 *
 * - 在 iframe 内时（qrconnect 的 self_redirect 会在弹窗 iframe 里跳转）：
 *   把当前完整 URL 交给顶层窗口，跳出 iframe 后重新执行本页逻辑
 * - 顶层：把 code + state 交给后端换 token（登录）或完成绑定
 *
 * action 由发起方（WechatQrModal）写入 sessionStorage —— redirect_uri 只需与
 * 开放平台登记的授权回调域一致，无需为 bind 单独登记一个地址。
 */
const route = useRoute()
const router = useRouter()
const userStore = useUserStore()

const status = ref<'working' | 'success' | 'error'>('working')
const errorMsg = ref('')

const ACTION_KEY = 'wx_qr_action'

function goLogin() {
  router.replace('/login')
}

onMounted(async () => {
  // 1) iframe 内：跳到顶层重新执行（顶层才能安全读写 sessionStorage 与跳路由）
  if (window.self !== window.top) {
    try {
      window.top?.location.replace(window.location.href)
    } catch {
      // 跨域受限时无法逃出，提示用户
      status.value = 'error'
      errorMsg.value = '请在浏览器窗口中完成微信登录'
    }
    return
  }

  const code = typeof route.query.code === 'string' ? route.query.code : ''
  const state = typeof route.query.state === 'string' ? route.query.state : ''
  const action = sessionStorage.getItem(ACTION_KEY) === 'bind' ? 'bind' : 'login'
  sessionStorage.removeItem(ACTION_KEY)

  if (!code || !state) {
    status.value = 'error'
    errorMsg.value = '缺少授权参数，请重新扫码'
    return
  }

  try {
    if (action === 'bind') {
      await authApi.bind({ provider: 'wechat', code, qrState: state })
      message.success('微信绑定成功')
      status.value = 'success'
      router.replace('/profile')
      return
    }

    const token = await authApi.login({ passWord: '', loginType: 'wechat', code, qrState: state })
    userStore.setTokenData({
      token: token.token,
      refreshToken: token.refreshToken,
      expireTime: token.expireTime,
      refreshExpireTime: token.refreshExpireTime,
    })
    await userStore.getUserInfo()
    status.value = 'success'
    // 未完善（微信自动注册的惰性账号）→ 先完善账户信息
    const info = userStore.userInfo
    router.replace(info && info.profileCompleted === false ? '/profile-setup' : '/')
  } catch (err: any) {
    status.value = 'error'
    errorMsg.value = err?.message || '微信登录失败，请重试'
  }
})
</script>

<style scoped>
.wx-callback {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  background: var(--bg-body, #f5f6fa);
}
</style>
