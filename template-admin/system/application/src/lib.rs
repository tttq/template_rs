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