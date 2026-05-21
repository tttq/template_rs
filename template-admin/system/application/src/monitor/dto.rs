use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisInfoVo {
    pub version: String,
    pub mode: String,
    pub uptime_in_seconds: i64,
    pub connected_clients: i64,
    pub used_memory_human: String,
    pub total_system_memory_human: String,
    pub maxmemory_human: String,
    pub keyspace_hits: i64,
    pub keyspace_misses: i64,
    pub hit_rate: String,
    pub db_size: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisKeyVo {
    pub key: String,
    pub key_type: String,
    pub ttl: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisValueVo {
    pub key: String,
    pub key_type: String,
    pub value: String,
    pub ttl: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfoVo {
    pub usage: f64,
    pub core_count: usize,
    pub load_avg_1: f64,
    pub load_avg_5: f64,
    pub load_avg_15: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfoVo {
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskInfoVo {
    pub name: String,
    pub total: u64,
    pub used: u64,
    pub available: u64,
    pub usage_percent: f64,
    pub mount_point: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatabasePoolVo {
    pub active_connections: i64,
    pub total_connections: i64,
    pub max_connections: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantDataSourceVo {
    pub tenant_id: String,
    pub tenant_name: String,
    pub status: String,
    pub active_connections: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TenantInfoVo {
    pub total_count: i64,
    pub enabled_count: i64,
    pub disabled_count: i64,
    pub mode: String,
    pub data_sources: Vec<TenantDataSourceVo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerInfoVo {
    pub cpu: CpuInfoVo,
    pub memory: MemoryInfoVo,
    pub disks: Vec<DiskInfoVo>,
    pub database_pool: DatabasePoolVo,
    pub tenant: TenantInfoVo,
}
