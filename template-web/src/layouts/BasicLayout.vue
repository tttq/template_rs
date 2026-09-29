<template>
  <a-layout style="height: 100vh">
    <a-layout-sider
      v-model:collapsed="appStore.collapsed"
      collapsible
      :trigger="null"
      width="240"
      class="sider"
    >
      <div class="logo">
        <svg
          viewBox="0 0 40 40"
          fill="none"
          xmlns="http://www.w3.org/2000/svg"
          class="logo-mark"
        >
          <rect width="40" height="40" rx="10" fill="rgba(255,255,255,0.2)" />
          <path
            d="M12 20L18 26L28 14"
            stroke="white"
            stroke-width="3"
            stroke-linecap="round"
            stroke-linejoin="round"
          />
        </svg>
        <span
          v-if="!appStore.collapsed"
          class="logo-text"
        >Template Admin</span>
      </div>
      <a-menu
        v-model:selected-keys="selectedKeys"
        v-model:open-keys="openKeys"
        mode="inline"
        theme="dark"
        class="sider-menu"
        @click="onMenuClick"
      >
        <template
          v-for="menu in userStore.menus"
          :key="menu.path || menu.id"
        >
          <a-menu-item
            v-if="!menu.children?.length"
            :key="menu.path || String(menu.id)"
          >
            <component :is="getIcon(menu.icon)" />
            <span>{{ getMenuName(menu) }}</span>
          </a-menu-item>
          <a-sub-menu
            v-else
            :key="menu.path || String(menu.id)"
          >
            <template #icon>
              <component :is="getIcon(menu.icon)" />
            </template>
            <template #title>
              {{ getMenuName(menu) }}
            </template>
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
          <span
            class="collapse-btn"
            @click="appStore.toggleCollapsed()"
          >
            <MenuFoldOutlined v-if="!appStore.collapsed" />
            <MenuUnfoldOutlined v-else />
          </span>
          <a-breadcrumb class="breadcrumb">
            <a-breadcrumb-item
              v-for="item in breadcrumbs"
              :key="item.path"
            >
              <router-link
                v-if="item.path"
                :to="item.path"
              >
                {{ item.title }}
              </router-link>
              <span v-else>{{ item.title }}</span>
            </a-breadcrumb-item>
          </a-breadcrumb>
        </div>
        <div class="header-right">
          <NotificationBell />
          <LocalePicker />
          <FontSizePicker />
          <ThemePicker />
          <a-dropdown>
            <div class="user-info">
              <a-avatar
                :size="32"
                :src="userStore.userInfo?.avatar"
                class="user-avatar"
              >
                <template #icon>
                  <UserOutlined />
                </template>
              </a-avatar>
              <span class="user-name">{{ userStore.userInfo?.nickName || userStore.userInfo?.userName || t('common.user') }}</span>
              <DownOutlined />
            </div>
            <template #overlay>
              <a-menu>
                <a-menu-item
                  key="center"
                  @click="$router.push('/profile')"
                >
                  <UserOutlined />
                  <span style="margin-left: 8px">{{ t('header.profile') }}</span>
                </a-menu-item>
                <a-menu-item
                  key="settings"
                  @click="$router.push('/system/config')"
                >
                  <SettingOutlined />
                  <span style="margin-left: 8px">{{ t('header.systemConfig') }}</span>
                </a-menu-item>
                <a-menu-divider />
                <a-menu-item
                  key="logout"
                  @click="doLogout"
                >
                  <LogoutOutlined />
                  <span style="margin-left: 8px">{{ t('header.logout') }}</span>
                </a-menu-item>
              </a-menu>
            </template>
          </a-dropdown>
        </div>
      </a-layout-header>
      <div
        v-if="visitedViews.length > 0"
        class="tags-bar"
      >
        <div class="tags-scroll">
          <a-dropdown
            v-for="tag in visitedViews"
            :key="tag.path"
            :trigger="['contextmenu']"
          >
            <span
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
            <template #overlay>
              <a-menu @click="onTabAction($event, tag)">
                <a-menu-item key="refresh">
                  <ReloadOutlined />
                  <span style="margin-left: 8px">{{ t('tabs.refresh') }}</span>
                </a-menu-item>
                <a-menu-item key="closeOthers">
                  <SwitcherOutlined />
                  <span style="margin-left: 8px">{{ t('tabs.closeOthers') }}</span>
                </a-menu-item>
                <a-menu-item
                  key="closeCurrent"
                  :disabled="tag.path === '/dashboard'"
                >
                  <CloseOutlined />
                  <span style="margin-left: 8px">{{ t('tabs.closeCurrent') }}</span>
                </a-menu-item>
                <a-menu-divider />
                <a-menu-item key="closeAll">
                  <CloseCircleOutlined />
                  <span style="margin-left: 8px">{{ t('tabs.closeAll') }}</span>
                </a-menu-item>
              </a-menu>
            </template>
          </a-dropdown>
        </div>
      </div>
      <a-layout-content class="content">
        <!-- 注意：不能用 <transition> 包 keep-alive —— 列表页为多根模板（弹窗在根节点外），
             Transition 无法动画多根组件会产生大量 Vue warn；
             且 transition 下 keep-alive 要求组件单根，二者只能取 keep-alive -->
        <router-view v-slot="{ Component }">
          <keep-alive :max="20">
            <component
              :is="Component"
              :key="`${route.path}${reloadFlag}`"
            />
          </keep-alive>
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
  CloseCircleOutlined,
  ReloadOutlined,
  SwitcherOutlined,
  TeamOutlined,
  MenuOutlined,
  ApartmentOutlined,
  BankOutlined,
  BookOutlined,
  ToolOutlined,
  CodeOutlined,
  RobotOutlined,
  ApiOutlined,
  TagsOutlined,
  KeyOutlined,
  PayCircleOutlined,
  WalletOutlined,
  GiftOutlined,
  BarChartOutlined,
  FileSearchOutlined,
  LineChartOutlined,
  SafetyCertificateOutlined,
  AlertOutlined,
  AccountBookOutlined,
  ClockCircleOutlined,
  UserSwitchOutlined,
  BellOutlined,
  ShareAltOutlined,
  ExperimentOutlined,
  ShoppingOutlined,
  ShopOutlined,
  AppstoreOutlined,
  AuditOutlined,
  DollarOutlined,
  PaperClipOutlined,
  FileTextOutlined,
  CloudServerOutlined,
  DownloadOutlined,
} from '@ant-design/icons-vue'
import { useAppStore } from '@/stores/app'
import { useUserStore } from '@/stores/user'
import type { MenuVo } from '@/api/menu'
import LocalePicker from '@/components/header/LocalePicker.vue'
import FontSizePicker from '@/components/header/FontSizePicker.vue'
import ThemePicker from '@/components/header/ThemePicker.vue'
import NotificationBell from '@/components/header/NotificationBell.vue'

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
  '/system/attachment': 'menu.attachment',
  '/system/notification': 'menu.notification',
  '/system/notification-template': 'menu.notificationTemplate',
  '/system/client': 'menu.client',
  '/system/export': 'menu.export',
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
  CodeOutlined,
  RobotOutlined,
  ApiOutlined,
  TagsOutlined,
  KeyOutlined,
  PayCircleOutlined,
  WalletOutlined,
  GiftOutlined,
  BarChartOutlined,
  FileSearchOutlined,
  LineChartOutlined,
  SafetyCertificateOutlined,
  AlertOutlined,
  AccountBookOutlined,
  ClockCircleOutlined,
  UserSwitchOutlined,
  BellOutlined,
  ShareAltOutlined,
  ExperimentOutlined,
  ShoppingOutlined,
  ShopOutlined,
  AppstoreOutlined,
  AuditOutlined,
  DollarOutlined,
  PaperClipOutlined,
  FileTextOutlined,
  CloudServerOutlined,
  DownloadOutlined,
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

