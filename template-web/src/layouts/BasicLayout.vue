<template>
  <a-layout style="min-height: 100vh">
    <a-layout-sider
      v-model:collapsed="appStore.collapsed"
      collapsible
      :trigger="null"
      width="240"
      class="sider"
    >
      <div class="logo">
        <div class="logo-icon">
          <DashboardOutlined />
        </div>
        <span v-if="!appStore.collapsed" class="logo-text">Template Admin</span>
      </div>
      <a-menu
        v-model:selectedKeys="selectedKeys"
        v-model:openKeys="openKeys"
        mode="inline"
        theme="dark"
        @click="onMenuClick"
        class="sider-menu"
      >
        <template v-for="menu in userStore.menus" :key="menu.path || menu.id">
          <a-menu-item v-if="!menu.children?.length" :key="menu.path || String(menu.id)">
            <component :is="getIcon(menu.icon)" />
            <span>{{ getMenuName(menu) }}</span>
          </a-menu-item>
          <a-sub-menu v-else :key="menu.path || String(menu.id)">
            <template #icon><component :is="getIcon(menu.icon)" /></template>
            <template #title>{{ getMenuName(menu) }}</template>
            <a-menu-item
              v-for="child in menu.children"
              :key="child.path || String(child.id)"
            >
              <component :is="getIcon(child.icon)" />
              <span>{{ getMenuName(child) }}</span>
            </a-menu-item>
          </a-sub-menu>
        </template>
      </a-menu>
    </a-layout-sider>
    <a-layout>
      <a-layout-header class="header">
        <div class="header-left">
          <span class="collapse-btn" @click="appStore.toggleCollapsed()">
            <MenuFoldOutlined v-if="!appStore.collapsed" />
            <MenuUnfoldOutlined v-else />
          </span>
          <a-breadcrumb class="breadcrumb">
            <a-breadcrumb-item v-for="item in breadcrumbs" :key="item.path">
              <router-link v-if="item.path" :to="item.path">{{ item.title }}</router-link>
              <span v-else>{{ item.title }}</span>
            </a-breadcrumb-item>
          </a-breadcrumb>
        </div>
        <div class="header-right">
          <LocalePicker />
          <FontSizePicker />
          <ThemePicker />
          <a-dropdown>
            <div class="user-info">
              <a-avatar :size="32" :src="userStore.userInfo?.avatar" class="user-avatar">
                <template #icon><UserOutlined /></template>
              </a-avatar>
              <span class="user-name">{{ userStore.userInfo?.nickName || userStore.userInfo?.userName || t('common.user') }}</span>
              <DownOutlined />
            </div>
            <template #overlay>
              <a-menu>
                <a-menu-item key="center" @click="$router.push('/profile')">
                  <UserOutlined />
                  <span style="margin-left: 8px">{{ t('header.profile') }}</span>
                </a-menu-item>
                <a-menu-item key="settings" @click="$router.push('/system/config')">
                  <SettingOutlined />
                  <span style="margin-left: 8px">{{ t('header.systemConfig') }}</span>
                </a-menu-item>
                <a-menu-divider />
                <a-menu-item key="logout" @click="doLogout">
                  <LogoutOutlined />
                  <span style="margin-left: 8px">{{ t('header.logout') }}</span>
                </a-menu-item>
              </a-menu>
            </template>
          </a-dropdown>
        </div>
      </a-layout-header>
      <div class="tags-bar" v-if="visitedViews.length > 0">
        <div class="tags-scroll">
          <span
            v-for="tag in visitedViews"
            :key="tag.path"
            class="tag-item"
            :class="{ active: tag.path === route.path }"
            @click="router.push(tag.path)"
          >
            {{ getTagTitle(tag) }}
            <CloseOutlined
              v-if="tag.path !== '/dashboard'"
              class="tag-close"
              @click.stop="closeTag(tag)"
            />
          </span>
        </div>
      </div>
      <a-layout-content class="content">
        <router-view v-slot="{ Component }">
          <transition name="fade-slide" mode="out-in">
            <div :key="$route.path">
              <component :is="Component" />
            </div>
          </transition>
        </router-view>
      </a-layout-content>
    </a-layout>
  </a-layout>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useI18n } from 'vue-i18n'
