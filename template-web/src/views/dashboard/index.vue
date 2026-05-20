<template>
  <div class="dashboard">
    <div class="welcome-section">
      <div class="welcome-content">
        <h2 class="welcome-title">
          {{ greeting }}，{{ userStore.userInfo?.nickName || userStore.userInfo?.userName || '用户' }}
        </h2>
        <p class="welcome-desc">欢迎回到 Template Admin 管理系统，祝您今天工作愉快！</p>
      </div>
    </div>

    <a-row :gutter="[16, 16]" class="stat-row">
      <a-col :xs="24" :sm="12" :lg="6">
        <a-card :bordered="false" class="stat-card stat-card--blue">
          <div class="stat-card-inner">
            <div class="stat-info">
              <div class="stat-label">用户数</div>
              <div class="stat-value">{{ stats.userCount }}</div>
            </div>
            <div class="stat-icon">
              <UserOutlined />
            </div>
          </div>
        </a-card>
      </a-col>
      <a-col :xs="24" :sm="12" :lg="6">
        <a-card :bordered="false" class="stat-card stat-card--green">
          <div class="stat-card-inner">
            <div class="stat-info">
              <div class="stat-label">角色数</div>
              <div class="stat-value">{{ stats.roleCount }}</div>
            </div>
            <div class="stat-icon">
              <TeamOutlined />
            </div>
          </div>
        </a-card>
      </a-col>
      <a-col :xs="24" :sm="12" :lg="6">
        <a-card :bordered="false" class="stat-card stat-card--orange">
          <div class="stat-card-inner">
            <div class="stat-info">
              <div class="stat-label">菜单数</div>
              <div class="stat-value">{{ stats.menuCount }}</div>
            </div>
            <div class="stat-icon">
              <MenuOutlined />
            </div>
          </div>
        </a-card>
      </a-col>
      <a-col :xs="24" :sm="12" :lg="6">
        <a-card :bordered="false" class="stat-card stat-card--purple">
          <div class="stat-card-inner">
            <div class="stat-info">
              <div class="stat-label">租户数</div>
              <div class="stat-value">{{ stats.tenantCount }}</div>
            </div>
            <div class="stat-icon">
              <HomeOutlined />
            </div>
          </div>
        </a-card>
      </a-col>
    </a-row>

    <a-row :gutter="[16, 16]" style="margin-top: 16px">
      <a-col :xs="24" :lg="16">
        <a-card title="快捷操作" :bordered="false" class="quick-card">
          <a-row :gutter="[16, 16]">
            <a-col :span="8">
              <div class="quick-item" @click="$router.push('/system/user')">
                <UserOutlined class="quick-icon" style="color: #1890ff" />
                <span>用户管理</span>
              </div>
            </a-col>
            <a-col :span="8">
              <div class="quick-item" @click="$router.push('/system/role')">
                <TeamOutlined class="quick-icon" style="color: #52c41a" />
                <span>角色管理</span>
              </div>
            </a-col>
            <a-col :span="8">
              <div class="quick-item" @click="$router.push('/system/menu')">
                <MenuOutlined class="quick-icon" style="color: #fa8c16" />
                <span>菜单管理</span>
              </div>
            </a-col>
            <a-col :span="8">
              <div class="quick-item" @click="$router.push('/system/dept')">
                <ApartmentOutlined class="quick-icon" style="color: #722ed1" />
                <span>部门管理</span>
              </div>
            </a-col>
            <a-col :span="8">
              <div class="quick-item" @click="$router.push('/system/dict')">
                <ReadOutlined class="quick-icon" style="color: #eb2f96" />
                <span>字典管理</span>
              </div>
            </a-col>
            <a-col :span="8">
              <div class="quick-item" @click="$router.push('/system/config')">
                <SettingOutlined class="quick-icon" style="color: #13c2c2" />
                <span>参数配置</span>
              </div>
            </a-col>
          </a-row>
        </a-card>
      </a-col>
      <a-col :xs="24" :lg="8">
        <a-card title="系统信息" :bordered="false" class="info-card">
          <div class="info-item">
            <span class="info-label">系统版本</span>
            <span class="info-value">1.0.0</span>
          </div>
          <div class="info-item">
            <span class="info-label">前端框架</span>
            <span class="info-value">Vue 3 + Ant Design Vue</span>
          </div>
          <div class="info-item">
            <span class="info-label">后端框架</span>
            <span class="info-value">Summer-rs + Sea-ORM</span>
          </div>
          <div class="info-item">
            <span class="info-label">数据库</span>
            <span class="info-value">PostgreSQL 17</span>
          </div>
          <div class="info-item">
            <span class="info-label">缓存</span>
            <span class="info-value">Redis 7</span>
          </div>
          <div class="info-item">
            <span class="info-label">当前用户</span>
            <span class="info-value">{{ userStore.userInfo?.userName || '-' }}</span>
          </div>
          <div class="info-item">
            <span class="info-label">角色</span>
            <span class="info-value">{{ userStore.roles.join(', ') || '-' }}</span>
          </div>
        </a-card>
      </a-col>
    </a-row>
  </div>
