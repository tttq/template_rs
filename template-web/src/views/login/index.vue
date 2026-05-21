<template>
  <div class="login-container">
    <div class="login-bg">
      <div class="bg-shape shape-1"></div>
      <div class="bg-shape shape-2"></div>
      <div class="bg-shape shape-3"></div>
    </div>
    <div class="login-wrapper">
      <div class="login-brand">
        <div class="brand-icon">
          <svg viewBox="0 0 40 40" fill="none" xmlns="http://www.w3.org/2000/svg">
            <rect width="40" height="40" rx="10" fill="rgba(255,255,255,0.25)" />
            <path d="M12 20L18 26L28 14" stroke="white" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" />
          </svg>
        </div>
        <h1 class="brand-title">Template Admin</h1>
        <p class="brand-subtitle">企业级后台管理系统</p>
      </div>
      <a-card class="login-card" :bordered="false">
        <div v-if="loginStep === 'locate'" class="login-step-locate">
          <div class="step-indicator">
            <span class="step-number active">1</span>
            <span class="step-line"></span>
            <span class="step-number">2</span>
          </div>
          <div class="step-title">步骤 1/2：输入用户名</div>
          <a-form :model="locateForm" @finish="handleLocateTenant" layout="vertical" class="login-form">
            <a-form-item name="userName" :rules="[{ required: true, message: '请输入用户名' }]">
              <a-input v-model:value="locateForm.userName" placeholder="请输入用户名" size="large" allow-clear>
                <template #prefix><UserOutlined class="input-icon" /></template>
              </a-input>
            </a-form-item>
            <a-form-item>
              <a-button type="primary" html-type="submit" :loading="locateLoading" block size="large" class="login-btn">
                下一步
              </a-button>
            </a-form-item>
          </a-form>
          <div class="login-footer">
            <span class="footer-text">还没有账号？</span>
            <a class="footer-link" @click="openRegister">注册账号</a>
          </div>
        </div>

        <div v-else class="login-step-login">
          <div class="step-indicator">
            <span class="step-number">1</span>
            <span class="step-line active"></span>
            <span class="step-number active">2</span>
          </div>
          <div class="step-title">步骤 2/2：输入密码</div>
          
          <div v-if="tenantInfo" class="tenant-info">
            <a-tag color="blue">{{ tenantInfo.tenantName }}</a-tag>
            <span class="tenant-hint">欢迎回来，{{ tenantInfo.userName }}</span>
          </div>

          <a-form :model="loginForm" @finish="handleLogin" layout="vertical" class="login-form">
            <a-form-item>
              <a-input :value="tenantInfo?.userName" disabled placeholder="用户名" size="large">
                <template #prefix><UserOutlined class="input-icon" /></template>
              </a-input>
            </a-form-item>
            <a-form-item name="passWord" :rules="[{ required: true, message: '请输入密码' }]">
              <a-input-password v-model:value="loginForm.passWord" placeholder="请输入密码" size="large">
                <template #prefix><LockOutlined class="input-icon" /></template>
              </a-input-password>
            </a-form-item>
            <a-form-item>
              <div class="login-options">
                <a-checkbox v-model:checked="rememberMe">{{ t('login.rememberMe') }}</a-checkbox>
              </div>
            </a-form-item>
            <a-form-item>
              <a-button type="primary" html-type="submit" :loading="loginLoading" block size="large" class="login-btn">
                登 录
              </a-button>
            </a-form-item>
          </a-form>
          <div class="login-actions">
            <a class="back-link" @click="backToLocate">返回上一步</a>
          </div>
        </div>
      </a-card>
    </div>

    <a-modal
      v-model:open="registerVisible"
      title="注册账号"
      :footer="null"
      :width="480"
      :centered="true"
      class="register-modal"
      @cancel="registerVisible = false"
    >
      <a-tabs v-model:activeKey="registerTab" centered class="register-tabs">
        <a-tab-pane key="username" tab="用户名注册">
          <a-form :model="usernameRegisterForm" @finish="handleRegister" layout="vertical" class="register-form">
            <a-form-item name="userName" :rules="[{ required: true, message: '请输入用户名' }]">
              <a-input v-model:value="usernameRegisterForm.userName" placeholder="用户名" size="large" allow-clear>
                <template #prefix><UserOutlined class="input-icon" /></template>
              </a-input>
            </a-form-item>
            <a-form-item name="tenantCode" :rules="[{ required: true, message: '请输入租户编码' }]">
              <a-input v-model:value="usernameRegisterForm.tenantCode" placeholder="租户编码" size="large" allow-clear>
                <template #prefix><BankOutlined class="input-icon" /></template>
              </a-input>
            </a-form-item>
            <a-form-item name="passWord" :rules="[{ required: true, message: '请输入密码' }, { min: 6, message: '密码至少6位' }]">
              <a-input-password v-model:value="usernameRegisterForm.passWord" placeholder="密码" size="large">
                <template #prefix><LockOutlined class="input-icon" /></template>
              </a-input-password>
            </a-form-item>
            <a-form-item name="nickName">
              <a-input v-model:value="usernameRegisterForm.nickName" placeholder="昵称（选填）" size="large" allow-clear>
                <template #prefix><UserOutlined class="input-icon" /></template>
              </a-input>
            </a-form-item>
            <a-form-item name="phone">
              <a-input v-model:value="usernameRegisterForm.phone" placeholder="手机号（选填）" size="large" allow-clear>
                <template #prefix><PhoneOutlined class="input-icon" /></template>
              </a-input>
            </a-form-item>
            <a-form-item>
              <a-button type="primary" html-type="submit" :loading="registerLoading" block size="large" class="login-btn">
                注 册
              </a-button>
            </a-form-item>
          </a-form>
        </a-tab-pane>
        <a-tab-pane key="email" tab="邮箱注册">
          <a-form :model="emailRegisterForm" @finish="handleRegister" layout="vertical" class="register-form">
            <a-form-item name="email" :rules="[{ required: true, message: '请输入邮箱' }, { type: 'email', message: '请输入有效的邮箱地址' }]">
              <a-input v-model:value="emailRegisterForm.email" placeholder="邮箱地址" size="large" allow-clear>
                <template #prefix><MailOutlined class="input-icon" /></template>
              </a-input>
            </a-form-item>
            <a-form-item name="tenantCode" :rules="[{ required: true, message: '请输入租户编码' }]">
              <a-input v-model:value="emailRegisterForm.tenantCode" placeholder="租户编码" size="large" allow-clear>
                <template #prefix><BankOutlined class="input-icon" /></template>
              </a-input>
            </a-form-item>
            <a-form-item name="passWord" :rules="[{ required: true, message: '请输入密码' }, { min: 6, message: '密码至少6位' }]">
              <a-input-password v-model:value="emailRegisterForm.passWord" placeholder="密码" size="large">
                <template #prefix><LockOutlined class="input-icon" /></template>
              </a-input-password>
            </a-form-item>
            <a-form-item name="userName">
              <a-input v-model:value="emailRegisterForm.userName" placeholder="用户名（选填）" size="large" allow-clear>
                <template #prefix><UserOutlined class="input-icon" /></template>
              </a-input>
            </a-form-item>
            <a-form-item name="nickName">
              <a-input v-model:value="emailRegisterForm.nickName" placeholder="昵称（选填）" size="large" allow-clear>
                <template #prefix><UserOutlined class="input-icon" /></template>
              </a-input>
            </a-form-item>
            <a-form-item name="phone">
              <a-input v-model:value="emailRegisterForm.phone" placeholder="手机号（选填）" size="large" allow-clear>
                <template #prefix><PhoneOutlined class="input-icon" /></template>
              </a-input>
            </a-form-item>
            <a-form-item>
              <a-button type="primary" html-type="submit" :loading="registerLoading" block size="large" class="login-btn">
                注 册
              </a-button>
            </a-form-item>
          </a-form>
        </a-tab-pane>
      </a-tabs>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import { reactive, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { UserOutlined, LockOutlined, MailOutlined, PhoneOutlined, BankOutlined } from '@ant-design/icons-vue'
import { message } from 'ant-design-vue'
import { useI18n } from 'vue-i18n'
import { useUserStore } from '@/stores/user'
import { authApi } from '@/api/auth'
import type { LoginParams, RegisterParams, LocateTenantParams, LocateTenantResult } from '@/api/auth'

const { t } = useI18n()

const router = useRouter()
const route = useRoute()
const userStore = useUserStore()

const loginStep = ref<'locate' | 'login'>('locate')
const locateLoading = ref(false)
const loginLoading = ref(false)
const registerLoading = ref(false)
const registerVisible = ref(false)
const registerTab = ref('username')
const rememberMe = ref(false)

const tenantInfo = ref<LocateTenantResult | null>(null)

const locateForm = reactive({
  userName: '',
})

const loginForm = reactive({
  passWord: '',
})

const usernameRegisterForm = reactive({
  userName: '',
  tenantCode: '',
  passWord: '',
  nickName: '',
  phone: '',
})

const emailRegisterForm = reactive({
  email: '',
  tenantCode: '',
  passWord: '',
  userName: '',
  nickName: '',
  phone: '',
})

async function handleLocateTenant() {
  locateLoading.value = true
  try {
    const params: LocateTenantParams = {
      userName: locateForm.userName,
    }
    const res = await authApi.locate(params)
    tenantInfo.value = res
    loginStep.value = 'login'
  } catch (_error: any) {
    tenantInfo.value = { tenantName: '', tenantCode: '', userName: locateForm.userName, loginType: 'username' }
    loginStep.value = 'login'
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
    await userStore.login(params)
    const redirect = (route.query.redirect as string) || '/'
    router.push(redirect)
  } catch (error: any) {
    message.error(error.response?.data?.message || '登录失败')
  } finally {
    loginLoading.value = false
  }
}

function backToLocate() {
  loginStep.value = 'locate'
  tenantInfo.value = null
  loginForm.passWord = ''
}

function openRegister() {
  registerVisible.value = true
  registerTab.value = 'username'
}

async function handleRegister() {
  registerLoading.value = true
  try {
    const isUsername = registerTab.value === 'username'
    const params: RegisterParams = isUsername
      ? {
          userName: usernameRegisterForm.userName,
          tenantCode: usernameRegisterForm.tenantCode,
          passWord: usernameRegisterForm.passWord,
          nickName: usernameRegisterForm.nickName || undefined,
          phone: usernameRegisterForm.phone || undefined,
          registerType: 'username',
        }
      : {
          userName: emailRegisterForm.userName,
          tenantCode: emailRegisterForm.tenantCode,
          passWord: emailRegisterForm.passWord,
          nickName: emailRegisterForm.nickName || undefined,
          phone: emailRegisterForm.phone || undefined,
          email: emailRegisterForm.email,
          registerType: 'email',
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
  usernameRegisterForm.userName = ''
  usernameRegisterForm.tenantCode = ''
  usernameRegisterForm.passWord = ''
  usernameRegisterForm.nickName = ''
  usernameRegisterForm.phone = ''
  emailRegisterForm.email = ''
  emailRegisterForm.tenantCode = ''
  emailRegisterForm.passWord = ''
  emailRegisterForm.userName = ''
  emailRegisterForm.nickName = ''
  emailRegisterForm.phone = ''
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

.brand-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  margin-bottom: 16px;
}

.brand-icon svg {
  width: 56px;
  height: 56px;
}

.brand-title {
  color: #fff;
  font-size: 28px;
  font-weight: 700;
  letter-spacing: 1px;
  margin: 0 0 8px;
}

.brand-subtitle {
  color: rgba(255, 255, 255, 0.65);
  font-size: 14px;
  margin: 0;
  letter-spacing: 2px;
}

.login-card {
  width: 100%;
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  backdrop-filter: blur(10px);
  background: rgba(255, 255, 255, 0.97);
}

.login-card :deep(.ant-card-body) {
  padding: 32px;
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
  background: #f0f0f0;
  color: #999;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.3s ease;
}

.step-number.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: #fff;
}

.step-line {
  width: 48px;
  height: 2px;
  margin: 0 8px;
  background: #f0f0f0;
  transition: all 0.3s ease;
}

.step-line.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.step-title {
  text-align: center;
  font-size: 16px;
  font-weight: 600;
  color: #333;
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
  color: rgba(0, 0, 0, 0.65);
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
  color: rgba(0, 0, 0, 0.25);
  font-size: 16px;
}

.login-btn {
  height: 44px;
  border-radius: 10px;
  font-size: 16px;
  font-weight: 600;
  letter-spacing: 4px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
  transition: all 0.3s ease;
}

.login-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.5);
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
  border-top: 1px solid #f0f0f0;
}

.footer-text {
  color: rgba(0, 0, 0, 0.45);
  font-size: 13px;
}

.footer-link {
  color: #667eea;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  margin-left: 4px;
  transition: color 0.2s;
}

.footer-link:hover {
  color: #764ba2;
}

.login-actions {
  display: flex;
  justify-content: center;
  margin-top: 16px;
}

.back-link {
  color: #667eea;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: color 0.2s;
}

.back-link:hover {
  color: #764ba2;
}

.tenant-info {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  margin-bottom: 20px;
  padding: 12px;
  background: rgba(102, 126, 234, 0.08);
  border-radius: 8px;
}

.tenant-hint {
  color: #666;
  font-size: 14px;
}

.register-tabs :deep(.ant-tabs-nav) {
  margin-bottom: 20px;
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