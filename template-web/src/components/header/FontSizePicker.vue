<template>
  <a-dropdown :trigger="['click']">
    <span class="header-action" @click.prevent>
      <FontSizeOutlined />
    </span>
    <template #overlay>
      <a-menu @click="onSelect" :selected-keys="[appStore.fontSize]">
        <a-menu-item key="small">
          <span style="font-size: 12px">A</span>
          <span class="font-size-label">{{ t('fontSizePicker.small') }}</span>
        </a-menu-item>
        <a-menu-item key="default">
          <span style="font-size: 14px">A</span>
          <span class="font-size-label">{{ t('fontSizePicker.default') }}</span>
        </a-menu-item>
        <a-menu-item key="large">
          <span style="font-size: 16px; font-weight: 600">A</span>
          <span class="font-size-label">{{ t('fontSizePicker.large') }}</span>
        </a-menu-item>
      </a-menu>
    </template>
  </a-dropdown>
</template>

<script setup lang="ts">
import { FontSizeOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import { useAppStore, type FontSize } from '@/stores/app'

const { t } = useI18n()
const appStore = useAppStore()

function onSelect({ key }: { key: string }) {
  appStore.setFontSize(key as FontSize)
}
</script>

<style scoped>
.font-size-label {
  margin-left: 8px;
  color: rgba(0, 0, 0, 0.45);
}
</style>