</template>

<script setup lang="ts">
import { computed, reactive, onMounted } from 'vue'
import {
  UserOutlined,
  TeamOutlined,
  MenuOutlined,
  HomeOutlined,
  ApartmentOutlined,
  ReadOutlined,
  SettingOutlined,
} from '@ant-design/icons-vue'
import { useUserStore } from '@/stores/user'
import { userApi } from '@/api/user'
import { roleApi } from '@/api/role'
import { menuApi } from '@/api/menu'
import { tenantApi } from '@/api/tenant'

const userStore = useUserStore()

const stats = reactive({
  userCount: 0,
  roleCount: 0,
  menuCount: 0,
  tenantCount: 0,
})

const greeting = computed(() => {
  const hour = new Date().getHours()
  if (hour < 6) return '凌晨好'
  if (hour < 9) return '早上好'
  if (hour < 12) return '上午好'
  if (hour < 14) return '中午好'
  if (hour < 18) return '下午好'
  return '晚上好'
})

function countTreeItems(items: any[]): number {
  let count = 0
  for (const item of items) {
    count++
    if (item.children?.length) {
      count += countTreeItems(item.children)
    }
  }
  return count
}

onMounted(async () => {
  try {
    const [userRes, roles, menus, tenantRes] = await Promise.all([
      userApi.list({ page: 1, pageSize: 1 }),
      roleApi.listAll(),
      menuApi.listTree(),
      tenantApi.list({ page: 1, pageSize: 1 }),
    ])
    stats.userCount = userRes.total
    stats.roleCount = Array.isArray(roles) ? countTreeItems(roles) : 0
    stats.menuCount = Array.isArray(menus) ? countTreeItems(menus) : 0
    stats.tenantCount = tenantRes.total
  } catch {}
})
</script>

<style scoped>
.dashboard {
  padding: 4px 0;
}

.welcome-section {
  background: linear-gradient(135deg, #1890ff 0%, #36cfc9 100%);
  border-radius: 8px;
  padding: 28px 32px;
  margin-bottom: 16px;
  color: #fff;
}

.welcome-title {
  font-size: 22px;
  font-weight: 600;
  margin: 0 0 8px;
  color: #fff;
}

.welcome-desc {
  font-size: 14px;
  margin: 0;
  opacity: 0.85;
}

.stat-card {
  border-radius: 8px;
  overflow: hidden;
  transition: all 0.3s;
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
}

.stat-card--blue {
  background: linear-gradient(135deg, #1890ff 0%, #69c0ff 100%);
}

.stat-card--green {
  background: linear-gradient(135deg, #52c41a 0%, #95de64 100%);
}

.stat-card--orange {
  background: linear-gradient(135deg, #fa8c16 0%, #ffc069 100%);
}

.stat-card--purple {
  background: linear-gradient(135deg, #722ed1 0%, #b37feb 100%);
}

.stat-card :deep(.ant-card-body) {
  padding: 20px 24px;
}

.stat-card-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.stat-info {
  display: flex;
  flex-direction: column;
}

.stat-label {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.85);
  margin-bottom: 8px;
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: #fff;
  line-height: 1;
}

.stat-icon {
  font-size: 40px;
  color: rgba(255, 255, 255, 0.3);
}

.quick-card {
  border-radius: 8px;
}

.quick-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 16px 8px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s;
  font-size: 14px;
  color: rgba(0, 0, 0, 0.65);
}

.quick-item:hover {
  background: #f5f5f5;
  color: #1890ff;
}

.quick-icon {
  font-size: 28px;
}

.info-card {
  border-radius: 8px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 0;
  border-bottom: 1px solid #f0f0f0;
}

.info-item:last-child {
  border-bottom: none;
}

.info-label {
  color: rgba(0, 0, 0, 0.45);
  font-size: 14px;
}

.info-value {
  color: rgba(0, 0, 0, 0.85);
  font-size: 14px;
}
</style>