/**
 * 在菜单树中查找 path 对应的 key 链（含自身）。
 * 例：/system/user → ['/system', '/system/user']；根级 /dashboard → ['/dashboard']
 */
function findMenuChain(menus: MenuVo[], targetPath: string, trail: string[] = []): string[] | null {
  for (const m of menus) {
    const key = m.path || String(m.id)
    if (m.path === targetPath) return [...trail, key]
    if (m.children?.length) {
      const found = findMenuChain(m.children, targetPath, [...trail, key])
      if (found) return found
    }
  }
  return null
}

/** 按当前路由同步侧边栏：选中项 + 自动展开其父级目录（保留用户手动展开的其他分组） */
function syncMenuState() {
  selectedKeys.value = [route.path]
  const chain = findMenuChain(userStore.menus, route.path)
  const parents = chain ? chain.slice(0, -1) : []
  openKeys.value = Array.from(new Set([...openKeys.value, ...parents]))
}

interface TagView {
  path: string
  titleKey: string
}

const visitedViews = ref<TagView[]>([{ path: '/dashboard', titleKey: 'route.dashboard' }])

// 刷新后只保留仪表盘和当前页
if (route.path !== '/dashboard' && route.meta?.title) {
  visitedViews.value.push({ path: route.path, titleKey: route.meta.title as string })
}

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

const reloadFlag = ref(0)
/** 最近一次路由记录的 path，用于识别「同页仅 query 变化」的场景 */
let lastRoutedPath = route.path