import {
  DashboardOutlined,
  SettingOutlined,
  MenuFoldOutlined,
  MenuUnfoldOutlined,
  UserOutlined,
  DownOutlined,
  LogoutOutlined,
  CloseOutlined,
  TeamOutlined,
  MenuOutlined,
  ApartmentOutlined,
  BankOutlined,
  BookOutlined,
  ToolOutlined,
} from '@ant-design/icons-vue'
import { useAppStore } from '@/stores/app'
import { useUserStore } from '@/stores/user'
import type { MenuVo } from '@/api/menu'
import LocalePicker from '@/components/header/LocalePicker.vue'
import FontSizePicker from '@/components/header/FontSizePicker.vue'
import ThemePicker from '@/components/header/ThemePicker.vue'

const { t, te } = useI18n()

const MENU_PATH_I18N_MAP: Record<string, string> = {
  '/dashboard': 'menu.dashboard',
  '/system': 'menu.system',
  '/system/user': 'menu.user',
  '/system/role': 'menu.role',
  '/system/menu': 'menu.menu',
  '/system/dept': 'menu.dept',
  '/system/tenant': 'menu.tenant',
  '/system/dict': 'menu.dict',
  '/system/config': 'menu.config',
  '/system/monitor': 'menu.monitor',
  '/system/generator': 'menu.generator',
  '/profile': 'menu.profile',
}

function getMenuName(menu: MenuVo): string {
  const i18nKey = MENU_PATH_I18N_MAP[menu.path || '']
  if (i18nKey && te(i18nKey)) {
    return t(i18nKey)
  }
  return menu.menuName
}

const iconMap: Record<string, any> = {
  DashboardOutlined,
  SettingOutlined,
  UserOutlined,
  TeamOutlined,
  MenuOutlined,
  ApartmentOutlined,
  BankOutlined,
  BookOutlined,
  ToolOutlined,
}

function getIcon(icon?: string | null) {
  if (!icon) return SettingOutlined
  return iconMap[icon] || SettingOutlined
}

const route = useRoute()
const router = useRouter()
const appStore = useAppStore()
const userStore = useUserStore()

const selectedKeys = ref<string[]>([route.path])
const openKeys = ref<string[]>([])

interface TagView {
  path: string
  titleKey: string
}

const visitedViews = ref<TagView[]>([{ path: '/dashboard', titleKey: 'route.dashboard' }])

const breadcrumbs = computed(() => {
  const matched = route.matched.filter((item) => item.meta?.title)
  return matched.map((item) => {
    const titleKey = item.meta.title as string
    const isI18nKey = titleKey.includes('.')
    return {
      path: item.redirect ? undefined : item.path,
      title: isI18nKey ? t(titleKey) : titleKey,
    }
  })
})

function getTagTitle(tag: TagView): string {
  return tag.titleKey.includes('.') ? t(tag.titleKey) : tag.titleKey
}

watch(
  () => route.path,
  (val) => {
    selectedKeys.value = [val]
    const exists = visitedViews.value.find((v) => v.path === val)
    if (!exists && route.meta?.title) {
      visitedViews.value.push({ path: val, titleKey: route.meta.title as string })
    }
  }
)

function onMenuClick({ key }: { key: string }) {
  router.push(key)
}

function closeTag(tag: TagView) {
  const idx = visitedViews.value.findIndex((v) => v.path === tag.path)
  if (idx > -1) {
    visitedViews.value.splice(idx, 1)
    if (tag.path === route.path) {
      const last = visitedViews.value[visitedViews.value.length - 1]
      if (last) router.push(last.path)
    }
  }
}

