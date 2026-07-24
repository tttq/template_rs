use std::time::Instant;

use summer_web::get;
use summer_web::extractor::Component;
use summer_web::axum::response::IntoResponse;
use summer_redis::Redis;
use summer_sa_token::sa_ignore;
use sea_orm_ext::{check_all_tenant_databases, HealthReport};

/// 健康检查端点
///
/// 检查项：
/// - 主数据库（含 fallback 链）+ 所有租户专属数据库（通过 `check_all_tenant_databases`）
/// - Redis 连通性（PING）
///
/// 返回 HTTP 200（全部健康）或 HTTP 503（存在不健康项），响应体为 JSON 报告。
///
/// 注意：本 handler 通过 `#[get("/health")]` 宏自动注册到 `auto_router()`（inventory），
/// 无需在 `system_routes()` 中手动 `typed_route`，否则会导致 `GET /health` 路由重复注册 panic。
#[get("/health")]
#[sa_ignore]
async fn health_check(Component(redis): Component<Redis>) -> impl IntoResponse {
    let start = Instant::now();

    // 1. 数据库健康检查（含主库 fallback 链 + 所有租户库）
    let db_report: HealthReport = check_all_tenant_databases().await;

    // 2. Redis 健康检查
    let redis_healthy = summer_redis::redis::cmd("PING")
        .query_async::<String>(&mut redis.clone())
        .await
        .map(|resp| resp == "PONG")
        .unwrap_or(false);

    let all_healthy = db_report.all_healthy() && redis_healthy;

    let entries: Vec<serde_json::Value> = db_report
        .entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "label": e.label,
                "healthy": e.healthy,
                "error": e.error,
                "elapsedMs": e.elapsed.as_millis() as u64,
            })
        })
        .collect();

    let body = serde_json::json!({
        "status": if all_healthy { "UP" } else { "DEGRADED" },
        "service": "template-admin",
        "version": env!("CARGO_PKG_VERSION"),
        "elapsedMs": start.elapsed().as_millis() as u64,
        "components": {
            "database": {
                "status": if db_report.all_healthy() { "UP" } else { "DOWN" },
                "summary": db_report.summary(),
                "entries": entries,
            },
            "redis": {
                "status": if redis_healthy { "UP" } else { "DOWN" },
            },
        },
    });

    let status = if all_healthy {
        summer_web::axum::http::StatusCode::OK
    } else {
        summer_web::axum::http::StatusCode::SERVICE_UNAVAILABLE
    };

    (status, summer_web::axum::Json(body))
}
