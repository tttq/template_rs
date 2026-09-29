<template>
  <div class="login-container">
    <div class="login-bg">
      <div class="bg-shape shape-1" />
      <div class="bg-shape shape-2" />
      <div class="bg-shape shape-3" />
    </div>
    <div class="login-wrapper">
      <div class="login-brand">
        <div class="brand-mark">
          <svg viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect width="40" height="40" rx="10" fill="rgba(255,255,255,0.25)" />
            <path d="M12 20L18 26L28 14" stroke="white" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </div>
        <h1 class="brand-title">
          Template Admin
        </h1>
        <p class="brand-subtitle">
          企业级后台管理系统
        </p>
      </div>
      <a-card
        class="login-card"
        :bordered="false"
      >
        <a-tabs
          v-model:active-key="loginTab"
          centered
          class="login-tabs"
          @change="onLoginTabChange"
        >
          <a-tab-pane
            key="password"
            tab="密码登录"
          >
            <div
              v-if="loginStep === 'locate'"
              class="login-step-locate"
            >
              <div class="step-indicator">
                <span class="step-number active">1</span>
                <span class="step-line" />
                <span class="step-number">2</span>
              </div>
              <div class="step-title">
                步骤 1/2：输入用户名
              </div>
              <a-form
                :model="locateForm"
                layout="vertical"
                class="login-form"
                @finish="handleLocateTenant"
              >
                <a-form-item
                  name="userName"
                  label="用户名"
                  :rules="[{ required: true, message: '请输入用户名' }]"
                >
                  <a-input
                    v-model:value="locateForm.userName"
                    placeholder="请输入用户名"
                    size="large"
                    allow-clear
                  >
                    <template #prefix>
                      <UserOutlined class="input-icon" />
                    </template>
                  </a-input>
                </a-form-item>
                <a-form-item>
                  <a-button
                    type="primary"
                    html-type="submit"
                    :loading="locateLoading"
                    block
                    size="large"
                    class="login-btn"
                  >
                    下一步
                  </a-button>
                </a-form-item>
              </a-form>
              <div class="login-footer">
                <span class="footer-text">还没有账号？</span>
                <a
                  class="footer-link"
                  @click="openRegister"
                >注册账号</a>
              </div>
            </div>

            <div
              v-else
              class="login-step-login"
            >
              <div class="step-indicator">
                <span class="step-number">1</span>
                <span class="step-line active" />
                <span class="step-number active">2</span>
              </div>
              <div class="step-title">
                步骤 2/2：输入密码
              </div>
          
              <div
                v-if="tenantInfo"
                class="tenant-info"
              >
                <a-tag color="green">
                  {{ tenantInfo.tenantName }}
                </a-tag>
                <span class="tenant-hint">欢迎回来，{{ tenantInfo.userName }}</span>
              </div>

              <a-form
                :model="loginForm"
                layout="vertical"
                class="login-form"
                @finish="handleLogin"
              >
                <a-form-item label="用户名">
                  <a-input
                    :value="tenantInfo?.userName"
                    disabled
                    placeholder="用户名"
                    size="large"
                  >
                    <template #prefix>
                      <UserOutlined class="input-icon" />
                    </template>
                  </a-input>
                </a-form-item>
                <a-form-item
                  name="passWord"
                  label="密码"
                  :rules="[{ required: true, message: '请输入密码' }]"
                >
                  <a-input-password
                    v-model:value="loginForm.passWord"
                    placeholder="请输入密码"
                    size="large"
                  >
                    <template #prefix>
                      <LockOutlined class="input-icon" />
                    </template>
                  </a-input-password>
                </a-form-item>
                <a-form-item
                  v-if="needTotp"
                  label="动态验证码"
                >
                  <a-input
                    v-model:value="totpCode"
                    placeholder="请输入 6 位动态验证码（两步验证）"
                    size="large"
                    :maxlength="6"
                    allow-clear
                  >
                    <template #prefix>
                      <SafetyOutlined class="input-icon" />
                    </template>
                  </a-input>
                </a-form-item>
                <a-form-item>
                  <div class="login-options">
                    <a-checkbox v-model:checked="rememberMe">
                      {{ t('login.rememberMe') }}
                    </a-checkbox>
                  </div>
                </a-form-item>
                <a-form-item>
                  <a-button
                    type="primary"
                    html-type="submit"
                    :loading="loginLoading"
                    block
                    size="large"
                    class="login-btn"
                  >
                    登 录
                  </a-button>
                </a-form-item>
              </a-form>
              <div class="login-actions">
                <a
                  class="back-link"
                  @click="backToLocate"
                >返回上一步</a>
              </div>
            </div>
          </a-tab-pane>
          <a-tab-pane
            key="emailCode"
            tab="邮箱验证码登录"
          >
            <div class="email-code-login">
              <a-form
                :model="emailCodeForm"
                layout="vertical"
                class="login-form"
                @finish="handleEmailCodeLogin"
              >
                <a-form-item
                  name="email"
                  label="邮箱"
                  :rules="[{ required: true, message: '请输入邮箱' }, { type: 'email', message: '邮箱格式不正确' }]"
                >
                  <a-input
                    v-model:value="emailCodeForm.email"
                    placeholder="请输入邮箱"
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
                  :rules="[{ required: true, message: '请输入图形验证码' }]"
                >
                  <div class="captcha-row">
                    <a-input
                      v-model:value="emailCodeForm.captchaCode"
                      placeholder="请输入图形验证码"
                      size="large"
                      class="captcha-input"
                    >
                      <template #prefix>
                        <SafetyOutlined class="input-icon" />
                      </template>
                    </a-input>
                    <img
                      v-if="emailCaptchaImage"
                      :src="emailCaptchaImage"
                      alt="图形验证码"
                      title="看不清？点击刷新"
                      class="captcha-img"
                      @click="loadLoginCaptcha"
                    >
                  </div>
                </a-form-item>
                <a-form-item
                  name="emailCode"
                  label="邮箱验证码"
                  :rules="[{ required: true, message: '请输入邮箱验证码' }]"
                >
                  <a-input-search
                    v-model:value="emailCodeForm.emailCode"
                    placeholder="请输入邮箱验证码"
                    size="large"
                    :enter-button="emailSendBtnText"
                    :loading="emailSendLoading"
                    @search="sendLoginCode"
                  />
                </a-form-item>
                <a-form-item>
                  <div class="login-options">
                    <a-checkbox v-model:checked="rememberMe">
                      记住我
                    </a-checkbox>
                  </div>
                </a-form-item>
                <a-form-item>
                  <a-button
                    type="primary"
                    html-type="submit"
                    :loading="emailLoginLoading"
                    block
                    size="large"
                    class="login-btn"
                  >
                    登 录
                  </a-button>
                </a-form-item>
              </a-form>
              <div class="login-actions">
                <a
                  class="back-link"
                  @click="switchToPasswordLogin"
                >使用密码登录</a>
              </div>
            </div>
          </a-tab-pane>
        </a-tabs>

        <!-- 微信扫码登录（开放平台网站应用） -->
        <div class="wx-login-entry">
          <a-divider class="wx-divider">
            <span class="wx-divider-text">其他登录方式</span>
          </a-divider>
          <a-button
            block
            size="large"
            class="wx-login-btn"
            @click="openWechatLogin"
          >
            <WechatOutlined class="wx-btn-icon" />
            微信扫码登录
          </a-button>
        </div>
      </a-card>
    </div>

    <!-- 微信扫码弹窗（登录） -->
    <WechatQrModal
      v-model:open="wechatLoginVisible"
      action="login"
    />

    <a-modal
      v-model:open="registerVisible"
      title="注册账号"
      :footer="null"
      :width="480"
      :centered="true"
      class="register-modal"
      @cancel="registerVisible = false"
    >
      <a-form
        :model="registerForm"
        :rules="registerRules"
        layout="vertical"
        class="register-form"
        @finish="handleRegister"
      >
        <a-form-item
          name="email"
          label="邮箱"
        >
          <a-input
            v-model:value="registerForm.email"
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
              v-model:value="registerForm.captchaCode"
              placeholder="请输入图形验证码"
              size="large"
              class="captcha-input"
            >
              <template #prefix>
                <SafetyOutlined class="input-icon" />
              </template>
            </a-input>
            <img
              v-if="registerCaptchaImage"
              :src="registerCaptchaImage"
              alt="图形验证码"
              title="看不清？点击刷新"
              class="captcha-img"
              @click="loadRegisterCaptcha"
            >
          </div>
        </a-form-item>

        <a-form-item
          name="emailCode"
          label="邮箱验证码"
        >
          <a-input-search
            v-model:value="registerForm.emailCode"
            placeholder="请输入邮箱验证码"
            size="large"
            :enter-button="registerSendBtnText"
            :loading="registerSendLoading"
            @search="sendRegisterCode"
          />
        </a-form-item>

        <a-form-item
          name="userName"
          label="用户名"
        >
          <a-input
            v-model:value="registerForm.userName"
            placeholder="用户名"
            size="large"
            allow-clear
          >
            <template #prefix>
              <UserOutlined class="input-icon" />
            </template>
          </a-input>
        </a-form-item>

        <a-form-item
          name="roleCode"
          label="身份"
        >
          <a-select
            v-model:value="registerForm.roleCode"
            :options="registerRoleOptions"
            placeholder="选择身份（数据来自数据字典）"
            size="large"
            allow-clear
          />
        </a-form-item>

        <a-form-item
          name="passWord"
          label="密码"
        >
          <a-input-password
            v-model:value="registerForm.passWord"
            placeholder="密码（6-64 位）"
            size="large"
          >
            <template #prefix>
              <LockOutlined class="input-icon" />
            </template>
          </a-input-password>
        </a-form-item>

        <a-form-item
          name="nickName"
          label="昵称"
        >
          <a-input
            v-model:value="registerForm.nickName"
            placeholder="昵称（选填）"
            size="large"
            allow-clear
          >
            <template #prefix>
              <UserOutlined class="input-icon" />
            </template>
          </a-input>
        </a-form-item>

        <a-form-item
          name="phone"
          label="手机号"
        >
          <a-input
            v-model:value="registerForm.phone"
            placeholder="手机号（选填）"
            size="large"
            allow-clear
          >
            <template #prefix>
              <PhoneOutlined class="input-icon" />
            </template>
          </a-input>
        </a-form-item>

        <a-form-item>
          <a-button
            type="primary"
            html-type="submit"
            :loading="registerLoading"
            block
            size="large"
            class="login-btn"
          >
            注 册
          </a-button>
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { UserOutlined, LockOutlined, MailOutlined, PhoneOutlined, SafetyOutlined, WechatOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { useI18n } from 'vue-i18n'
import { useUserStore } from '@/stores/user'
import { authApi } from '@/api/auth'
import type { Rule } from 'ant-design-vue/es/form'
import type { LoginParams, RegisterParams, LocateTenantParams, LocateTenantResult } from '@/api/auth'
import { blurActiveFocus } from '@/utils/blurOnTabChange'
import WechatQrModal from '@/components/WechatQrModal.vue'

const { t } = useI18n()

const router = useRouter()
const route = useRoute()
const userStore = useUserStore()

const loginStep = ref<'locate' | 'login'>('locate')
const locateLoading = ref(false)
const loginLoading = ref(false)
const registerLoading = ref(false)
const registerVisible = ref(false)
const rememberMe = ref(false)
/** 登录需要两步验证：服务端返回 428 后置位并显示动态码输入 */
const needTotp = ref(false)
const totpCode = ref('')

/** 登录方式切换：密码登录 / 邮箱验证码登录 */
const loginTab = ref<'password' | 'emailCode'>('password')
const emailCodeForm = reactive({
  email: '',
  captchaCode: '',
  emailCode: '',
})
/** 邮箱验证码登录的图形验证码（发码前置校验，点击图片可刷新） */
const emailCaptchaId = ref('')
const emailCaptchaImage = ref('')
const emailSendLoading = ref(false)
const emailLoginLoading = ref(false)
const emailCountdown = ref(0)
const emailSendBtnText = computed(() =>
  emailCountdown.value > 0 ? `${emailCountdown.value}s 后重发` : '发送验证码',
)

const tenantInfo = ref<LocateTenantResult | null>(null)

const locateForm = reactive({
  userName: '',
})

const loginForm = reactive({
  passWord: '',
})

// 注册固定写入后端配置的默认租户：注册表单不再提供租户选择，因此不传 tenantCode。
// 邮箱 + 邮箱验证码为必填（服务端在建号前一次性校验），注册成功后账号即带着已验证的邮箱。
const registerForm = reactive({
  email: '',
  captchaCode: '',
  emailCode: '',
  userName: '',
  roleCode: undefined as string | undefined,
  passWord: '',
  nickName: '',
  phone: '',
})

const registerRules: Record<string, Rule[]> = {
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    { type: 'email', message: '请输入有效的邮箱地址', trigger: 'blur' },
  ],
  captchaCode: [{ required: true, message: '请输入图形验证码', trigger: 'blur' }],
  emailCode: [{ required: true, message: '请输入邮箱验证码', trigger: 'blur' }],
  userName: [
    { required: true, message: '请输入用户名', trigger: 'blur' },
    { min: 3, max: 50, message: '用户名需为 3-50 位', trigger: 'blur' },
  ],
  roleCode: [{ required: true, message: '请选择身份', trigger: 'change' }],
  passWord: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    { min: 6, max: 64, message: '密码需为 6-64 位', trigger: 'blur' },
  ],
}

