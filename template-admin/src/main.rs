use std::sync::Arc;
use sea_orm_ext::plugin::{FieldFillHandlerComponent, SeaOrmExtPlugin, SnowflakeIdGenerator, TenantIdProviderComponent, TenantPlugin};
use summer::{auto_config, App};
use summer::plugin::MutableComponentRegistry;
use summer_web::{WebPlugin, WebConfigurator};
use summer_sea_orm::SeaOrmPlugin;
use summer_redis::RedisPlugin;
use summer_sa_token::{SaTokenPlugin, SaTokenAuthConfigurator};
use common::{AuditFieldFillHandler, SaTokenTenantIdProvider};
use sea_orm_ext::set_id_generator;
use system_interface::routes;

mod config;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    jsonwebtoken::crypto::CryptoProvider::install_default(&jsonwebtoken::crypto::rust_crypto::DEFAULT_PROVIDER)
        .expect("Failed to install CryptoProvider");

    set_id_generator(Box::new(SnowflakeIdGenerator::new(1)));

    App::new()
        .add_plugin(SeaOrmPlugin)
        .add_plugin(RedisPlugin)
        .add_plugin(SaTokenPlugin)
        .add_plugin(SeaOrmExtPlugin)
        .add_plugin(TenantPlugin)
        .add_plugin(WebPlugin)
        .add_component(FieldFillHandlerComponent::new(Arc::new(AuditFieldFillHandler::new("system".to_string()))))
        .add_component(TenantIdProviderComponent::new(Arc::new(SaTokenTenantIdProvider)))
        .sa_token_configure(config::SaTokenConfig)
        .add_router(routes::system_routes())
        .run()
        .await
}
