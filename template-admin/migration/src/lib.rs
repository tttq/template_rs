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

#[tokio::main]
async fn run() {
    let args: Vec<String> = std::env::args().collect();
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:root@localhost:5432/template3".to_string());

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

    let sql = include_str!("m20220101_000001_initial.sql");
    for statement in sql.split(';') {
        let trimmed = statement.trim();
        if trimmed.is_empty() {
            continue;
        }
        match client.execute(trimmed, &[]).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error executing: {}... Error: {}", &trimmed[..trimmed.len().min(80)], e);
                panic!("Migration failed");
            }
        }
    }

    let rows = client.query("SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' AND table_name LIKE 'auth_sys%' ORDER BY table_name", &[]).await.unwrap();
    println!("Tables after migration:");
    for row in &rows {
        let name: &str = row.get(0);
        println!("  {}", name);
    }

    let menu_count: i64 = client.query_one("SELECT COUNT(*) FROM auth_sys_menu", &[]).await.unwrap().get(0);
    let role_menu_count: i64 = client.query_one("SELECT COUNT(*) FROM auth_sys_role_menu", &[]).await.unwrap().get(0);
    println!("Menus: {}, RoleMenus: {}", menu_count, role_menu_count);

    println!("Migration completed successfully!");
}

pub fn main() {
    run();
}