/** 注册弹窗自己的图形验证码状态（与「邮箱验证码登录」tab 的状态相互独立，避免互相刷新） */
const registerCaptchaId = ref('')
const registerCaptchaImage = ref('')
const registerSendLoading = ref(false)
const registerCountdown = ref(0)
const registerSendBtnText = computed(() =>
  registerCountdown.value > 0 ? `${registerCountdown.value}s 后重发` : '发送验证码',
)

/** 获取/刷新注册用的图形验证码 */
async function loadRegisterCaptcha() {
  try {
    const data = await authApi.captcha()
    registerCaptchaId.value = data.captchaId
    registerCaptchaImage.value = data.image
    registerForm.captchaCode = ''
  } catch {
    registerCaptchaImage.value = ''
    message.warning('图形验证码加载失败，请点击图片或稍后重试')
  }
}

function startRegisterCountdown() {
  registerCountdown.value = 60
  const timer = setInterval(() => {
    registerCountdown.value--
    if (registerCountdown.value <= 0) clearInterval(timer)
  }, 1000)
}

/** 发送注册邮箱验证码：图形码一次性消费，发送成功后自动刷新下一张 */
async function sendRegisterCode() {
  if (!registerForm.email || !registerForm.email.includes('@')) {
    message.warning('请先输入正确的邮箱')
    return
  }
  if (registerCountdown.value > 0) return
  if (!registerCaptchaId.value || !registerForm.captchaCode) {
    if (!registerCaptchaImage.value) await loadRegisterCaptcha()
    message.warning('请先输入图形验证码')
    return
  }
  registerSendLoading.value = true
  try {
    await authApi.sendCode({
      email: registerForm.email,
      scene: 'register',
      captchaId: registerCaptchaId.value,
      captchaCode: registerForm.captchaCode,
    })
    message.success('验证码已发送，请查收邮件')
    startRegisterCountdown()
    await loadRegisterCaptcha()
  } catch {
    await loadRegisterCaptcha()
  } finally {
    registerSendLoading.value = false
  }
}

