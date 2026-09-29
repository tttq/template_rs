<template>
  <div>
    <a-card
      title="服务监控"
      :bordered="false"
    >
      <template #extra>
        <a-button @click="fetchData">
          刷新
        </a-button>
      </template>
      <a-spin :spinning="loading">
        <a-tabs
          v-model:active-key="activeTab"
          @change="blurActiveFocus"
        >
          <a-tab-pane
            key="server"
            tab="服务器"
          >
            <a-row :gutter="[16, 16]">
              <a-col :span="12">
                <a-card
                  title="CPU"
                  size="small"
                >
                  <a-progress
                    :percent="Number(serverInfo.cpu?.usage) || 0"
                    :precision="1"
                  />
                  <a-descriptions
                    :column="2"
                    size="small"
                    class="mt-4"
                  >
                    <a-descriptions-item label="核心数">
                      {{ serverInfo.cpu?.coreCount }}
                    </a-descriptions-item>
                    <a-descriptions-item label="1分钟负载">
                      {{ serverInfo.cpu?.loadAvg1?.toFixed(2) }}
                    </a-descriptions-item>
                    <a-descriptions-item label="5分钟负载">
                      {{ serverInfo.cpu?.loadAvg5?.toFixed(2) }}
                    </a-descriptions-item>
                    <a-descriptions-item label="15分钟负载">
                      {{ serverInfo.cpu?.loadAvg15?.toFixed(2) }}
                    </a-descriptions-item>
                  </a-descriptions>
                </a-card>
              </a-col>
              <a-col :span="12">
                <a-card
                  title="内存"
                  size="small"
                >
                  <a-progress
                    :percent="Number(serverInfo.memory?.usagePercent) || 0"
                    :precision="1"
                  />
                  <a-descriptions
                    :column="2"
                    size="small"
                    class="mt-4"
                  >
                    <a-descriptions-item label="总量">
                      {{ formatBytes(serverInfo.memory?.total) }}
                    </a-descriptions-item>
                    <a-descriptions-item label="已用">
                      {{ formatBytes(serverInfo.memory?.used) }}
                    </a-descriptions-item>
                    <a-descriptions-item label="可用">
                      {{ formatBytes(serverInfo.memory?.available) }}
                    </a-descriptions-item>
                  </a-descriptions>
                </a-card>
              </a-col>
              <a-col :span="12">
                <a-card
                  title="数据库连接池"
                  size="small"
                >
                  <a-progress
                    :percent="dbPoolPercent"
                    :precision="1"
                  />
                  <a-descriptions
                    :column="3"
                    size="small"
                    class="mt-4"
                  >
                    <a-descriptions-item label="活跃连接">
                      {{ serverInfo.databasePool?.activeConnections }}
                    </a-descriptions-item>
                    <a-descriptions-item label="总连接">
                      {{ serverInfo.databasePool?.totalConnections }}
                    </a-descriptions-item>
                    <a-descriptions-item label="最大连接">
                      {{ serverInfo.databasePool?.maxConnections }}
                    </a-descriptions-item>
                  </a-descriptions>
                </a-card>
              </a-col>
              <a-col :span="12">
                <a-card
                  title="多租户"
                  size="small"
                >
                  <a-descriptions
                    :column="2"
                    size="small"
                  >
                    <a-descriptions-item label="租户总数">
                      {{ serverInfo.tenant?.totalCount }}
                    </a-descriptions-item>
                    <a-descriptions-item label="隔离模式">
                      {{ serverInfo.tenant?.mode }}
                    </a-descriptions-item>
                    <a-descriptions-item label="启用数">
                      <a-tag color="green">
                        {{ serverInfo.tenant?.enabledCount }}
                      </a-tag>
                    </a-descriptions-item>
                    <a-descriptions-item label="禁用数">
                      <a-tag color="red">
                        {{ serverInfo.tenant?.disabledCount }}
                      </a-tag>
                    </a-descriptions-item>
                  </a-descriptions>
                </a-card>
              </a-col>
              <a-col :span="24">
                <a-card
                  title="磁盘"
                  size="small"
                >
                  <a-table
                    :columns="diskColumns"
                    :data-source="serverInfo.disks || []"
                    :pagination="false"
                    size="small"
                    row-key="name"
                  >
                    <template #bodyCell="{ column, record, index }">
                      <template v-if="column.key === 'index'">
                        {{ index + 1 }}
                      </template>
                      <template v-if="column.dataIndex === 'usagePercent'">
                        <a-progress
                          :percent="Number(record.usagePercent)"
                          :precision="1"
                          size="small"
                        />
                      </template>
                      <template v-if="column.dataIndex === 'total'">
                        {{ formatBytes(record.total) }}
                      </template>
                      <template v-if="column.dataIndex === 'used'">
                        {{ formatBytes(record.used) }}
                      </template>
                      <template v-if="column.dataIndex === 'available'">
                        {{ formatBytes(record.available) }}
                      </template>
                    </template>
                  </a-table>
                </a-card>
              </a-col>
              <a-col :span="24">
                <a-card
                  title="租户数据源"
                  size="small"
                >
                  <a-table
                    :columns="tenantColumns"
                    :data-source="serverInfo.tenant?.dataSources || []"
                    :pagination="false"
                    size="small"
                    row-key="tenantId"
                  >
                    <template #bodyCell="{ column, record, index }">
                      <template v-if="column.key === 'index'">
                        {{ index + 1 }}
                      </template>
                      <template v-if="column.dataIndex === 'status'">
                        <a-tag :color="record.status === 'enabled' ? 'green' : 'red'">
                          {{ record.status }}
                        </a-tag>
                      </template>
                    </template>
                  </a-table>
                </a-card>
              </a-col>
            </a-row>
          </a-tab-pane>
          <a-tab-pane
            key="redis"
            tab="Redis"
          >
            <a-row :gutter="[16, 16]">
              <a-col :span="6">
                <a-card>
                  <a-statistic
                    title="状态"
                    :value="redisInfo.status"
                  >
                    <template #formatter>
                      <a-tag :color="redisInfo.status === 'UP' ? 'green' : 'red'">
                        {{ redisInfo.status }}
                      </a-tag>
                    </template>
                  </a-statistic>
                </a-card>
              </a-col>
              <a-col :span="6">
                <a-card>
                  <a-statistic
                    title="版本"
                    :value="redisInfo.version"
                  />
                </a-card>
              </a-col>
              <a-col :span="6">
                <a-card>
                  <a-statistic
                    title="运行模式"
                    :value="redisInfo.mode"
                  />
                </a-card>
              </a-col>
              <a-col :span="6">
                <a-card>
                  <a-statistic
                    title="运行时间(秒)"
                    :value="redisInfo.uptimeInSeconds"
                  />
                </a-card>
              </a-col>
              <a-col :span="6">
                <a-card>
                  <a-statistic
                    title="连接数"
                    :value="redisInfo.connectedClients"
                  />
                </a-card>
              </a-col>
              <a-col :span="6">
                <a-card>
                  <a-statistic
                    title="已用内存"
                    :value="redisInfo.usedMemoryHuman"
                  />
                </a-card>
              </a-col>
              <a-col :span="6">
                <a-card>
                  <a-statistic
                    title="总内存"
                    :value="redisInfo.totalSystemMemoryHuman"
                  />
                </a-card>
              </a-col>
              <a-col :span="6">
                <a-card>
                  <a-statistic
                    title="最大内存"
                    :value="redisInfo.maxmemoryHuman"
                  />
                </a-card>
              </a-col>
              <a-col :span="6">
                <a-card>
                  <a-statistic
                    title="缓存命中数"
                    :value="redisInfo.keyspaceHits"
                  />
                </a-card>
              </a-col>
              <a-col :span="6">
                <a-card>
                  <a-statistic
                    title="缓存未命中数"
                    :value="redisInfo.keyspaceMisses"
                  />
                </a-card>
              </a-col>
              <a-col :span="6">
                <a-card>
                  <a-statistic
                    title="命中率"
                    :value="redisInfo.hitRate"
                  />
                </a-card>
              </a-col>
              <a-col :span="6">
                <a-card>
                  <a-statistic
                    title="Key 数量"
                    :value="redisInfo.dbSize"
                  />
                </a-card>
              </a-col>
            </a-row>
          </a-tab-pane>
        </a-tabs>
      </a-spin>
    </a-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { monitorApi } from '@/api/monitor'
