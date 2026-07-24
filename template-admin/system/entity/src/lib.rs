pub mod user;
pub mod role;
pub mod menu;
pub mod dept;
pub mod tenant;
pub mod tenant_user;
pub mod dict_type;
pub mod dict_item;
pub mod config;
pub mod role_menu;
pub mod user_role;
pub mod rbac_resource;
pub mod rbac_permission;
pub mod rbac_role;
pub mod rbac_role_hierarchy;
pub mod rbac_role_permission;
pub mod rbac_user_role;
pub mod rbac_user_override;

// 宏展开后引用 `summer_sea_orm_ext` 路径，但项目通过 package rename 用 `sea_orm_ext` 导入。
// 此处建立 crate 别名，让宏生成的完全限定路径可被正确解析。
extern crate sea_orm_ext as summer_sea_orm_ext;

pub use sea_orm;