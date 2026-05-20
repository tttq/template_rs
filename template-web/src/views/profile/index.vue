<template>
  <div class="profile-page">
    <a-row :gutter="16">
      <a-col :span="8" :xs="24" :sm="24" :md="8">
        <a-card class="profile-card">
          <div class="profile-avatar-section">
            <a-avatar :size="80" :src="userStore.userInfo?.avatar" class="profile-avatar">
              <template #icon><UserOutlined /></template>
            </a-avatar>
            <h3 class="profile-name">{{ userStore.userInfo?.nickName || userStore.userInfo?.userName || t('common.user') }}</h3>
            <p class="profile-role">{{ userStore.roles?.join(', ') || t('common.noRole') }}</p>
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
      <a-col :span="16" :xs="24" :sm="24" :md="16">
        <a-card :title="t('common.basicInfo')">
          <a-form layout="vertical">
            <a-row :gutter="16">
              <a-col :span="12" :xs="24">
                <a-form-item :label="t('common.userName')">
                  <a-input :value="userStore.userInfo?.userName" disabled />
                </a-form-item>
              </a-col>
              <a-col :span="12" :xs="24">
                <a-form-item :label="t('common.nickName')">
                  <a-input :value="userStore.userInfo?.nickName" />
                </a-form-item>
              </a-col>
              <a-col :span="12" :xs="24">
                <a-form-item :label="t('common.email')">
                  <a-input :value="userStore.userInfo?.email" />
                </a-form-item>
              </a-col>
              <a-col :span="12" :xs="24">
                <a-form-item :label="t('common.phone')">
                  <a-input :value="userStore.userInfo?.phone" />
                </a-form-item>
              </a-col>
            </a-row>
            <a-form-item>
              <a-button type="primary">{{ t('common.saveModify') }}</a-button>
            </a-form-item>
          </a-form>
        </a-card>
      </a-col>
    </a-row>
  </div>
</template>

<script setup lang="ts">
import { UserOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import { useUserStore } from '@/stores/user'

const { t } = useI18n()
const userStore = useUserStore()
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
  color: rgba(0, 0, 0, 0.45);
  font-size: 13px;
}

.profile-info-list {
  text-align: left;
}

.info-item {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid #f0f0f0;
}

.info-item:last-child {
  border-bottom: none;
}

.info-label {
  color: rgba(0, 0, 0, 0.45);
}

.info-value {
  color: rgba(0, 0, 0, 0.85);
}
</style>
