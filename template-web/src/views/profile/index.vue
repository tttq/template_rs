<template>
  <div class="profile-page">
    <a-row :gutter="16">
      <a-col
        :span="8"
        :xs="24"
        :sm="24"
        :md="8"
      >
        <a-card class="profile-card">
          <div class="profile-avatar-section">
            <a-avatar
              :size="80"
              :src="userStore.userInfo?.avatar"
              class="profile-avatar"
            >
              <template #icon>
                <UserOutlined />
              </template>
            </a-avatar>
            <h3 class="profile-name">
              {{ userStore.userInfo?.nickName || userStore.userInfo?.userName || t('common.user') }}
            </h3>
            <p class="profile-role">
              {{ userStore.roles?.join(', ') || t('common.noRole') }}
            </p>
          </div>
          <a-divider />
          <div class="profile-info-list">
            <div class="info-item">
              <span class="info-label">{{ t('common.userName') }}</span>
              <span class="info-value">{{ userStore.userInfo?.userName || '-' }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ t('common.nickName') }}</span>
              <span class="info-value">{{ userStore.userInfo?.nickName || '-' }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ t('common.email') }}</span>
              <span class="info-value">{{ userStore.userInfo?.email || '-' }}</span>
            </div>
            <div class="info-item">
              <span class="info-label">{{ t('common.phone') }}</span>
              <span class="info-value">{{ userStore.userInfo?.phone || '-' }}</span>
            </div>
          </div>
        </a-card>
      </a-col>
      <a-col
        :span="16"
        :xs="24"
        :sm="24"
        :md="16"
      >
        <a-card
          :title="t('common.basicInfo')"
          style="margin-bottom: 16px"
        >
          <a-form layout="vertical">
            <a-row :gutter="16">
              <a-col
                :span="12"
                :xs="24"
              >
                <a-form-item :label="t('common.userName')">
                  <a-input
                    :value="userStore.userInfo?.userName"
                    disabled
                  />
                </a-form-item>
              </a-col>
              <a-col
                :span="12"
                :xs="24"
              >
                <a-form-item :label="t('common.nickName')">
                  <a-input :value="userStore.userInfo?.nickName" />
                </a-form-item>
              </a-col>
              <a-col
                :span="12"
                :xs="24"
              >
                <a-form-item :label="t('common.email')">
                  <a-input :value="userStore.userInfo?.email" />
                </a-form-item>
              </a-col>
              <a-col
                :span="12"
                :xs="24"
              >
                <a-form-item :label="t('common.phone')">
                  <a-input :value="userStore.userInfo?.phone" />
                </a-form-item>
              </a-col>
            </a-row>
            <a-form-item>
              <a-button type="primary">
                {{ t('common.saveModify') }}
              </a-button>
            </a-form-item>
          </a-form>
        </a-card>

        <a-card
          title="微信绑定"
          style="margin-bottom: 16px"
        >
          <template #extra>
            <a-tag :color="wechatBound ? 'green' : 'default'">
              {{ wechatBound ? '已绑定' : '未绑定' }}
            </a-tag>
          </template>
          <p class="wechat-desc">
            一个账号只能绑定一个微信。绑定后可用微信扫码直接登录；
            如需更换微信，请先解绑再重新绑定。
          </p>
          <a-space>
            <a-button
              v-if="!wechatBound"
              type="primary"
              :loading="bindingsLoading"
              @click="bindWechat"
            >
              扫码绑定微信
            </a-button>
            <a-button
              v-else
              danger
              :loading="unbindLoading"
              @click="unbindWechat"
            >
              解绑微信
            </a-button>
          </a-space>
        </a-card>
      </a-col>
    </a-row>

    <!-- 微信扫码绑定弹窗（与登录页复用同一组件） -->
    <WechatQrModal
      v-model:open="wechatBindVisible"
      action="bind"
    />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { UserOutlined } from '@ant-design/icons-vue'
import { message, Modal } from 'ant-design-vue'
import { useI18n } from 'vue-i18n'
import { useUserStore } from '@/stores/user'
import { authApi } from '@/api/auth'
import WechatQrModal from '@/components/WechatQrModal.vue'

const { t } = useI18n()
const userStore = useUserStore()

/** 微信绑定状态（一个账号只能绑一个微信） */
const wechatBound = ref(false)
const bindingsLoading = ref(false)
const unbindLoading = ref(false)
const wechatBindVisible = ref(false)

async function loadBindings() {
  bindingsLoading.value = true
  try {
    const list = await authApi.bindings()
    wechatBound.value = (list || []).some((b) => b.provider === 'wechat' || b.provider === 'wechat_mp')
  } catch {
    wechatBound.value = false
  } finally {
    bindingsLoading.value = false
  }
}

function bindWechat() {
  wechatBindVisible.value = true
}

async function unbindWechat() {
  Modal.confirm({
    title: '解绑微信',
    content: '解绑后该微信将无法扫码登录本账号，确定继续吗？',
    okText: '解绑',
    okType: 'danger',
    cancelText: '取消',
    onOk: async () => {
      unbindLoading.value = true
      try {
        await authApi.unbind('wechat')
        message.success('已解绑')
        wechatBound.value = false
      } finally {
        unbindLoading.value = false
      }
    },
  })
}

// 说明：两步验证（TOTP）由后端 `TwoFactorHook` 全局钩子决定是否启用，
// 本模板未接入该钩子的实现，因此个人中心不展示 2FA 开关；
// 需要时实现 system 层 `auth::two_fa::TwoFactorHook` 并在此补充绑定流程即可。

onMounted(() => {
  loadBindings()
})

// 绑定弹窗关闭后刷新绑定状态（用户可能在弹窗内完成了扫码绑定）
watch(wechatBindVisible, (val) => {
  if (!val) loadBindings()
})
</script>

<style scoped>
.profile-page {
  padding: 0;
}

.profile-card {
  text-align: center;
}

.profile-avatar-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.profile-avatar {
  background: linear-gradient(135deg, #1890ff, #36cfc9);
}

.profile-name {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.profile-role {
  margin: 0;
  color: var(--header-text-secondary);
  font-size: 13px;
}

.profile-info-list {
  text-align: left;
}

.info-item {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid var(--border-color);
}

.info-item:last-child {
  border-bottom: none;
}

.info-label {
  color: var(--header-text-secondary);
}

.info-value {
  color: var(--header-text);
}

.wechat-desc {
  color: var(--header-text-secondary);
  font-size: 13px;
  margin: 0 0 16px;
}
</style>
