<template>
  <a-config-provider :theme="themeConfig">
    <router-view />
  </a-config-provider>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { theme } from 'ant-design-vue'
import { useAppStore, FONT_SIZE_PX } from '@/stores/app'

const appStore = useAppStore()

const themeConfig = computed(() => ({
  algorithm: appStore.isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
  token: {
    fontSize: FONT_SIZE_PX[appStore.fontSize],
  },
}))
</script>