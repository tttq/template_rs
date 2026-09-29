//! sa-token 会话残留清理（修复 Redis 无界膨胀）
//!
//! 背景：sa-token 的账号索引键写入时 TTL 固定为 `None`（永久有效，见 `save_string_list` /
//! `save_session`），且只在「显式登出某个 token」时才会缩减对应条目：
//! - `{prefix}login:tokens:{login_id}`  多设备 token 索引（每次登录追加）
//! - `{prefix}session:{login_id}`       账号会话（每次登录追加 `terminal_list`）
//! - `{prefix}refresh:user:{login_id}`  refresh token 用户索引（每次登录追加）
//!
//! 而 token 自然过期（1 天）、关闭浏览器、并发登录都不会触发登出，因此这三类永久键会随登录
//! 次数单调增长；叠加 sa-token 默认 `is_concurrent = true` + `max_login_count = -1`
//! （不限制并发登录），同一账号反复登录即导致 Redis 持续膨胀。
//!
//! 本模块提供两档清理（均不触碰 `{prefix}token:*`，不影响在线态与鉴权）：
//! - [`cleanup_user`]：单账号即时收敛，登录成功后调用
//! - [`sweep_all`]：全量扫描，启动执行一次 + 周期兜底，同时清理历史存量脏数据

use std::collections::HashSet;

use summer_redis::Redis;
use summer_redis::redis;

/// 清理统计（用于日志观测）
#[derive(Debug, Default, Clone)]
pub struct CleanupStat {
    /// 本次处理的账号数
    pub users: u64,
    /// 移除的失效索引条目数
    pub removed_entries: u64,
    /// 删除的空索引键数
    pub deleted_keys: u64,
}

impl CleanupStat {
    fn merge(&mut self, other: &CleanupStat) {
        self.users += other.users;
        self.removed_entries += other.removed_entries;
        self.deleted_keys += other.deleted_keys;
    }

    /// 是否有实际清理动作（用于抑制无意义日志）
    pub fn is_empty(&self) -> bool {
        self.removed_entries == 0 && self.deleted_keys == 0
    }
}

// ---------- 键名构造（与 sa-token `{{prefix}}{{suffix}}{{id}}` 规则一致） ----------

fn login_tokens_key(prefix: &str, login_id: &str) -> String {
    format!("{}login:tokens:{}", prefix, login_id)
}

fn session_key(prefix: &str, login_id: &str) -> String {
    format!("{}session:{}", prefix, login_id)
}

fn refresh_user_key(prefix: &str, login_id: &str) -> String {
    format!("{}refresh:user:{}", prefix, login_id)
}

fn token_key(prefix: &str, token: &str) -> String {
    format!("{}token:{}", prefix, token)
}

fn refresh_key(prefix: &str, refresh_token: &str) -> String {
    format!("{}refresh:{}", prefix, refresh_token)
}

// ---------- Redis 基础操作 ----------

async fn get_raw(conn: &mut Redis, key: &str) -> Option<String> {
    redis::cmd("GET")
        .arg(key)
        .query_async(conn)
        .await
        .ok()
        .flatten()
}

async fn get_string_list(conn: &mut Redis, key: &str) -> Option<Vec<String>> {
    get_raw(conn, key).await.and_then(|s| serde_json::from_str(&s).ok())
}

async fn exists(conn: &mut Redis, key: &str) -> bool {
    redis::cmd("EXISTS")
        .arg(key)
        .query_async::<i64>(conn)
        .await
        .unwrap_or(0)
        > 0
}

/// 回写字符串列表；列表为空则删除键（避免残留永久空键），非空保持无 TTL（与库语义一致）
async fn save_string_list(conn: &mut Redis, key: &str, list: &[String]) {
    if list.is_empty() {
        let _ = redis::cmd("DEL").arg(key).query_async::<i64>(conn).await;
    } else if let Ok(value) = serde_json::to_string(list) {
        let _ = redis::cmd("SET").arg(key).arg(value).query_async::<String>(conn).await;
    }
}