/** 注册身份下拉（免登录公开接口，字典驱动；与后端 resolve_selectable_role 同源） */
const registerRoleOptions = ref<{ label: string; value: string }[]>([])
async function loadRegisterRoles() {
  try {
    const list = await authApi.registerRoles()
    registerRoleOptions.value = (list || []).map((r) => ({ label: r.label, value: r.value }))
  } catch {
    registerRoleOptions.value = []
  }
}

/** 微信扫码登录弹窗 */
const wechatLoginVisible = ref(false)
function openWechatLogin() {
  wechatLoginVisible.value = true
}

async function handleLocateTenant() {
  locateLoading.value = true
  try {
    const params: LocateTenantParams = {
      userName: locateForm.userName,
    }
    const res = await authApi.locate(params)
    tenantInfo.value = res
    loginStep.value = 'login'
  } catch {
    // 账户不存在 / 已停用 / 租户过期 / 服务不可用：停留在用户名步骤，不进入密码步骤。
    // 错误提示已由 request 拦截器统一弹出，这里不重复提示。
    tenantInfo.value = null
    loginStep.value = 'locate'
  } finally {
    locateLoading.value = false
  }
}

async function handleLogin() {
  loginLoading.value = true
  try {
    const params: LoginParams = {
      userName: tenantInfo.value?.userName,
      passWord: loginForm.passWord,
      tenantCode: tenantInfo.value?.tenantCode,
      loginType: 'password',
      rememberMe: rememberMe.value,
    }
    if (needTotp.value && totpCode.value) {
      params.totpCode = totpCode.value
    }
    await userStore.login(params)
    const redirect = (route.query.redirect as string) || '/'
    router.push(redirect)
  } catch (error: any) {
    // 428：密码正确但需要两步验证码 → 显示动态码输入并聚焦
    if (error?.code === 428) {
      needTotp.value = true
      message.info('该账号已开启两步验证，请输入动态验证码')
    } else {
      message.error(error.response?.data?.message || error?.message || '登录失败')
    }
  } finally {
    loginLoading.value = false
  }
}

