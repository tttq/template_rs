use summer_web::Router;

pub fn system_routes() -> Router {
    Router::new()
        .nest("/api/auth", super::handlers::auth_routes())
        .nest("/api/system", super::handlers::system_routes())
}
