//! 权限变更后的实时同步：
//! 1. 覆盖 sa-token 在 Redis 中的角色/权限快照（接口鉴权下个请求即生效）；
//! 2. 通过 Redis Pub/Sub 广播变更事件，由各实例的 SSE 监听器推送给
//!    受影响的在线用户浏览器，前端收到后重拉菜单并重建动态路由。
//!
//! 调用时机：角色分配菜单 / 角色增删改 / 用户分配角色等写库成功之后。
//!
//! 客户端维度：登录会话按 `login_id = 用户ID#客户端编码` 隔离（见 `common::user`），
//! 同一账号在不同客户端可以有不同的角色/权限快照，因此刷新时要逐个客户端会话处理。

use std::collections::HashSet;

use common::error::AppError;
use sa_token_core::StpUtil;
use sea_orm::{ColumnTrait, QueryFilter};
use sea_orm_ext::{DbConn, TenantIgnoreGuard};
use summer_redis::Redis;
use system_entity::{client, menu, role, role_menu, user, user_role};

/// 单条 SQL 的 IN 参数批大小：角色/菜单数量大时按批查询合并，
/// 避免 `IN (...)` 列表过长（PG 参数上限 / SQL 过大导致的性能问题）。
const IN_VALUE_BATCH: usize = 500;

/// 权限变更广播频道（订阅端见 system-interface 的 notify_handler）
pub const PERM_CHANGED_CHANNEL: &str = "sys:perm:changed";

/// 用户在某个客户端下的授权快照
pub struct UserGrant {
    /// 角色编码（用于 sa-token 的角色校验）
    pub role_codes: Vec<String>,
    /// 功能权限码（菜单/按钮的 permission，用于 sa-token 的权限校验）
    pub permissions: Vec<String>,
    /// 命中的菜单行（登录只取权限；用户信息接口据此构建菜单树）
    pub menus: Vec<menu::Model>,
}

/// 按角色批量加载 role_menu（分批 IN 查询后合并）。
///
/// 用户在多个客户端/多角色下 `role_menu` 行数可能很大，一次性
/// `RoleId.is_in(全部角色)` 会让单条 SQL 的参数列表过大；分批控制在
/// [`IN_VALUE_BATCH`] 以内，避免 PG 参数上限与 IN 过多导致的性能问题。
async fn fetch_role_menus(
    db: &DbConn,
    role_ids: &[String],
) -> Result<Vec<role_menu::Model>, AppError> {
    let mut all = Vec::new();
    for chunk in role_ids.chunks(IN_VALUE_BATCH) {
        let part = role_menu::Entity::find()
            .filter(role_menu::Column::RoleId.is_in(chunk.to_vec()))
            .all(db)
            .await?;
        all.extend(part);
    }
    Ok(all)
}

/// 加载用户在指定客户端下的授权。
///
/// - `client_id = None`：不按客户端收敛（该账号全部角色绑定的全部菜单）
/// - `client_id = Some(id)`：角色不按客户端收敛，但菜单只取该客户端下的；超管直接放行该客户端全部菜单，
///   避免新客户端还没分配角色时超管被锁在门外。（角色可跨客户端挂菜单，各端由菜单收敛）
pub async fn load_user_grant(
    db: &DbConn,
    user_id: &str,
    client_id: Option<&str>,
) -> Result<UserGrant, AppError> {
    // 校验用户存在（命中后丢弃模型：菜单收敛已不依赖 admin_flag）
    user::Entity::find()
        .filter(user::Column::Id.eq(user_id))
        .one(db)
        .await?
        .ok_or_else(|| AppError::NotFound("@user_not_found".to_string()))?;

    let user_roles = user_role::Entity::find()
        .filter(user_role::Column::UserId.eq(user_id))
        .all(db)
        .await?;

    let role_ids: Vec<String> = user_roles.iter().map(|r| r.role_id.clone()).collect();

    // 一个角色可跨客户端挂菜单：角色不再按会话客户端收敛，
    // 而是由下方按 menu.client_id 收敛出该端可见的菜单，实现「单角色多端」。
    let roles: Vec<role::Model> = if role_ids.is_empty() {
        Vec::new()
    } else {
        role::Entity::find()
            .filter(role::Column::Id.is_in(role_ids))
            .all(db)
            .await?
    };

    let role_codes: Vec<String> = roles.iter().map(|r| r.role_code.clone()).collect();
    let effective_role_ids: Vec<String> = roles.iter().map(|r| r.id.clone()).collect();

    let menus: Vec<menu::Model> = if effective_role_ids.is_empty() || !client_id.is_some() {
        Vec::new()
    } else {
        let role_menus = fetch_role_menus(db, &effective_role_ids).await?;
        // 同一菜单可能被多个角色绑定：先按 menu_id 去重，避免下游菜单查询的 IN 重复/膨胀
        let menu_ids: Vec<String> = {
            let mut seen = HashSet::with_capacity(role_menus.len());
            role_menus
                .iter()
                .filter_map(|rm| {
                    if seen.insert(rm.menu_id.clone()) {
                        Some(rm.menu_id.clone())
                    } else {
                        None
                    }
                })
                .collect()
        };
        if menu_ids.is_empty() {
            Vec::new()
        } else {
            // 菜单 ID 同样可能很多：分批 IN 查询后合并。
            // 注意分批会破坏跨批的全局顺序（原单次查询的 ORDER BY 只保证批内），
            // 最后统一按 sort_order 排序恢复原有顺序。
            let mut menus: Vec<menu::Model> = Vec::new();
            for chunk in menu_ids.chunks(IN_VALUE_BATCH) {
                let mut select = menu::Entity::find().filter(menu::Column::Id.is_in(chunk.to_vec()));
                if let Some(cid) = client_id {
                    select = select.filter(menu::Column::ClientId.eq(cid));
                }
                menus.extend(select.all(db).await?);
            }
            menus.sort_by_key(|m| m.sort_order);
            menus
        }
    };

    let permissions: Vec<String> = menus
        .iter()
        .filter(|m| m.status == 1)
        .filter_map(|m| m.permission.clone())
        .collect();

    Ok(UserGrant {
        role_codes,
        permissions,
        menus,
    })
}