/// 清理单个账号的三类残留索引。
///
/// `keep_max` 为该账号允许保留的会话/refresh 索引上限（来自 `sa-token.max_login_count`，
/// `<= 0` 表示不修剪数量、只移除失效条目）。
pub async fn cleanup_user(
    redis: &Redis,
    prefix: &str,
    login_id: &str,
    keep_max: i64,
) -> CleanupStat {
    let mut conn = redis.clone();
    let mut stat = CleanupStat {
        users: 1,
        ..Default::default()
    };

    // 1. 多设备 token 索引：移除 token 已失效（过期/被踢）的条目
    let key = login_tokens_key(prefix, login_id);
    if let Some(list) = get_string_list(&mut conn, &key).await {
        let mut kept: Vec<String> = Vec::with_capacity(list.len());
        for token in &list {
            if exists(&mut conn, &token_key(prefix, token)).await {
                kept.push(token.clone());
            }
        }
        if kept.len() != list.len() {
            stat.removed_entries += (list.len() - kept.len()) as u64;
            stat.deleted_keys += u64::from(kept.is_empty());
            save_string_list(&mut conn, &key, &kept).await;
        }
    }

    // 2. refresh token 用户索引：移除记录已失效的条目，并按并发上限撤销最旧的多余 refresh token
    let key = refresh_user_key(prefix, login_id);
    if let Some(list) = get_string_list(&mut conn, &key).await {
        let mut kept: Vec<String> = Vec::with_capacity(list.len());
        for rt in &list {
            if exists(&mut conn, &refresh_key(prefix, rt)).await {
                kept.push(rt.clone());
            }
        }
        // 索引按追加顺序（由旧到新）排列，超出上限时从最旧的一端撤销
        if keep_max > 0 && kept.len() as i64 > keep_max {
            let drop_count = kept.len() - keep_max as usize;
            for rt in kept.drain(..drop_count) {
                let _ = redis::cmd("DEL")
                    .arg(refresh_key(prefix, &rt))
                    .query_async::<i64>(&mut conn)
                    .await;
            }
        }
        if kept.len() != list.len() {
            stat.removed_entries += (list.len() - kept.len()) as u64;
            stat.deleted_keys += u64::from(kept.is_empty());
            save_string_list(&mut conn, &key, &kept).await;
        }
    }

    // 3. 账号会话：移除 terminal_list 中 token 已失效的终端；全部失效则删除会话键
    let key = session_key(prefix, login_id);
    if let Some(raw) = get_raw(&mut conn, &key).await
        && let Ok(mut value) = serde_json::from_str::<serde_json::Value>(&raw)
        && let Some(terminals) = value
            .get_mut("terminal_list")
            .and_then(|v| v.as_array_mut())
    {
        let mut dead: HashSet<String> = HashSet::new();
        for terminal in terminals.iter() {
            if let Some(token) = terminal.get("token_value").and_then(|v| v.as_str())
                && !exists(&mut conn, &token_key(prefix, token)).await
            {
                dead.insert(token.to_string());
            }
        }
        let before = terminals.len();
        if !dead.is_empty() {
            terminals.retain(|t| {
                t.get("token_value")
                    .and_then(|v| v.as_str())
                    .map(|s| !dead.contains(s))
                    .unwrap_or(true)
            });
        }
        let removed = before - terminals.len();
        let empty = terminals.is_empty();
        if removed > 0 || empty {
            stat.removed_entries += removed as u64;
            stat.deleted_keys += u64::from(empty);
            if empty {
                let _ = redis::cmd("DEL").arg(&key).query_async::<i64>(&mut conn).await;
            } else if let Ok(serialized) = serde_json::to_string(&value) {
                let _ = redis::cmd("SET")
                    .arg(&key)
                    .arg(serialized)
                    .query_async::<String>(&mut conn)
                    .await;
            }
        }
    }

    stat
}

/// 撤销与指定 access token 绑定的 refresh token（登出时调用）。
///
/// 库的 `logout` 只删 access token，不回收 refresh token —— 登出后 7 天内仍可用
/// refresh token 换取新的 access token，既造成凭证残留也带来安全隐患。
pub async fn revoke_refresh_by_access_token(
    redis: &Redis,
    prefix: &str,
    login_id: &str,
    access_token: &str,
) -> u64 {
    let mut conn = redis.clone();
    let Some(list) = get_string_list(&mut conn, &refresh_user_key(prefix, login_id)).await else {
        return 0;
    };
    let mut revoked = 0u64;
    for rt in &list {
        let Some(raw) = get_raw(&mut conn, &refresh_key(prefix, rt)).await else {
            continue;
        };
        let bound = serde_json::from_str::<serde_json::Value>(&raw)
            .ok()
            .and_then(|v| v.get("access_token").and_then(|x| x.as_str()).map(String::from));
        if bound.as_deref() == Some(access_token) {
            let _ = redis::cmd("DEL")
                .arg(refresh_key(prefix, rt))
                .query_async::<i64>(&mut conn)
                .await;
            revoked += 1;
        }
    }
    if revoked > 0 {
        // 同步收敛用户索引，避免留下指向已删记录的条目
        let _ = cleanup_user(redis, prefix, login_id, 0).await;
    }
    revoked
}

/// 全量扫描并清理所有账号的残留索引（启动一次 + 周期兜底）。
///
/// 通过 `SCAN` 收集三类索引键中的账号 id 后逐个清理，可一次性回收历史存量脏数据。
pub async fn sweep_all(redis: &Redis, prefix: &str, keep_max: i64) -> CleanupStat {
    let mut conn = redis.clone();
    let mut ids: HashSet<String> = HashSet::new();

    for suffix in ["login:tokens:", "session:", "refresh:user:"] {
        let pattern = format!("{}{}*", prefix, suffix);
        let strip_prefix = format!("{}{}", prefix, suffix);
        let mut cursor: u64 = 0;
        loop {
            let scanned: Result<(u64, Vec<String>), _> = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(500)
                .query_async(&mut conn)
                .await;
            // SCAN 失败（如存储不支持）时放弃本轮，避免任务空转刷屏
            let Ok((next_cursor, keys)) = scanned else {
                break;
            };
            for key in keys {
                if let Some(id) = key.strip_prefix(&strip_prefix)
                    && !id.is_empty()
                {
                    ids.insert(id.to_string());
                }
            }
            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }
    }

    let mut stat = CleanupStat::default();
    for id in &ids {
        stat.merge(&cleanup_user(redis, prefix, id, keep_max).await);
    }
    stat
}