//! 权限变更实时推送（SSE）
//!
//! 浏览器通过 EventSource 订阅 `GET /api/auth/notify/stream`。EventSource 无法
//! 自定义请求头，token 以查询参数传递（sa-token 按顺序从 header → cookie →
//! `[token_name]` 查询参数提取）。
//!
//! 数据流：application 层权限变更成功后 PUBLISH 到 Redis 频道 `sys:perm:changed`，
//! 本模块监听任务收到后按 user_id 匹配本实例内的连接并下发 `perm-changed` 事件，
//! 前端收到后重拉 `/auth/user-info` 并重建动态路由。
//! 多实例部署时各实例只持有并推送自己的连接；单实例下 PUBLISH 自收亦走此链路。

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

use futures_util::stream::{poll_fn, Stream};
use futures_util::StreamExt;
use sa_token_core::StpUtil;
use summer_redis::redis;
use summer_sa_token::sa_check_login;
use summer_web::axum::response::sse::{Event, Sse};
use summer_web::error::WebError;
use summer_web::get;
use summer_web::nest;
use tokio::sync::mpsc;

type SseItem = Result<Event, Infallible>;

struct PermConnection {
    user_id: String,
    sender: mpsc::Sender<SseItem>,
}

/// 本实例存活的 SSE 连接（conn_id → 连接）
static CONNECTIONS: OnceLock<Mutex<HashMap<u64, PermConnection>>> = OnceLock::new();
static CONN_SEQ: AtomicU64 = AtomicU64::new(1);

fn connections() -> &'static Mutex<HashMap<u64, PermConnection>> {
    CONNECTIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// 加锁（锁中毒恢复）：注册表内容无不变量约束，持锁任务 panic 后
/// 直接接管内部数据，避免 SSE 通知链路因单次 panic 永久失效
fn lock_connections() -> std::sync::MutexGuard<'static, HashMap<u64, PermConnection>> {
    connections()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[nest("/auth")]
mod controller {
    use super::*;

/// 权限变更通知流（登录即可订阅，无需菜单权限）
#[get("/notify/stream")]
#[sa_check_login]
async fn perm_stream() -> Result<Sse<impl Stream<Item = SseItem>>, WebError> {
    let login_id = StpUtil::get_login_id_as_string()
        .await
        .map_err(|_| WebError::from(summer_web::error::KnownWebError::unauthorized("未登录")))?
        .to_string();
    // 客户端会话的 login_id 形如 `用户ID#客户端编码`，SSE 连接按真实用户ID匹配广播
    let user_id = common::user::base_user_id_from_login_id(&login_id);

    let (tx, mut rx) = mpsc::channel::<SseItem>(16);

    let conn_id = CONN_SEQ.fetch_add(1, Ordering::Relaxed);
    lock_connections().insert(
        conn_id,
        PermConnection {
            user_id,
            sender: tx.clone(),
        },
    );

    // 连接建立即确认，前端据此认为链路可用
    let _ = tx
        .send(Ok(Event::default().event("connected").data("ok")))
        .await;

    // 心跳：保活 + 感知断开。客户端断开后响应体（rx 端）被 drop，
    // try_send 返回 Closed → 从注册表移除并结束任务。
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(20));
        loop {
            interval.tick().await;
            match tx.try_send(Ok(Event::default().comment("ping"))) {
                Ok(()) | Err(mpsc::error::TrySendError::Full(_)) => {}
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    lock_connections().remove(&conn_id);
                    break;
                }
            }
        }
    });

    Ok(Sse::new(poll_fn(move |cx| rx.poll_recv(cx))))
}
}

/// 推送给本实例内受影响的连接（由 PubSub 监听任务调用）
async fn push_to_local(user_ids: &[String]) {
    let targets: Vec<mpsc::Sender<SseItem>> = {
        let map = lock_connections();
        map.values()
            .filter(|c| user_ids.contains(&c.user_id))
            .map(|c| c.sender.clone())
            .collect()
    };
    if targets.is_empty() {
        return;
    }
    let event = Event::default()
        .event("perm-changed")
        .data(serde_json::json!({ "userIds": user_ids }).to_string());
    for sender in targets {
        // 发送失败说明对端已断开，由心跳任务负责清理注册表
        let _ = sender.send(Ok(event.clone())).await;
    }
}

/// 启动权限变更广播订阅（main.rs 启动时调用一次）。
///
/// 使用独立 PubSub 连接（业务连接为 ConnectionManager，不支持订阅态复用）；
/// 断线自动重连，失败不影响主流程。
pub fn start_perm_notify_listener() {
    tokio::spawn(async move {
        use summer::config::ConfigRegistry;
        // 等待全局容器就绪后读取 [redis] 配置
        let uri = loop {
            match summer::App::global().get_config::<summer_redis::config::RedisConfig>() {
                Ok(c) => break c.uri,
                Err(_) => tokio::time::sleep(Duration::from_secs(1)).await,
            }
        };
        loop {
            if let Err(e) = listen_perm_changed(&uri).await {
                log::warn!("perm notify listener error: {}, reconnecting", e);
            }
            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    });
}

async fn listen_perm_changed(uri: &str) -> Result<(), redis::RedisError> {
    use system_application::auth::permission_sync::PERM_CHANGED_CHANNEL;

    let client = redis::Client::open(uri)?;
    let mut pubsub = client.get_async_pubsub().await?;
    pubsub.subscribe(PERM_CHANGED_CHANNEL).await?;
    log::info!("perm notify listener subscribed");

    let mut stream = pubsub.on_message();
    while let Some(msg) = stream.next().await {
        let payload: String = msg.get_payload()?;
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&payload) else {
            continue;
        };
        let user_ids: Vec<String> = v
            .get("userIds")
            .and_then(|x| x.as_array())
            .map(|arr| arr.iter().filter_map(|x| x.as_str().map(String::from)).collect())
            .unwrap_or_default();
        if !user_ids.is_empty() {
            push_to_local(&user_ids).await;
        }
    }
    Ok(())
}