function backToLocate() {
  loginStep.value = 'locate'
  tenantInfo.value = null
  loginForm.passWord = ''
  needTotp.value = false
  totpCode.value = ''
}

/** 切换到密码登录 tab（保留密码两步流程状态） */
function switchToPasswordLogin() {
  loginTab.value = 'password'
}

/** 切到邮箱验证码登录 tab 时加载图形验证码（仅首次） */
function onLoginTabChange(key: string) {
  // 先让被隐藏面板内的焦点元素失焦（否则 Chrome 报 aria-hidden 无障碍告警）
  blurActiveFocus()
  if (key === 'emailCode' && !emailCaptchaId.value) {
    loadLoginCaptcha()
  }
}

/** 获取/刷新图形验证码（邮箱验证码登录的发码前置校验） */
async function loadLoginCaptcha() {
  try {
    const data = await authApi.captcha()
    emailCaptchaId.value = data.captchaId
    emailCaptchaImage.value = data.image
    emailCodeForm.captchaCode = ''
  } catch {
    emailCaptchaImage.value = ''
    message.warning('图形验证码加载失败，请点击图片或稍后重试')
  }
}

function startLoginCountdown() {
  emailCountdown.value = 60
  const timer = setInterval(() => {
    emailCountdown.value--
    if (emailCountdown.value <= 0) clearInterval(timer)
  }, 1000)
}

