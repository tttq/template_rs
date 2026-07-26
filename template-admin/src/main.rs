use std::sync::Arc;
use sea_orm_ext::plugin::{
    DynamicTenantConfigProviderComponent, DynamicTenantPlugin,
    FieldFillHandlerComponent, SeaOrmPlugin as SeaOrmExtPlugin, SnowflakeIdGenerator,
    TenantIdProviderComponent, TenantPlugin,
};
use summer::{auto_config, App};
use summer::plugin::MutableComponentRegistry;
use summer_web::{WebPlugin, WebConfigurator, LayerConfigurator};
use summer_redis::RedisPlugin;
use summer_sa_token::{SaTokenPlugin, SaTokenAuthConfigurator};
use common::{AuditFieldFillHandler, SaTokenTenantIdProvider};
use sea_orm_ext::set_id_generator;
use system_infrastructure::TenantConfigProvider;
use system_interface::routes;

mod config;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    jsonwebtoken::crypto::CryptoProvider::install_default(&jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER)
        .expect("Failed to install CryptoProvider");

    set_id_generator(Box::new(SnowflakeIdGenerator::new(1)));

    App::new()
        .add_plugin(RedisPlugin)
        .add_plugin(SaTokenPlugin)
        .add_plugin(SeaOrmExtPlugin)
        .add_plugin(TenantPlugin)
        // DynamicTenantPlugin 必须在 TenantPlugin 之后注册：
        // 启动时从主库 auth_sys_tenant 加载所有 database 模式租户并建立连接，
        // 运行时由 TenantManager 增删改租户并同步缓存。
        .add_plugin(DynamicTenantPlugin::new())
        .add_plugin(WebPlugin)
        .add_component(FieldFillHandlerComponent::new(Arc::new(AuditFieldFillHandler::new("system".to_string()))))
        .add_component(TenantIdProviderComponent::new(Arc::new(SaTokenTenantIdProvider)))
        .add_component(DynamicTenantConfigProviderComponent::new(Arc::new(TenantConfigProvider::new())))
        .sa_token_configure(config::SaTokenConfig)
        .add_router(routes::system_routes())
        // 注册全局错误响应中间件：
        // 将 summer-sa-token 权限校验失败（401/403）的英文错误响应
        // 替换为统一的 ApiResponse JSON 与中文提示（"无权限访问" / "未登录或登录已过期"）
        .add_router_layer(routes::with_error_handler)
        .run()
        .await
}