import { blurActiveFocus } from '@/utils/blurOnTabChange'

const loading = ref(false)
const activeTab = ref('server')
const serverInfo = ref<any>({})
const redisInfo = ref<any>({})

const dbPoolPercent = computed(() => {
  const pool = serverInfo.value.databasePool
  if (!pool || !pool.maxConnections) return 0
  return (pool.totalConnections / pool.maxConnections) * 100
})

const diskColumns = [
  { title: '序号', key: 'index', width: 56 },
  { title: '名称', dataIndex: 'name', width: 120 },
  { title: '挂载点', dataIndex: 'mountPoint', width: 150 },
  { title: '总量', dataIndex: 'total', width: 120 },
  { title: '已用', dataIndex: 'used', width: 120 },
  { title: '可用', dataIndex: 'available', width: 120 },
  { title: '使用率', dataIndex: 'usagePercent', width: 200 },
]

const tenantColumns = [
  { title: '序号', key: 'index', width: 56 },
  { title: '租户名称', dataIndex: 'tenantName', width: 200 },
  { title: '状态', dataIndex: 'status', width: 120 },
  { title: '活跃连接', dataIndex: 'activeConnections', width: 120 },
]

function formatBytes(bytes?: number | string) {
  const size = Number(bytes) || 0
  if (!size) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let i = 0
  let v = size
  while (v >= 1024 && i < units.length - 1) { v /= 1024; i++ }
  return `${v.toFixed(2)} ${units[i]}`
}

async function fetchData() {
  loading.value = true
  try {
    const [serverRes, redisRes] = await Promise.all([
      monitorApi.getServerInfo(),
      monitorApi.getRedisInfo(),
    ])
    serverInfo.value = serverRes
    redisInfo.value = redisRes
  } finally {
    loading.value = false
  }
}

onMounted(() => fetchData())
</script>