/** 发送登录邮箱验证码：图形验证码一次性消费，发送成功后自动刷新下一张 */
async function sendLoginCode() {
  if (!emailCodeForm.email || !emailCodeForm.email.includes('@')) {
    message.warning('请先输入正确的邮箱')
    return
  }
  if (emailCountdown.value > 0) return
  if (!emailCaptchaId.value || !emailCodeForm.captchaCode) {
    if (!emailCaptchaImage.value) await loadLoginCaptcha()
    message.warning('请先输入图形验证码')
    return
  }
  emailSendLoading.value = true
  try {
    await authApi.sendCode({
      email: emailCodeForm.email,
      scene: 'login',
      captchaId: emailCaptchaId.value,
      captchaCode: emailCodeForm.captchaCode,
    })
    message.success('验证码已发送，请查收邮件')
    startLoginCountdown()
    await loadLoginCaptcha()
  } catch {
    await loadLoginCaptcha()
  } finally {
    emailSendLoading.value = false
  }
}

/** 邮箱验证码登录：loginType = email_code，验证码替代密码，无需 locate 两步 */
async function handleEmailCodeLogin() {
  emailLoginLoading.value = true
  try {
    const params: LoginParams = {
      // 邮箱验证码登录无需密码，后端 email_code 分支跳过密码校验
      passWord: '',
      email: emailCodeForm.email,
      emailCode: emailCodeForm.emailCode,
      loginType: 'email_code',
      rememberMe: rememberMe.value,
    }
    await userStore.login(params)
    const redirect = (route.query.redirect as string) || '/'
    router.push(redirect)
  } catch (error: any) {
    // 邮箱验证码登录不提供 TOTP 输入，若账号开启了 2FA 引导走密码登录
    if (error?.code === 428) {
      message.warning('该账号已开启两步验证，请切换到密码登录')
    } else {
      message.error(error.response?.data?.message || error?.message || '登录失败')
    }
  } finally {
    emailLoginLoading.value = false
  }
}