watch(
  () => route.fullPath,
  () => {
    if (route.path === lastRoutedPath) {
      // keep-alive 下同页仅 query 变化（如详情 ?id=xxx 切换）必须重建实例，
      // 否则命中旧缓存展示上一笔数据
      reloadFlag.value++
    }
    lastRoutedPath = route.path
    syncMenuState()
    const exists = visitedViews.value.find((v) => v.path === route.path)
    if (!exists && route.meta?.title) {
      visitedViews.value.push({ path: route.path, titleKey: route.meta.title as string })
    }
  }
)

// 刷新/异步加载菜单后（menus 由守卫拉取，可能晚于本组件挂载）同步一次
watch(
  () => userStore.menus,
  () => syncMenuState(),
  { deep: true }
)
syncMenuState()

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

type TabMenuKey = 'refresh' | 'closeAll' | 'closeOthers' | 'closeCurrent'

function onTabAction({ key }: { key: string | number }, tag: TagView) {
  switch (key as TabMenuKey) {
    case 'refresh':
      if (tag.path !== route.path) router.push(tag.path)
      reloadFlag.value++
      break
    case 'closeAll':
      visitedViews.value = [{ path: '/dashboard', titleKey: 'route.dashboard' }]
      if (route.path !== '/dashboard') router.push('/dashboard')
      break
    case 'closeOthers':
      visitedViews.value = visitedViews.value.filter(
        (v) => v.path === '/dashboard' || v.path === tag.path
      )
      if (tag.path !== route.path) router.push(tag.path)
      break
    case 'closeCurrent':
      closeTag(tag)
      break
  }
}

async function doLogout() {
  await userStore.logout()
  router.push('/login')
}
</script>

<style scoped>
.sider {
  background: var(--sider-bg) !important;
  box-shadow: 2px 0 8px rgba(0, 0, 0, 0.15);
  overflow: hidden;
}

.sider :deep(.ant-layout-sider-children) {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.logo {
  height: 64px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 0 16px;
  overflow: hidden;
  border-bottom: 1px solid var(--sider-border);
}

.sider-menu {
  flex: 1;
  min-height: 0;
  border-right: none;
  overflow-y: auto;
  overflow-x: hidden;
  background: transparent;
}

/* 侧栏菜单配色统一走品牌变量（避免 antd 暗色主题默认蓝，与品牌绿冲突） */
.sider :deep(.ant-menu-dark),
.sider :deep(.ant-menu-dark .ant-menu-sub) {
  background: transparent;
}

.sider :deep(.ant-menu-dark .ant-menu-item),
.sider :deep(.ant-menu-dark .ant-menu-submenu-title) {
  color: var(--sider-text);
}

.sider :deep(.ant-menu-dark .ant-menu-item:hover),
.sider :deep(.ant-menu-dark .ant-menu-submenu-title:hover) {
  background: var(--sider-item-hover-bg);
  color: var(--sider-text-active);
}

.sider :deep(.ant-menu-dark .ant-menu-item-selected) {
  background: var(--sider-item-active-bg);
  color: var(--sider-text-active);
}

.sider :deep(.ant-menu-dark .ant-menu-submenu-selected > .ant-menu-submenu-title) {
  color: var(--sider-text-active);
}

.logo-text {
  color: var(--sider-logo-text);
  font-size: 18px;
  font-weight: 700;
  letter-spacing: 1px;
  white-space: nowrap;
}

.logo-mark {
  width: 32px;
  height: 32px;
  flex-shrink: 0;
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
  color: var(--brand-primary);
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
  color: var(--brand-primary);
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
  background: var(--brand-gradient);
}

.user-name {
  font-size: 14px;
  color: var(--header-text, rgba(0, 0, 0, 0.85));
}

.tags-bar {
  background: var(--component-bg, #fff);
  border-bottom: 1px solid var(--border-color, #f0f0f0);
  padding: 6px 16px;
}

.tags-scroll {
  display: flex;
  align-items: center;
  gap: 8px;
  overflow-x: auto;
  scrollbar-width: none;
}

.tags-scroll::-webkit-scrollbar {
  display: none;
}

.tag-item {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 14px;
  font-size: 13px;
  border: 1px solid var(--border-color, #d9d9d9);
  border-radius: 4px;
  cursor: pointer;
  white-space: nowrap;
  transition: all 0.3s;
  color: var(--header-text-secondary, rgba(0, 0, 0, 0.65));
  background: var(--component-bg, #fff);
}

.tag-item:hover {
  color: var(--brand-primary);
  border-color: var(--brand-primary);
}

.tag-item.active {
  color: var(--brand-primary);
  background: var(--brand-primary-light);
  border-color: var(--brand-primary);
}

.tag-close {
  font-size: 11px;
  margin-left: 2px;
  border-radius: 50%;
  padding: 1px;
  transition: all 0.2s;
}

.tag-close:hover {
  background: var(--brand-primary);
  color: #fff;
}

.content {
  margin: 16px;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
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
