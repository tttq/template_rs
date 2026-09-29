<template>
  <div class="setup-container">
    <div class="setup-wrapper">
      <div class="setup-brand">
        <svg
          viewBox="0 0 40 40"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          class="brand-mark"
        >
          <rect width="40" height="40" rx="10" fill="rgba(102,126,234,0.18)" />
          <path
            d="M12 20L18 26L28 14"
            stroke="#667eea"
            stroke-width="3"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <h1 class="brand-title">
          完善账户信息
        </h1>
        <p class="brand-subtitle">
          首次使用请先补全账号信息并选择身份，完成后即可进入系统
        </p>
      </div>

      <a-card
        class="setup-card"
        :bordered="false"
      >
        <a-form
          ref="formRef"
          :model="form"
          :rules="rules"
          layout="vertical"
          class="setup-form"
          @finish="onSubmit"
        >
          <a-form-item
            name="userName"
            label="用户名"
          >
            <a-input
              v-model:value="form.userName"
              placeholder="3-50 位字母、数字或下划线"
              size="large"
              allow-clear
            >
              <template #prefix>
                <UserOutlined class="input-icon" />
              </template>
            </a-input>
          </a-form-item>

          <a-form-item
            name="passWord"
            label="密码"
          >
            <a-input-password
              v-model:value="form.passWord"
              placeholder="8-64 位"
              size="large"
            >
              <template #prefix>
                <LockOutlined class="input-icon" />
              </template>
            </a-input-password>
          </a-form-item>

          <a-form-item
            name="confirmPassWord"
            label="确认密码"
          >
            <a-input-password
              v-model:value="form.confirmPassWord"
              placeholder="再次输入密码"
              size="large"
            >
              <template #prefix>
                <LockOutlined class="input-icon" />
              </template>
            </a-input-password>
          </a-form-item>

          <a-form-item
            name="phone"
            label="手机号"
          >
            <a-input
              v-model:value="form.phone"
              placeholder="请输入手机号"
              size="large"
              allow-clear
            >
              <template #prefix>
                <PhoneOutlined class="input-icon" />
              </template>
            </a-input>
          </a-form-item>

          <a-form-item
            name="email"
            label="邮箱"
          >
            <a-input
              v-model:value="form.email"
              placeholder="邮箱地址（用于接收验证码）"
              size="large"
              allow-clear
            >
              <template #prefix>
                <MailOutlined class="input-icon" />
              </template>
            </a-input>
          </a-form-item>

          <a-form-item
            name="captchaCode"
            label="图形验证码"
          >
            <div class="captcha-row">
              <a-input
                v-model:value="form.captchaCode"
                placeholder="请输入图形验证码"
                size="large"
                class="captcha-input"
              >
                <template #prefix>
                  <SafetyOutlined class="input-icon" />
                </template>
              </a-input>
              <img
                v-if="captchaImage"
                :src="captchaImage"
                alt="图形验证码"
                title="看不清？点击刷新"
                class="captcha-img"
                @click="loadCaptcha"
              >
            </div>
          </a-form-item>

          <a-form-item
            name="emailCode"
            label="邮箱验证码"
          >
            <a-input-search
              v-model:value="form.emailCode"
              placeholder="请输入邮箱验证码"
              size="large"
              :enter-button="sendBtnText"
              :loading="sendLoading"
              @search="sendCode"
            />
          </a-form-item>

          <a-form-item
            name="nickName"
            label="昵称"
          >
            <a-input
              v-model:value="form.nickName"
              placeholder="昵称（选填，最多 30 字）"
              size="large"
              allow-clear
            />
          </a-form-item>

          <a-form-item
            name="roleCode"
            label="身份"
          >
            <a-select
              v-model:value="form.roleCode"
              :options="roleOptions"
              placeholder="选择身份（数据来自数据字典）"
              size="large"
              :loading="rolesLoading"
            />
          </a-form-item>

          <a-form-item>
            <a-button
              type="primary"
              html-type="submit"
              :loading="submitting"
              block
              size="large"
            >
              完成并进入系统
            </a-button>
          </a-form-item>
        </a-form>

        <div class="setup-footer">
          <a @click="onLogout">退出登录</a>
        </div>
      </a-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted, reactive, ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { message } from 'ant-design-vue'
import type { FormInstance, Rule } from 'ant-design-vue/es/form'
import { UserOutlined, LockOutlined, PhoneOutlined, MailOutlined, SafetyOutlined } from '@ant-design/icons-vue'
import { authApi } from '@/api/auth'
import { useUserStore } from '@/stores/user'

/**
 * 完善账户信息页（微信自动注册的惰性账号首次进入系统前必填）
 *
 * 服务端闸口：未完善账号零角色，除 `sa_check_login` 端点外一律 403；
 * 客户端闸口：路由守卫发现 `profileCompleted === false` 即强制跳本页。
 *
 * 邮箱 + 邮箱验证码为必填：与注册同口径，服务端在建号/完善前一次性校验验证码。
 */
const router = useRouter()
const userStore = useUserStore()

const formRef = ref<FormInstance>()
const submitting = ref(false)
const rolesLoading = ref(false)
const roleOptions = ref<{ label: string; value: string }[]>([])

/** 图形验证码：captchaId 提交回传；image 为 PNG data URI（点击可刷新） */
const captchaId = ref('')
const captchaImage = ref('')
const sendLoading = ref(false)
const countdown = ref(0)
const sendBtnText = computed(() =>
  countdown.value > 0 ? `${countdown.value}s 后重发` : '发送验证码',
)

