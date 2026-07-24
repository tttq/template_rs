// 宏展开后引用 `summer_sea_orm_ext` 路径，但项目通过 package rename 用 `sea_orm_ext` 导入。
// 此处建立 crate 别名，让宏生成的完全限定路径可被正确解析。
extern crate sea_orm_ext as summer_sea_orm_ext;

pub mod user;
pub mod role;
pub mod menu;
pub mod dept;
pub mod tenant;
pub mod tenant_user;
pub mod dict_type;
pub mod dict_item;
pub mod config;
pub mod auth;
pub mod monitor;
pub mod generator;

pub mod dto {
    pub use super::user::dto::*;
    pub use super::role::dto::*;
    pub use super::menu::dto::*;
    pub use super::dept::dto::*;
    pub use super::tenant::dto::*;
    pub use super::dict_type::dto::*;
    pub use super::dict_item::dto::*;
    pub use super::config::dto::*;
    pub use super::auth::dto::*;
    pub use super::monitor::dto::*;
    pub use super::generator::dto::*;
}

pub mod service {
    pub use super::user::service::*;
    pub use super::role::service::*;
    pub use super::menu::service::*;
    pub use super::dept::service::*;
    pub use super::tenant::service::*;
    pub use super::tenant_user::service::*;
    pub use super::dict_type::service::*;
    pub use super::dict_item::service::*;
    pub use super::config::service::*;
    pub use super::auth::service::*;
    pub use super::monitor::service::*;
    pub use super::generator::service::*;
}