use std::sync::atomic::AtomicI64;

use summer::{auto_config, App};
use summer_web::{WebPlugin, WebConfigurator};
use summer_sea_orm::SeaOrmPlugin;
use summer_redis::RedisPlugin;
use summer_sa_token::{SaTokenPlugin, SaTokenAuthConfigurator};
use common::{CommonPlugin, TenantPlugin};
use sea_orm_ext::{IdGenerator, set_id_generator};
use sea_query::Value;
use system_interface::routes;

mod config;

struct SnowflakeIdGenerator {
    worker_id: i64,
    sequence: AtomicI64,
    epoch: i64,
}

impl SnowflakeIdGenerator {
    fn new(worker_id: i64) -> Self {
        let epoch = 1700000000000i64;
        Self {
            worker_id: worker_id & 0x1F,
            sequence: AtomicI64::new(0),
            epoch,
        }
    }
}

impl IdGenerator for SnowflakeIdGenerator {
    fn generate(&self) -> Value {
        let now = chrono::Utc::now().timestamp_millis();
        let timestamp = now - self.epoch;
        let seq = self
            .sequence
            .fetch_update(
                std::sync::atomic::Ordering::SeqCst,
                std::sync::atomic::Ordering::SeqCst,
                |s| Some((s + 1) & 0xFFF),
            )
            .unwrap_or(0);
        let id = (timestamp << 22) | (self.worker_id << 17) | seq;
        Value::BigInt(Some(id))
    }
}

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    set_id_generator(Box::new(SnowflakeIdGenerator::new(1)));

    App::new()
        .add_plugin(SeaOrmPlugin)
        .add_plugin(RedisPlugin)
        .add_plugin(SaTokenPlugin)
        .add_plugin(CommonPlugin)
        .add_plugin(TenantPlugin)
        .add_plugin(WebPlugin)
        .sa_token_configure(config::SaTokenConfig)
        .add_router(routes::system_routes())
        .run()
        .await
}