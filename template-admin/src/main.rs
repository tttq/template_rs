//! 应用入口：插件装配、路由挂载与后台任务编排。
//!
//! 【路由注册】本项目通过 `#[get]`/`#[post]` 等路由宏 + `#[nest]` 声明完整路径，
//! 由 `summer_web::handler::auto_router()` 在启动期经 inventory 跨 crate 收集并挂载，
//! 不再手工 `typed_route` + `nest` 聚合。`/api` 统一由 `config/app.toml` 的
//! `[web] global_prefix = "api"` 提供。

mod config;

use std::sync::Arc;
use std::time::Duration;

use common::{AuditFieldFillHandler, SaTokenTenantIdProvider};
use sea_orm_ext::plugin::{
    DynamicTenantConfigProviderComponent, DynamicTenantPlugin, FieldFillHandlerComponent,
    SeaOrmPlugin as SeaOrmExtPlugin, SnowflakeIdGenerator, TenantIdProviderComponent, TenantPlugin,
};
use sea_orm_ext::set_id_generator;
use summer::plugin::{ComponentRegistry, MutableComponentRegistry};
use summer::App;
use summer::app::AppBuilder;
use summer_mail::MailPlugin;
use summer_redis::RedisPlugin;
use summer_sa_token::{SaTokenAuthConfigurator, SaTokenPlugin};
use summer_web::handler::auto_router;
use summer_web::{LayerConfigurator, WebConfigurator, WebPlugin};
use system_application::auth::service::AuthAppService;
use system_application::service::NotificationService;
use system_infrastructure::{StorageService, TenantConfigProvider};
use system_interface::routes::{with_error_handler, with_i18n};

#[tokio::main]
async fn main() {
    // JWT（HS256）签名依赖运行时 CryptoProvider，必须先于 sa-token 初始化安装
    jsonwebtoken::crypto::CryptoProvider::install_default(
        &jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER,
    )
    .expect("Failed to install CryptoProvider");

    set_id_generator(Box::new(SnowflakeIdGenerator::new(1)));

    // 后台任务在 run() 前启动：内部自行等待容器就绪（见各任务的文档注释），
    // 与主流程并行、互不阻塞
    tokio::spawn(log_storage_driver());
    tokio::spawn(run_session_index_cleanup());
    tokio::spawn(run_startup_tasks());

    build_app().run().await;
}

/// 装配应用：插件注册 → 全局组件 → sa-token 配置 → 路由挂载 → 全局中间件
///
/// 顺序约束：
/// - `DynamicTenantPlugin` 必须在 `TenantPlugin` 之后注册：前者启动时从主库
///   `auth_sys_tenant` 加载所有 database 模式租户并建立连接，依赖后者提供的租户基座；
///   运行时由 TenantManager 增删改租户并同步缓存
/// - `with_error_handler` / `with_i18n` 最后注册，确保包裹全部已挂载路由
fn build_app() -> AppBuilder {
    let mut app = App::new();

    // ---------- 插件 ----------
    app.add_plugin(RedisPlugin)
        .add_plugin(MailPlugin)
        .add_plugin(SaTokenPlugin)
        .add_plugin(SeaOrmExtPlugin)
        .add_plugin(TenantPlugin)
        .add_plugin(DynamicTenantPlugin::new())
        .add_plugin(WebPlugin);

    // ---------- 全局组件 ----------
    app.add_component(FieldFillHandlerComponent::new(Arc::new(
        AuditFieldFillHandler::new("system".to_string()),
    )))
    .add_component(TenantIdProviderComponent::new(Arc::new(
        SaTokenTenantIdProvider,
    )))
    .add_component(DynamicTenantConfigProviderComponent::new(Arc::new(
        TenantConfigProvider::new(),
    )));

    app.sa_token_configure(config::SaTokenConfig);

    // ---------- 路由（自动扫描注册）----------
    // auto_router() 通过 inventory 跨 crate 收集全部路由宏 handler（含 #[nest] 前缀后的完整路径）
    app.add_router(auto_router());

    // ---------- 全局中间件（最后注册以包裹全部路由）----------
    // 将 sa-token 权限校验失败（401/403）的英文响应重写为统一 ApiResponse JSON
    app.add_router_layer(with_error_handler);
    // 业务消息翻译：按请求 Accept-Language 翻译 @key 消息；注册在 error_handler 外层
    app.add_router_layer(with_i18n);

    app
}

/// 启动期依赖组件就绪的初始化任务：
/// - 注册全局通知发送器（`common::notify::NotifySender`），使各模块可发送站内通知
/// - 启动权限变更 SSE 推送订阅端：权限变更时通知在线用户浏览器刷新菜单
async fn run_startup_tasks() {
    let notify_svc: NotificationService = wait_component_ready().await;
    system_application::notification::service::install_as_global(Arc::new(notify_svc));
    system_interface::handlers::notify_handler::start_perm_notify_listener();
}

/// 附件存储自检：启动后打印实际生效的存储驱动与目标。
///
/// 背景：`attachment.storage` 是嵌套表，若配置前缀写错会静默退化为默认值
/// （driver=local，文件写进 ./uploads/attachment 而不是 RustFS），且上传一切正常、
/// 毫无报错——只能靠这行启动日志发现。driver 取值非法时 StorageService 初始化即 panic。
async fn log_storage_driver() {
    let storage: StorageService = wait_component_ready().await;
    log::info!("附件存储驱动 = {}", storage.describe());
    if storage.is_local() {
        log::warn!(
            "附件存储为本地磁盘（driver=local）：生产/多实例部署请改用 rustfs，否则容器重建后附件会丢失"
        );
    }
}

/// sa-token 会话残留索引清理：启动即执行一次（回收历史存量脏数据），此后每小时兜底。
///
/// 背景：sa-token 的 `login:tokens` / `session` / `refresh:user` 三类索引键写入时不带 TTL，
/// 只在显式登出对应 token 时才缩减 —— token 自然过期、关闭浏览器、并发登录都不会触发清理，
/// 导致 Redis 随登录次数无界膨胀（登录链路已即时收敛，本任务负责兜底与历史数据回收）。
async fn run_session_index_cleanup() {
    /// sa-token 会话残留索引清理周期（登录时已即时收敛，此处为兜底 + 清理存量脏数据）
    const SESSION_CLEANUP_INTERVAL: Duration = Duration::from_secs(3600);

    let auth: AuthAppService = wait_component_ready().await;

    loop {
        let stat = auth.sweep_session_index().await;
        if stat.is_empty() {
            log::info!("sa-token 会话索引巡检完成：无残留（账号数 {}）", stat.users);
        } else {
            log::info!(
                "sa-token 会话索引巡检完成：账号数 {}，移除失效条目 {}，删除空键 {}",
                stat.users,
                stat.removed_entries,
                stat.deleted_keys
            );
        }
        tokio::time::sleep(SESSION_CLEANUP_INTERVAL).await;
    }
}

/// 轮询等待全局容器中指定组件就绪并返回其克隆。
///
/// 容器在 `build_app().run()` 阶段才完成装配，因此后台任务不能在启动时直接取组件；
/// 若对应插件初始化失败，此循环将永久空转——根因由插件自身的错误日志暴露。
async fn wait_component_ready<C>() -> C
where
    C: Clone + Send + Sync + 'static,
{
    /// 后台任务等待组件就绪的轮询间隔
    const COMPONENT_POLL_INTERVAL: Duration = Duration::from_millis(200);

    loop {
        if let Ok(component) = App::global().try_get_component::<C>() {
            return component;
        }
        tokio::time::sleep(COMPONENT_POLL_INTERVAL).await;
    }
}