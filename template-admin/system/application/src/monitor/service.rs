use common::error::AppError;
use summer::plugin::service::Service;
use sea_orm_ext::DbConn;
use summer_redis::Redis;
use sea_orm::{ConnectionTrait, Statement, DatabaseBackend};
use sysinfo::{Disks, System};
use system_entity::tenant;

use super::dto::*;

#[derive(Clone, Service)]
pub struct MonitorAppService {
    #[inject(component)]
    db: DbConn,
    #[inject(component)]
    redis: Redis,
}

impl MonitorAppService {
    pub async fn redis_info(&self) -> Result<RedisInfoVo, AppError> {
        let info: String = summer_redis::redis::cmd("INFO")
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|e| AppError::Internal(format!("Redis INFO 失败: {}", e)))?;

        let db_size: i64 = summer_redis::redis::cmd("DBSIZE")
            .query_async(&mut self.redis.clone())
            .await
            .map_err(|e| AppError::Internal(format!("Redis DBSIZE 失败: {}", e)))?;

        let info_map = parse_redis_info(&info);

        let keyspace_hits = info_map.get("keyspace_hits").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
        let keyspace_misses = info_map.get("keyspace_misses").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0);
        let total_hits = keyspace_hits + keyspace_misses;
        let hit_rate = if total_hits > 0 {
            format!("{:.2}%", (keyspace_hits as f64 / total_hits as f64) * 100.0)
        } else {
            "0%".to_string()
        };

        Ok(RedisInfoVo {
            version: info_map.get("redis_version").cloned().unwrap_or_default(),
            mode: info_map.get("redis_mode").cloned().unwrap_or_default(),
            uptime_in_seconds: info_map.get("uptime_in_seconds").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0),
            connected_clients: info_map.get("connected_clients").and_then(|v| v.parse::<i64>().ok()).unwrap_or(0),
            used_memory_human: info_map.get("used_memory_human").cloned().unwrap_or_default(),
            total_system_memory_human: info_map.get("total_system_memory_human").cloned().unwrap_or_default(),
            maxmemory_human: {
                let val = info_map.get("maxmemory").and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                if val == 0 { "unlimited".to_string() } else { format_bytes(val) }
            },
            keyspace_hits,
            keyspace_misses,
            hit_rate,
            db_size,
            status: "UP".to_string(),
        })
    }

    pub async fn server_info(&self) -> Result<ServerInfoVo, AppError> {
        let cpu = self.get_cpu_info();
        let memory = self.get_memory_info();
        let disks = self.get_disk_info();
        let database_pool = self.get_database_pool_info().await?;
        let tenant = self.get_tenant_info().await?;

        Ok(ServerInfoVo {
            cpu,
            memory,
            disks,
            database_pool,
            tenant,
        })
    }

    fn get_cpu_info(&self) -> CpuInfoVo {
        let core_count = num_cpus::get();
        let mut sys = System::new();
        sys.refresh_cpu_usage();
        let usage = sys.global_cpu_usage() as f64;
        let load = System::load_average();
        CpuInfoVo {
            usage,
            core_count,
            load_avg_1: load.one,
            load_avg_5: load.five,
            load_avg_15: load.fifteen,
        }
    }

    fn get_memory_info(&self) -> MemoryInfoVo {
        let mut sys = System::new();
        sys.refresh_memory();
        let total = sys.total_memory();
        let available = sys.available_memory();
        let used = total - available;
        let usage_percent = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };
        MemoryInfoVo { total, used, available, usage_percent }
    }

    fn get_disk_info(&self) -> Vec<DiskInfoVo> {
        let disks = Disks::new_with_refreshed_list();
        let mut result = Vec::new();
        for disk in disks.list() {
            let total = disk.total_space();
            let available = disk.available_space();
            let used = total - available;
            let usage_percent = if total > 0 { (used as f64 / total as f64) * 100.0 } else { 0.0 };
            result.push(DiskInfoVo {
                name: disk.name().to_string_lossy().to_string(),
                total,
                used,
                available,
                usage_percent,
                mount_point: disk.mount_point().to_string_lossy().to_string(),
            });
        }
        result
    }

    async fn get_database_pool_info(&self) -> Result<DatabasePoolVo, AppError> {
        let active_sql = "SELECT count(*) as cnt FROM pg_stat_activity WHERE state = 'active' AND datname = current_database()";
        let total_sql = "SELECT count(*) as cnt FROM pg_stat_activity WHERE datname = current_database()";
        let max_sql = "SELECT setting::int as cnt FROM pg_settings WHERE name = 'max_connections'";

        let active: i64 = self.db.query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres, active_sql,
        )).await.map_err(|e| AppError::Internal(e.to_string()))?
            .and_then(|row| row.try_get::<i64>("", "cnt").ok())
            .unwrap_or(0);

        let total: i64 = self.db.query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres, total_sql,
        )).await.map_err(|e| AppError::Internal(e.to_string()))?
            .and_then(|row| row.try_get::<i64>("", "cnt").ok())
            .unwrap_or(0);

        let max: i64 = self.db.query_one_raw(Statement::from_string(
            DatabaseBackend::Postgres, max_sql,
        )).await.map_err(|e| AppError::Internal(e.to_string()))?
            .and_then(|row| row.try_get::<i64>("", "cnt").ok())
            .unwrap_or(100);

        Ok(DatabasePoolVo {
            active_connections: active,
            total_connections: total,
            max_connections: max,
        })
    }

    async fn get_tenant_info(&self) -> Result<TenantInfoVo, AppError> {
        let all_tenants = tenant::Entity::find()
            .all(&self.db)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let total_count = all_tenants.len() as i64;
        let enabled_count = all_tenants.iter().filter(|t| t.status == 1).count() as i64;
        let disabled_count = total_count - enabled_count;
        let data_sources = all_tenants.iter().map(|t| {
            TenantDataSourceVo {
                tenant_id: t.id.to_string(),
                tenant_name: t.tenant_name.clone(),
                status: if t.status == 1 { "enabled".to_string() } else { "disabled".to_string() },
                active_connections: 0,
            }
        }).collect();
        Ok(TenantInfoVo {
            total_count,
            enabled_count,
            disabled_count,
            mode: "table".to_string(),
            data_sources,
        })
    }
}

fn parse_redis_info(info: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    for line in info.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once(':') {
            map.insert(key.to_string(), value.to_string());
        }
    }
    map
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    if bytes >= GB {
        format!("{:.2}GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2}MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2}KB", bytes as f64 / KB as f64)
    } else {
        format!("{}B", bytes)
    }
}