const form = reactive({
  userName: '',
  passWord: '',
  confirmPassWord: '',
  phone: '',
  email: '',
  captchaCode: '',
  emailCode: '',
  nickName: '',
  roleCode: undefined as string | undefined,
})

const rules: Record<string, Rule[]> = {
  userName: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { pattern: /^[A-Za-z0-9_]{3,50}$/, message: '3-50 位字母、数字或下划线', trigger: 'blur' },
  ],
  passWord: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 8, max: 64, message: '密码需为 8-64 位', trigger: 'blur' },
  ],
  confirmPassWord: [
    { required: true, message: '请再次输入密码', trigger: 'blur' },
    {
      validator: (_: Rule, value: string) => {
        if (!value) return Promise.resolve()
        return value === form.passWord ? Promise.resolve() : Promise.reject('两次输入的密码不一致')
      },
      trigger: 'blur',
    },
  ],
  phone: [
    { required: true, message: '请输入手机号', trigger: 'blur' },
    { pattern: /^1[3-9]\d{9}$/, message: '手机号格式不正确', trigger: 'blur' },
  ],
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '请输入有效的邮箱地址', trigger: 'blur' },
  ],
  captchaCode: [{ required: true, message: '请输入图形验证码', trigger: 'blur' }],
  emailCode: [{ required: true, message: '请输入邮箱验证码', trigger: 'blur' }],
  nickName: [{ max: 30, message: '昵称不能超过 30 个字符', trigger: 'blur' }],
  roleCode: [{ required: true, message: '请选择身份', trigger: 'change' }],
}

/** 获取/刷新图形验证码（发送邮箱验证码的前置校验） */
async function loadCaptcha() {
  try {
    const data = await authApi.captcha()
    captchaId.value = data.captchaId
    captchaImage.value = data.image
    form.captchaCode = ''
  } catch {
    captchaImage.value = ''
    message.warning('图形验证码加载失败，请点击图片或稍后重试')
  }
}

function startCountdown() {
  countdown.value = 60
  const timer = setInterval(() => {
    countdown.value--
    if (countdown.value <= 0) clearInterval(timer)
  }, 1000)
}

/** 发送邮箱验证码（scene=register，与注册共用同一验证码空间） */
async function sendCode() {
  if (!form.email || !form.email.includes('@')) {
    message.warning('请先输入正确的邮箱')
    return
  }
  if (countdown.value > 0) return
  if (!captchaId.value || !form.captchaCode) {
    if (!captchaImage.value) await loadCaptcha()
    message.warning('请先输入图形验证码')
    return
  }
  sendLoading.value = true
  try {
    await authApi.sendCode({
      email: form.email,
      scene: 'register',
      captchaId: captchaId.value,
      captchaCode: form.captchaCode,
    })
    message.success('验证码已发送，请查收邮件')
    startCountdown()
    // 图形码一次性消费，发送成功后刷新下一张
    await loadCaptcha()
  } catch {
    await loadCaptcha()
  } finally {
    sendLoading.value = false
  }
}

onMounted(async () => {
  // 已完善则不该停在本页
  if (userStore.userInfo && userStore.userInfo.profileCompleted !== false) {
    router.replace('/')
    return
  }
  rolesLoading.value = true
  try {
    const list = await authApi.registerRoles()
    roleOptions.value = (list || []).map((r) => ({ label: r.label, value: r.value }))
  } catch {
    roleOptions.value = []
  } finally {
    rolesLoading.value = false
  }
  loadCaptcha()
})

async function onSubmit() {
  if (!form.roleCode) return
  submitting.value = true
  try {
    await authApi.completeProfile({
      userName: form.userName,
      passWord: form.passWord,
      phone: form.phone,
      email: form.email,
      emailCode: form.emailCode,
      nickName: form.nickName || undefined,
      roleCode: form.roleCode,
    })
    // 角色与菜单在提交后才产生：刷新 store（含 profileCompleted 与新菜单）
    await userStore.getUserInfo()
    message.success('已完善，欢迎使用')
    router.replace('/')
  } finally {
    submitting.value = false
  }
}

async function onLogout() {
  await userStore.logout()
  router.replace('/login')
}
</script>

<style scoped>
.setup-container {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  padding: 24px;
  background: linear-gradient(135deg, #0f0c29 0%, #302b63 50%, #24243e 100%);
}

.setup-wrapper {
  position: relative;
  z-index: 1;
  width: 100%;
  max-width: 460px;
}

.setup-brand {
  text-align: center;
  margin-bottom: 24px;
}

.brand-mark {
  display: block;
  width: 52px;
  height: 52px;
  margin: 0 auto 12px;
}

.brand-title {
  margin: 0 0 8px;
  font-size: 26px;
  font-weight: 600;
  color: #fff;
}

.brand-subtitle {
  margin: 0;
  font-size: 14px;
  line-height: 1.6;
  color: rgba(255, 255, 255, 0.72);
}

.setup-card {
  border-radius: 16px;
}

.setup-footer {
  margin-top: 8px;
  text-align: center;
}

/* 图形验证码行内布局 */
.captcha-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.captcha-input {
  flex: 1;
  min-width: 0;
}

.captcha-img {
  width: 110px;
  height: 40px;
  flex-shrink: 0;
  cursor: pointer;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.08);
}

.input-icon {
  color: inherit;
  opacity: 0.45;
}
</style>