function openRegister() {
  registerVisible.value = true
  // 身份下拉数据源（字典驱动）+ 图形验证码：打开弹窗时各拉一次
  if (!registerRoleOptions.value.length) loadRegisterRoles()
  loadRegisterCaptcha()
}

async function handleRegister() {
  registerLoading.value = true
  try {
    // 不传 tenantCode：后端按 sea-orm-ext-tenant.default_tenant_id 落到默认租户
    const params: RegisterParams = {
      userName: registerForm.userName,
      passWord: registerForm.passWord,
      email: registerForm.email,
      emailCode: registerForm.emailCode,
      nickName: registerForm.nickName || undefined,
      phone: registerForm.phone || undefined,
      roleCode: registerForm.roleCode || '',
    }
    await authApi.register(params)
    message.success('注册成功，请登录')
    registerVisible.value = false
    resetRegisterForms()
  } finally {
    registerLoading.value = false
  }
}

function resetRegisterForms() {
  registerForm.email = ''
  registerForm.captchaCode = ''
  registerForm.emailCode = ''
  registerForm.userName = ''
  registerForm.roleCode = undefined
  registerForm.passWord = ''
  registerForm.nickName = ''
  registerForm.phone = ''
}
</script>

<style scoped>
.login-container {
  position: relative;
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  overflow: hidden;
  background: linear-gradient(135deg, #0f0c29 0%, #302b63 50%, #24243e 100%);
}

.login-bg {
  position: absolute;
  inset: 0;
  overflow: hidden;
  pointer-events: none;
}

.bg-shape {
  position: absolute;
  border-radius: 50%;
  filter: blur(80px);
  opacity: 0.5;
}

.shape-1 {
  width: 500px;
  height: 500px;
  background: radial-gradient(circle, rgba(99, 102, 241, 0.4), transparent 70%);
  top: -150px;
  right: -100px;
  animation: float 8s ease-in-out infinite;
}

.shape-2 {
  width: 400px;
  height: 400px;
  background: radial-gradient(circle, rgba(168, 85, 247, 0.35), transparent 70%);
  bottom: -100px;
  left: -80px;
  animation: float 10s ease-in-out infinite reverse;
}

.shape-3 {
  width: 300px;
  height: 300px;
  background: radial-gradient(circle, rgba(59, 130, 246, 0.3), transparent 70%);
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  animation: float 12s ease-in-out infinite;
}

@keyframes float {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-30px); }
}

/* 微信扫码登录入口 */
.wx-login-entry {
  margin-top: 8px;
}

.wx-divider {
  margin: 12px 0 16px;
}

.wx-divider-text {
  font-size: 13px;
  color: var(--text-muted, #8a94a6);
}

.wx-login-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  color: #07c160;
  border-color: #07c160;
  background: transparent;
}

.wx-login-btn:hover {
  color: #07c160;
  border-color: #07c160;
  background: rgba(7, 193, 96, 0.08);
}

.wx-btn-icon {
  font-size: 18px;
}

.login-wrapper {
  position: relative;
  z-index: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  width: 100%;
  max-width: 420px;
  padding: 0 20px;
}

.login-brand {
  text-align: center;
  margin-bottom: 32px;
}