async function doLogout() {
  await userStore.logout()
  router.push('/login')
}
</script>

<style scoped>
.sider {
  background: linear-gradient(180deg, #001529 0%, #002140 100%) !important;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.15);
}

.sider :deep(.ant-layout-sider-children) {
  display: flex;
  flex-direction: column;
}

.sider-menu {
  flex: 1;
  border-right: none;
}

.logo {
  height: 64px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 0 16px;
  overflow: hidden;
}

.logo-icon {
  width: 36px;
  height: 36px;
  border-radius: 8px;
  background: linear-gradient(135deg, #1890ff, #36cfc9);
  display: flex;
  align-items: center;
  justify-content: center;
  color: #fff;
  font-size: 20px;
  flex-shrink: 0;
}

.logo-text {
  color: #fff;
  font-size: 18px;
  font-weight: 700;
  white-space: nowrap;
  background: linear-gradient(90deg, #e6f7ff, #bae7ff);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.header {
  background: var(--header-bg, #fff);
  padding: 0 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  box-shadow: 0 1px 4px rgba(0, 21, 41, 0.08);
  z-index: 10;
  height: 56px;
  line-height: 56px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.collapse-btn {
  font-size: 18px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: all 0.3s;
  color: var(--header-text-secondary, rgba(0, 0, 0, 0.65));
}

.collapse-btn:hover {
  background: var(--header-hover-bg, rgba(0, 0, 0, 0.04));
  color: #1890ff;
}

.breadcrumb {
  font-size: 14px;
}

.header-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.header-right :deep(.header-action) {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.3s;
  font-size: 16px;
  color: var(--header-text-secondary, rgba(0, 0, 0, 0.65));
}

.header-right :deep(.header-action:hover) {
  background: var(--header-hover-bg, rgba(0, 0, 0, 0.04));
  color: #1890ff;
}

.user-info {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 6px;
  transition: all 0.3s;
  margin-left: 4px;
}

.user-info:hover {
  background: var(--header-hover-bg, rgba(0, 0, 0, 0.04));
}

.user-avatar {
  background: linear-gradient(135deg, #1890ff, #36cfc9);
}

.user-name {
  font-size: 14px;
  color: var(--header-text, rgba(0, 0, 0, 0.85));
}

.tags-bar {
  background: var(--component-bg, #fff);
  border-bottom: 1px solid var(--border-color, #f0f0f0);
  padding: 4px 16px;
}

.tags-scroll {
  display: flex;
  gap: 6px;
  overflow-x: auto;
  scrollbar-width: none;
}

.tags-scroll::-webkit-scrollbar {
  display: none;
}

.tag-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 2px 12px;
  font-size: 12px;
  border: 1px solid var(--border-color, #d9d9d9);
  border-radius: 4px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.3s;
  color: var(--header-text-secondary, rgba(0, 0, 0, 0.65));
  background: var(--component-bg, #fff);
}

.tag-item:hover {
  color: #1890ff;
  border-color: #1890ff;
}

.tag-item.active {
  color: #1890ff;
  background: #e6f7ff;
  border-color: #1890ff;
}

.tag-close {
  font-size: 10px;
  margin-left: 2px;
  border-radius: 50%;
  padding: 1px;
  transition: all 0.2s;
}

.tag-close:hover {
  background: #1890ff;
  color: #fff;
}

.content {
  margin: 16px;
  min-height: 280px;
}

.fade-slide-enter-active,
.fade-slide-leave-active {
  transition: all 0.3s ease;
}

.fade-slide-enter-from {
  opacity: 0;
  transform: translateX(-16px);
}

.fade-slide-leave-to {
  opacity: 0;
  transform: translateX(16px);
}

@media (max-width: 768px) {
  .header {
    padding: 0 12px;
  }

  .breadcrumb {
    display: none;
  }

  .user-name {
    display: none;
  }

  .header-right {
    gap: 2px;
  }
}
</style>
