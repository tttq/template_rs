<template>
  <a-dropdown :trigger="['click']">
    <span class="header-action" @click.prevent>
      <GlobalOutlined />
    </span>
    <template #overlay>
      <a-menu @click="onSelect" :selected-keys="[appStore.locale]">
        <a-menu-item key="zh-CN">
          <span>简体中文</span>
        </a-menu-item>
        <a-menu-item key="en-US">
          <span>English</span>
        </a-menu-item>
      </a-menu>
    </template>
  </a-dropdown>
</template>

<script setup lang="ts">
import { GlobalOutlined } from '@ant-design/icons-vue'
import { useI18n } from 'vue-i18n'
import { useAppStore, type Locale } from '@/stores/app'

const appStore = useAppStore()
const { locale } = useI18n()

function onSelect({ key }: { key: string }) {
  appStore.setLocale(key as Locale)
  locale.value = key
}
</script>