/// 重算某个登录会话的最新角色/权限并覆盖 Redis 快照（仅在线会话；离线会话下次登录时自然重建）
async fn refresh_login_snapshot(db: &DbConn, login_id: &str, client_id: Option<&str>) -> Result<(), AppError> {
    if !StpUtil::is_login_by_login_id(login_id).await {
        return Ok(());
    }

    let user_id = common::user::base_user_id_from_login_id(login_id);
    let grant = load_user_grant(db, &user_id, client_id).await?;

    // 与登录时一致：整体覆盖快照。key 永久有效，覆盖即生效。
    StpUtil::set_roles(login_id, grant.role_codes).await.ok();
    StpUtil::set_permissions(login_id, grant.permissions).await.ok();

    Ok(())
}

/// 刷新某账号的全部在线会话：历史会话（纯用户ID）+ 各客户端会话（用户ID#客户端编码）
async fn refresh_user_snapshots(db: &DbConn, user_id: &str) {
    if StpUtil::is_login_by_login_id(user_id).await {
        if let Err(e) = refresh_login_snapshot(db, user_id, None).await {
            log::warn!("refresh permission snapshot failed for user {}: {}", user_id, e);
        }
    }

    // 客户端注册表在全局主库：database 模式下 self.db 会路由到租户库，本次查询需忽略租户
    // （作用域仅限该查询，后续菜单/角色快照仍走租户库）
    let clients: Vec<client::Model> = {
        let _main_db_guard = TenantIgnoreGuard::new();
        match client::Entity::find().all(db).await {
            Ok(rows) => rows,
            Err(e) => {
                log::warn!("query clients for permission sync failed: {}", e);
                return;
            }
        }
    };
    for c in clients {
        let login_id = common::user::build_scoped_login_id(user_id, Some(&c.client_code));
        if !StpUtil::is_login_by_login_id(login_id.as_str()).await {
            continue;
        }
        if let Err(e) = refresh_login_snapshot(db, &login_id, Some(&c.id)).await {
            log::warn!("refresh permission snapshot failed for session {}: {}", login_id, e);
        }
    }
}

/// 广播权限变更事件（多实例部署下各实例各自推送给本地 SSE 连接）
async fn publish_changed(redis: &Redis, user_ids: &[String]) {
    let payload = serde_json::json!({ "userIds": user_ids }).to_string();
    if let Err(e) = summer_redis::redis::cmd("PUBLISH")
        .arg(PERM_CHANGED_CHANNEL)
        .arg(&payload)
        .query_async::<i64>(&mut redis.clone())
        .await
    {
        log::warn!("publish perm-changed failed: {}", e);
    }
}

/// 刷新受影响用户的接口权限快照并广播前端刷新。
/// 同步失败仅记日志，不影响已成功的数据库变更主流程。
pub async fn sync_and_notify(db: &DbConn, redis: &Redis, user_ids: &[String]) {
    if user_ids.is_empty() {
        return;
    }
    for uid in user_ids {
        refresh_user_snapshots(db, uid).await;
    }
    publish_changed(redis, user_ids).await;
}
