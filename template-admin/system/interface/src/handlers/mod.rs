pub mod auth_handler;
pub mod user_handler;
pub mod role_handler;
pub mod menu_handler;
pub mod dept_handler;
pub mod tenant_handler;
pub mod dict_handler;
pub mod config_handler;
pub mod file_handler;

use summer_web::Router;

pub fn auth_routes() -> Router {
    auth_handler::routes()
}

pub fn system_routes() -> Router {
    Router::new()
        .merge(user_handler::routes())
        .merge(role_handler::routes())
        .merge(menu_handler::routes())
        .merge(dept_handler::routes())
        .merge(tenant_handler::routes())
        .merge(dict_handler::routes())
        .merge(config_handler::routes())
        .merge(file_handler::routes())
}