.brand-mark {
  display: block;
  width: 56px;
  height: 56px;
  margin: 0 auto 18px;
}

.brand-mark svg {
  width: 56px;
  height: 56px;
}

.brand-title {
  color: #fff;
  font-size: 30px;
  font-weight: 700;
  letter-spacing: 3px;
  margin: 0 0 8px;
}

.brand-subtitle {
  color: rgba(255, 255, 255, 0.6);
  font-size: 14px;
  margin: 0;
  letter-spacing: 6px;
  text-indent: 6px;
}

.login-card {
  width: 100%;
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.login-card :deep(.ant-card-body) {
  padding: 32px;
}

.login-tabs :deep(.ant-tabs-nav) {
  margin-bottom: 20px;
}

.login-tabs :deep(.ant-tabs-tab) {
  font-size: 14px;
}

.captcha-row {
  display: flex;
  gap: 8px;
  align-items: stretch;
}

.captcha-row .captcha-input {
  flex: 1;
  min-width: 0;
}

.captcha-row .captcha-img {
  width: 118px;
  height: 40px;
  flex-shrink: 0;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  background: var(--component-bg);
  object-fit: contain;
  cursor: pointer;
  user-select: none;
}

.step-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 16px;
}

.step-number {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: var(--border-color);
  color: var(--header-text-secondary);
  font-size: 14px;
  font-weight: 600;
  transition: all 0.3s ease;
}

.step-number.active {
  background: var(--brand-gradient);
  color: #fff;
}

.step-line {
  width: 48px;
  height: 2px;
  margin: 0 8px;
  background: var(--border-color);
  transition: all 0.3s ease;
}

.step-line.active {
  background: var(--brand-gradient);
}

.step-title {
  text-align: center;
  font-size: 16px;
  font-weight: 600;
  color: var(--header-text);
  margin-bottom: 24px;
}

.login-form {
  margin-top: 4px;
}

.login-form :deep(.ant-form-item) {
  margin-bottom: 20px;
}

.login-form :deep(.ant-form-item:last-child) {
  margin-bottom: 0;
}

.login-options {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.login-options :deep(.ant-checkbox-wrapper) {
  font-size: 14px;
}

.login-form :deep(.ant-input-affix-wrapper),
.login-form :deep(.ant-input) {
  border-radius: 10px;
}

.login-form :deep(.ant-input-affix-wrapper .ant-input) {
  background: transparent;
}

.input-icon {
  color: inherit;
  opacity: 0.45;
  font-size: 16px;
}

.login-btn {
  height: 44px;
  border-radius: 10px;
  font-size: 16px;
  font-weight: 600;
  letter-spacing: 4px;
  background: var(--brand-gradient);
  border: none;
  box-shadow: 0 4px 15px rgba(31, 138, 95, 0.35);
  transition: all 0.3s ease;
}

.login-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(31, 138, 95, 0.45);
}

.login-btn:active {
  transform: translateY(0);
}

.login-footer {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-top: 20px;
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
}

.footer-text {
  color: var(--header-text-secondary);
  font-size: 13px;
}

.footer-link {
  color: var(--brand-primary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  margin-left: 4px;
  transition: color 0.2s;
}

.footer-link:hover {
  color: var(--brand-primary-hover);
}

.login-actions {
  display: flex;
  justify-content: center;
  margin-top: 16px;
}

.back-link {
  color: var(--brand-primary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: color 0.2s;
}

.back-link:hover {
  color: var(--brand-primary-hover);
}

.tenant-info {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-bottom: 20px;
  padding: 12px;
  background: var(--brand-primary-lighter);
  border-radius: 8px;
}

.tenant-hint {
  color: var(--header-text-secondary);
  font-size: 14px;
}

.register-form :deep(.ant-form-item) {
  margin-bottom: 16px;
}

.register-form :deep(.ant-form-item:last-child) {
  margin-bottom: 0;
}

.register-form :deep(.ant-input-affix-wrapper),
.register-form :deep(.ant-input) {
  border-radius: 10px;
}

.register-form :deep(.ant-input-affix-wrapper .ant-input) {
  background: transparent;
}
</style>
