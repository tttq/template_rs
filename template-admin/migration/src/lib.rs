use sea_orm_migration::prelude::*;

mod m20220101_000001_initial;
mod m20240521_000002_tenant_user;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_initial::Migration),
            Box::new(m20240521_000002_tenant_user::Migration),
        ]
    }
}

/// 按 `;` 切分 SQL 脚本，剔除 `--` 行注释与空块。
///
/// 注意：必须先剥离行注释再判断空块 —— 否则以注释开头的语句块会被整块误判为注释而跳过
/// （这也是历史「菜单权限同步」脚本实际未生效的原因）。
fn split_statements(sql: &str) -> Vec<String> {
    sql.split(';')
        .map(|chunk| {
            chunk
                .lines()
                .filter(|line| !line.trim_start().starts_with("--"))
                .collect::<Vec<_>>()
                .join("\n")
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

/// 逐条执行 SQL 脚本，任一条失败即终止（迁移必须整体生效）。
async fn run_sql_script(client: &tokio_postgres::Client, sql: &str, label: &str) {
    for statement in split_statements(sql) {
        if let Err(e) = client.execute(statement.as_str(), &[]).await {
            let head: String = statement.chars().take(80).collect();
            eprintln!("Error executing [{label}]: {head}... Error: {e}");
            panic!("Migration failed: {label}");
        }
    }
    println!("[{label}] done");
}

#[tokio::main]
async fn run() {
    let args: Vec<String> = std::env::args().collect();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:root@localhost:5432/template2".to_string());

    let (client, connection) = tokio_postgres::connect(&db_url, tokio_postgres::NoTls)
        .await
        .expect("Failed to connect to PostgreSQL");

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    if args.len() > 1 && args[1] == "verify" {
        let rows = client.query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE 'auth_sys%' ORDER BY table_name", &[]).await.unwrap();
        for row in &rows {
            let name: &str = row.get(0);
            println!("Table: {}", name);
        }
        let user_count: i64 = client.query_one("SELECT COUNT(*) FROM auth_sys_user", &[]).await.unwrap().get(0);
        let role_count: i64 = client.query_one("SELECT COUNT(*) FROM auth_sys_role", &[]).await.unwrap().get(0);
        let menu_count: i64 = client.query_one("SELECT COUNT(*) FROM auth_sys_menu", &[]).await.unwrap().get(0);
        println!("Users: {}, Roles: {}, Menus: {}", user_count, role_count, menu_count);
        return;
    }

    // 基础表结构与初始数据
    run_sql_script(
        &client,
        include_str!("m20220101_000001_initial.sql"),
        "initial",
    )
    .await;

    // 增量迁移：同步菜单权限数据，与后端 handler 的 #[sa_check_permission] 宏对齐
    // 使用 ON CONFLICT 实现幂等 upsert，已存在的记录只更新业务字段，可重复执行
    run_sql_script(
        &client,
        include_str!("m20260726_000003_sync_menu_permissions.sql"),
        "sync_menu_permissions",
    )
    .await;

    // 增量迁移：系统管理补齐（客户端 / 附件中心 / 通知中心 / 通知模板 / 导出中心）
    run_sql_script(
        &client,
        include_str!("m20260929_000004_system_modules.sql"),
        "system_modules",
    )
    .await;

    let rows = client.query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE 'auth_sys%' ORDER BY table_name", &[]).await.unwrap();
    println!("Tables after migration:");
    for row in &rows {
        let name: &str = row.get(0);
        println!("  {}", name);
    }

    let menu_count: i64 = client.query_one("SELECT COUNT(*) FROM auth_sys_menu", &[]).await.unwrap().get(0);
    let role_menu_count: i64 = client.query_one("SELECT COUNT(*) FROM auth_sys_role_menu", &[]).await.unwrap().get(0);
    println!("Menus: {}, RoleMenus: {}", menu_count, role_menu_count);

    // 将初始 admin 用户的明文密码替换为 bcrypt 哈希
    let admin_hash = bcrypt::hash("admin123", bcrypt::DEFAULT_COST)
        .expect("Failed to hash admin password");
    client.execute(
        "UPDATE auth_sys_user SET pass_word = $1 WHERE user_name = 'admin'",
        &[&admin_hash],
    ).await.expect("Failed to update admin password hash");
    println!("Admin password hashed successfully (default: admin123)");

    println!("Migration completed successfully!");
}

pub fn main() {
    run();
}