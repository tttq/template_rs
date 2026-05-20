<template>
  <a-dropdown :trigger="['click']">
    <span class="header-action" @click.prevent>
      <BulbOutlined v-if="appStore.themeMode === 'light'" />
      <BulbFilled v-else-if="appStore.themeMode === 'dark'" />
      <LaptopOutlined v-else />
    </span>
    <template #overlay>
      <a-menu @click="onSelect" :selected-keys="[appStore.themeMode]">
        <a-menu-item key="light">
          <BulbOutlined />
          <span class="theme-label">{{ t('themePicker.light') }}</span>
        </a-menu-item>
        <a-menu-item key="dark">
          <BulbFilled />
          <span class="theme-label">{{ t('themePicker.dark') }}</span>
        </a-menu-item>
        <a-menu-item key="system">
          <LaptopOutlined />
          <span class="theme-label">{{ t('themePicker.system') }}</span>
        </a-menu-item>
      </a-menu>
    </template>
  </a-dropdown>
</template>

<script setup lang="ts">
import { BulbOutlined, BulbFilled, LaptopOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import { useAppStore, type ThemeMode } from '@/stores/app'

const { t } = useI18n()
const appStore = useAppStore()

function onSelect({ key }: { key: string }) {
  appStore.setThemeMode(key as ThemeMode)
}
</script>

<style scoped>
.theme-label {
  margin-left: 8px;
}
</style>
