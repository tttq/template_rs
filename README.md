# Template Admin - Rust DDD 后台管理模板

**版本**: v4.2
**最后更新**: 2026-05-19
**技术栈**: Rust + Summer-rs 0.5 + Sea-ORM 2.0 + sea-orm-ext + PostgreSQL 17 + Redis 7 + Sa-Token

---

## 核心能力

| # | 能力 | 说明 |
|---|------|------|
| 1 | 自动填充审计字段 + 乐观锁 | `CommonPlugin` 注册 `AuditFieldFillHandler`，自动填充 create_by/update_by/version 等，业务层零代码 |
| 2 | 主键自动生成 | `IdGenerator` trait 支持雪花算法 / UUID，`#[sea_orm(primary_key, auto_generate)]` 声明即启用 |
| 3 | 逻辑删除 | `DeriveSoftDelete` 宏 + `#[soft_delete(default=0, del=1)]`，DELETE 自动转 UPDATE，`find_active()` 自动过滤 |
| 4 | 多租户（双模式） | `DeriveTenant` 宏，支持表隔离和数据库隔离两种模式，RAII `TenantGuard` 管理上下文 |
| 5 | 批量操作 | 宏自动生成 `insert_many_with_fill` / `update_many_with_fill` / `delete_many_soft` |
| 6 | RBAC2 双层权限 | summer-sa-token（HTTP 级）+ sea-orm RBAC 引擎（数据级） |
| 7 | OAuth2 授权 | 通用 AuthProvider trait，配置添加任意 OAuth2 提供者 |
| 8 | OSS 存储 | 配置开关，支持 MinIO/阿里云/腾讯云/华为云/自定义 |
| 9 | 用户工具 | `common::user` 封装 StpUtil，提供 `get_current_user_id()` / `get_current_user()` 等同步/异步 API |

---

## 项目结构

```
template-admin/                         # Workspace 根
│
│  ── 公共层 ──
├── common/                             # 共享内核：错误/返回/分页 + StpUtil 用户封装 + CommonPlugin
│
│  ── System 服务（基础管理）──
├── system/
│   ├── entity/                         # Sea-ORM 实体（cli 生成 + sea-orm-ext 注解）
│   ├── domain/                         # 领域层：仓储 trait + 领域服务
│   ├── application/                    # 应用层：DTO + 应用服务
│   ├── infrastructure/                 # 基础设施：仓储实现 + 中间件
│   └── interface/                      # 接口层：HTTP handlers
│
│  ── 公共入口 ─
├── config/                             # 配置文件（TOML，支持环境变量插值）
├── locales/                            # i18n 资源（zh-CN / en-US）
├── src/main.rs                         # 入口（插件注册）
├── migration/                          # 数据库迁移（sea-orm-migration）
├── docker-compose.yml                  # PostgreSQL + Redis + MinIO
│
│  ── 前端 ─
└── template-web/                       # Ant Design Vue + TypeScript
```

**依赖关系（无循环）**：
```
system/interface → system/application → system/domain ← system/infrastructure
                                                ↓              ↓
                                            common        system/entity
                                                ↑
                                            common ────────┘
```

**跨服务复用**：新建业务服务（如 wms）时，依赖 `common` 即可获得全部公共能力（错误、返回、分页、用户工具、审计填充）。

---

## Sea-ORM 生态依赖

```toml
[dependencies]
sea-orm = "2.0"
sea-orm-ext = { path = "D:\\Projcets\\Rust\\sea-orm-ext" }
common = { path = "common" }
```

### sea-orm-ext 提供的能力

| 能力 | Derive 宏 | 注解 | 说明 |
|------|-----------|------|------|
| Auto-Fill | `DeriveAutoFill` | `#[sea_orm_ext(insert)]` / `#[sea_orm_ext(update)]` / `#[sea_orm_ext(insert_update)]` | INSERT/UPDATE 时自动调用 `FieldFillHandler` 填充字段 |
| ID 生成 | （内置） | `#[sea_orm(primary_key, auto_generate)]` | INSERT 主键为空时调用 `IdGenerator::generate()` |
| Soft-Delete | `DeriveSoftDelete` | `#[soft_delete(default=0, del=1)]` | DELETE 转 UPDATE，`find_active()` 过滤已删除 |
| Multi-Tenant | `DeriveTenant` | `#[sea_orm_ext(TENANT)]` | 表隔离 / 数据库隔离双模式 |
| 组合宏 | `DeriveAutoFillSoftDelete` / `DeriveAutoFillSoftDeleteTenant` | 组合注解 | 同时启用多种能力 |

---

## 插件注册（main.rs）

```rust
use summer::{auto_config, App};
use summer_web::{WebPlugin, WebConfigurator};
use summer_sea_orm::SeaOrmPlugin;
use summer_redis::RedisPlugin;
use summer_sa_token::{SaTokenPlugin, SaTokenAuthConfigurator};
use common::CommonPlugin;
use sea_orm_ext::TenantPlugin;

#[auto_config(WebConfigurator)]
#[tokio::main]
async fn main() {
    App::new()
        .add_plugin(SeaOrmPlugin)
        .add_plugin(RedisPlugin)
        .add_plugin(SaTokenPlugin)
        .add_plugin(CommonPlugin)       // 审计填充 + 用户工具
        .add_plugin(TenantPlugin)       // 多租户（自动读取 [sea-orm-ext.tenant] 配置）
        .add_plugin(WebPlugin)
        .sa_token_configure(config::SaTokenConfig)
        .run()
        .await
}
```

| 插件 | 来源 | 职责 |
|------|------|------|
| `CommonPlugin` | `common` | 注册 `AuditFieldFillHandler`（自动填充 create_time/create_by/update_time/version 等审计字段） |
| `TenantPlugin` | `sea-orm-ext` | 读取 `[sea-orm-ext.tenant]` 配置，初始化租户隔离 |

---

## JSON 命名约定

**前后端统一使用 camelCase**，所有序列化结构体必须标注 `#[serde(rename_all = "camelCase")]`：

| 后端字段 (snake_case) | JSON 字段 (camelCase) |
|------------------------|----------------------|
| `user_name` | `userName` |
| `create_time` | `createTime` |
| `tenant_id` | `tenantId` |
| `page_size` | `pageSize` |
| `total_pages` | `totalPages` |

`common/` 中的所有 struct（ApiResponse、PageQuery、PageResult、CurrentUser）均已内置此注解。

---

## 自动填充（Auto-Fill）

`sea-orm-ext` 的 `DeriveAutoFill` 宏 + `CommonPlugin`，三步完成：

1. **字段注解**：标记 `#[sea_orm_ext(insert)]` / `#[sea_orm_ext(update)]` / `#[sea_orm_ext(insert_update)]`
2. **Derive 宏**：`DeriveAutoFill` 编译期生成 `ActiveModelBehavior::before_save`
3. **CommonPlugin**：启动时读取 `[common]` 配置，注册 `AuditFieldFillHandler`

### 注解含义

| 注解 | INSERT | UPDATE | 典型用途 |
|------|:---:|:---:|---------|
| `#[sea_orm_ext(insert)]` | ✅ | ❌ | create_time, create_by, tenant_id |
| `#[sea_orm_ext(update)]` | ❌ | ✅ | update_time, update_by |
| `#[sea_orm_ext(insert_update)]` | ✅ | ✅ | version |

### 实体定义示例

```rust
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveAutoFillSoftDeleteTenant;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFillSoftDeleteTenant)]
#[sea_orm(table_name = "auth_sys_user")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: i64,
    pub user_name: String,
    pub pass_word: String,
    #[sea_orm_ext(insert)]
    pub create_time: DateTimeUtc,
    #[sea_orm_ext(insert)]
    pub create_by: Option<String>,
    #[sea_orm_ext(insert)]
    pub create_id: Option<String>,
    #[sea_orm_ext(update)]
    pub update_time: DateTimeUtc,
    #[sea_orm_ext(update)]
    pub update_by: Option<String>,
    #[sea_orm_ext(update)]
    pub update_id: Option<String>,
    #[sea_orm_ext(insert_update)]
    pub version: i32,
    #[soft_delete(default = 0, del = 1)]
    pub delete_flag: i32,
    #[sea_orm_ext(TENANT)]
    pub tenant_id: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}
```

`AuditFieldFillHandler` 通过 `common::user::get_current_user_id()` 获取当前用户（同步），运行时无上下文时回退到 `[common].default_user` 配置。

---

## 主键自动生成（IdGenerator）

`#[sea_orm(primary_key, auto_generate)]` 标记，在 `main.rs` 注册：

```rust
use sea_orm_ext::{IdGenerator, set_id_generator};
use sea_query::Value;

fn main() {
    set_id_generator(Box::new(SnowflakeIdGenerator::new(1)));
}

struct SnowflakeIdGenerator { worker_id: i64, sequence: AtomicI64 }

impl IdGenerator for SnowflakeIdGenerator {
    fn generate(&self) -> Value { /* 雪花算法 */ }
}
```

**行为**：INSERT 时主键 `NotSet` → 自动生成；`Set(val)` → 不覆盖；UPDATE → 不触发。

---

## 逻辑删除（Soft-Delete）

`DeriveSoftDelete` 宏 + `#[soft_delete(default=0, del=1)]`：

- `before_delete` 将 DELETE 转为 UPDATE，返回 `DbErr::Custom(...)` 阻止物理删除
- `Entity::find_active()` 自动过滤 `WHERE delete_flag = 0`
- 原始 `Entity::find()` 不受影响（可查已删除数据）

---

## 多租户（Multi-Tenant）

`DeriveTenant` 宏 + `TenantPlugin`，支持**表隔离**和**数据库隔离**两种模式。

### 表隔离模式

```rust
use sea_orm_ext::{TenantContext, set_tenant_context, clear_tenant_context, TenantGuard};

// 设置租户上下文
set_tenant_context(TenantContext { tenant_id: 1i32.into() });

// INSERT — tenant_id 自动填充
let product = product::ActiveModel { name: Set("P1".to_owned()), ..Default::default() };
product.insert(&db).await?;

// 查询 — 自动添加 WHERE tenant_id = 1
let list = Entity::find_with_tenant().all(&db).await?;

clear_tenant_context();

// RAII guard — 离开作用域自动清除
{
    let _guard = TenantGuard::set(2i32.into());
    Entity::find_with_tenant().all(&db).await?;  // 租户 2
}
```

### 数据库隔离模式

```rust
use sea_orm_ext::{HashMapConnectionStore, ConnectionStore, TenantDatabaseManager, get_tenant_database};

let store: Arc<dyn ConnectionStore> = Arc::new(HashMapConnectionStore::new());
let manager = TenantDatabaseManager::new(store.clone(), Box::new(|tenant_id| {
    sea_orm::Database::connect(&format!("postgres://.../tenant_{}", tenant_id))
}));
manager.initialize_tenants(vec![1i64.into(), 2i64.into()])?;

set_tenant_store(store);
set_tenant_context(TenantContext { tenant_id: 1i64.into() });
if let Some(db) = get_tenant_database()? {
    Entity::find().all(&*db).await?;
}
```

### 配置方式

```toml
[sea-orm-ext.tenant]
enabled = true
mode = "table"                    # "table" | "database"
default_tenant_id = 1
header_name = "x-tenant-id"
ignore_if_missing = true

# 数据库隔离模式
[[sea-orm-ext.tenant.databases]]
tenant_id = 1
[sea-orm-ext.tenant.databases.database]
url = "postgres://user:pass@localhost/tenant_1_db"
max_connections = 20
```

---

## 批量操作

Derive 宏自动生成：

| 方法 | 说明 |
|------|------|
| `Entity::insert_many_with_fill(models, db)` | 批量插入，自动填充 |
| `Entity::update_many_with_fill(models, db)` | 批量更新，自动填充 |
| `Entity::delete_many_soft(models, db)` | 批量软删除 |

---

## DDD 代码组织结构

### 分层职责

```
┌─────────────────────────────────────────────────────┐
│  Interface 层                                        │
│  HTTP handler，调用 Application 层                    │
│  路由宏: #[get("/")] #[post("/")]                     │
│  提取器: Component<DbConn> Json<Path> Query<PageQuery>│
├─────────────────────────────────────────────────────┤
│  Application 层                                      │
│  编排领域服务，定义 DTO，不包含业务规则                  │
│  依赖注入: #[derive(Service)] #[inject(component)]    │
├─────────────────────────────────────────────────────┤
│  Domain 层                                           │
│  定义业务模型和仓储 trait，不依赖任何基础设施            │
│  仓储 trait: UserRepository, RoleRepository           │
├─────────────────────────────────────────────────────┤
│  Infrastructure 层                                   │
│  实现仓储 trait，适配外部服务                          │
│  仓储实现: UserRepositoryImpl                         │
├─────────────────────────────────────────────────────┤
│  Entity 层                                           │
│  Sea-ORM 实体定义，含 sea-orm-ext 注解                 │
│  由 sea-orm-cli 生成 + 手动注解                       │
└─────────────────────────────────────────────────────┘
```

### 各层实现示例

**Entity 层** — `system/entity/src/user.rs`

```rust
use sea_orm::entity::prelude::*;
use sea_orm_ext::DeriveAutoFillSoftDeleteTenant;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFillSoftDeleteTenant)]
#[sea_orm(table_name = "auth_sys_user")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: i64,
    pub user_name: String,
    pub pass_word: String,
    #[sea_orm_ext(insert)]
    pub create_time: DateTimeUtc,
    #[sea_orm_ext(update)]
    pub update_time: DateTimeUtc,
    #[sea_orm_ext(insert_update)]
    pub version: i32,
    #[soft_delete(default = 0, del = 1)]
    pub delete_flag: i32,
    #[sea_orm_ext(TENANT)]
    pub tenant_id: i32,
}
```

**Domain 层** — `system/domain/src/user_repository.rs`

```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<user::Model>>;
    async fn find_by_name(&self, name: &str) -> Result<Option<user::Model>>;
    async fn create(&self, user: user::ActiveModel) -> Result<user::Model>;
    async fn update(&self, user: user::ActiveModel) -> Result<user::Model>;
    async fn delete(&self, id: &str) -> Result<()>;
}
```

**Application 层** — `system/application/src/user_service.rs`

```rust
#[derive(Clone, Service)]
pub struct UserAppService {
    #[inject(component)]
    db: DbConn,
}

impl UserAppService {
    pub async fn create_user(&self, dto: CreateUserDto) -> Result<UserVo> {
        let active_model = dto.into_active_model();
        let model = active_model.insert(&self.db).await?;
        Ok(model.into())
    }
}
```

**Infrastructure 层** — `system/infrastructure/src/persistence/user_repository_impl.rs`

```rust
pub struct UserRepositoryImpl {
    db: DbConn,
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_by_id(&self, id: &str) -> Result<Option<user::Model>> {
        Ok(user::Entity::find_by_id(id).one(&self.db).await?)
    }
}
```

**Interface 层** — `system/interface/src/handlers/user_handler.rs`

```rust
#[get("/users")]
#[sa_check_permission("user:list")]
async fn list_users(
    Component(db): Component<DbConn>,
    Query(query): Query<PageQuery>,
) -> Result<Json<ApiResponse<PageResult<UserVo>>>> {
    let result = UserAppService::new(db).list(query).await?;
    Ok(Json(ApiResponse::success(result)))
}
```

---

## 公共核心 common 模块

### 模块结构

```
common/src/
├── lib.rs          # 模块导出
├── error.rs        # AppError 统一错误类型
├── response.rs     # ApiResponse<T> 统一返回
├── pagination.rs   # PageQuery / PageResult 分页 DTO
├── user.rs         # StpUtil 封装（同步 + 异步获取当前用户）
└── plugin.rs       # CommonPlugin：注册 AuditFieldFillHandler
```

### 用户工具（user.rs）

对 `StpUtil` 的封装，替代 `LoginIdExtractor`：

```rust
use common::{get_current_user_id, get_current_user_name, get_current_user};

// 同步 — 适合 FieldFillHandler 等非异步上下文
let uid = get_current_user_id();       // Option<String>
let name = get_current_user_name();    // Option<String>
let user = get_current_user();         // Option<CurrentUser { id, name }>

// 异步 — 适合 Handler / Service
let uid = common::user::get_current_user_id_async().await;
let user = common::user::get_current_user_async().await;
```

| 函数 | 返回 | 调用方式 |
|------|------|---------|
| `get_current_user_id()` | `Option<String>` | 同步（内部 block_on） |
| `get_current_user_name()` | `Option<String>` | 同步 |
| `get_current_user()` | `Option<CurrentUser>` | 同步 |
| `get_current_user_id_async()` | `async Option<String>` | 异步 |
| `get_current_user_async()` | `async Option<CurrentUser>` | 异步 |

### 插件（plugin.rs）

`CommonPlugin` 启动时自动注册 `AuditFieldFillHandler`：

```toml
[common]
default_user = "system"
```

---

## 审计字段

所有实体统一包含，`CommonPlugin` → `AuditFieldFillHandler` 自动填充：

| 字段 | 类型 | 注解 | 说明 |
|------|------|------|------|
| create_time | DateTimeUtc | `#[sea_orm_ext(insert)]` | 创建时间 |
| create_by | Option\<String\> | `#[sea_orm_ext(insert)]` | 创建人名称 |
| create_id | Option\<String\> | `#[sea_orm_ext(insert)]` | 创建人ID |
| update_time | DateTimeUtc | `#[sea_orm_ext(update)]` | 修改时间 |
| update_by | Option\<String\> | `#[sea_orm_ext(update)]` | 修改人名称 |
| update_id | Option\<String\> | `#[sea_orm_ext(update)]` | 修改人ID |
| version | i32 | `#[sea_orm_ext(insert_update)]` | 乐观锁版本号 |
| delete_flag | i32 | `#[soft_delete(default=0, del=1)]` | 逻辑删除标记 |
| tenant_id | 业务定义 | `#[sea_orm_ext(TENANT)]` | 租户ID（DeriveTenant 自动填充） |

---

## 配置

完整配置参考 `config/app.toml`：

```toml
[web]
port = 8080
graceful = true

[sea-orm]
uri = "${DATABASE_URL:postgres://postgres:123456@localhost:5432/template}"
min_connections = 1
max_connections = 10
enable_logging = true

[sa-token]
token_name = "Authorization"
timeout = 86400
auto_renew = true
token_style = "Uuid"
token_prefix = "Bearer "

[common]
default_user = "system"

[sea-orm-ext.tenant]
enabled = true
mode = "table"
default_tenant_id = 1
header_name = "x-tenant-id"
ignore_if_missing = true

[storage]
enabled = true
provider = "minio"                    # minio | aliyun | tencent | huawei | custom
```

---

## 数据库调用流程

### 1. 通过 MCP 查看表结构

```sql
SELECT table_name FROM information_schema.tables WHERE table_schema = 'public' ORDER BY table_name;
SELECT column_name, data_type FROM information_schema.columns WHERE table_name = 'auth_sys_user' ORDER BY ordinal_position;
```

### 2. sea-orm-cli 生成实体

```bash
sea-orm-cli generate entity \
  -u "postgres://postgres:123456@localhost:5432/template" \
  -s public \
  -o system/entity/src \
  --with-serde both \
  --date-time-crate chrono \
  --model-extra-derives "Serialize, Deserialize" \
  --impl-active-model-behavior
```

### 3. 手动添加注解

1. 添加 `#[serde(rename_all = "camelCase")]` 到 Model struct
2. 添加 Derive 宏：`DeriveAutoFill` / `DeriveSoftDelete` / `DeriveTenant` 或组合宏
3. 审计字段：`#[sea_orm_ext(insert)]` / `#[sea_orm_ext(update)]` / `#[sea_orm_ext(insert_update)]`
4. 主键自动生成：`#[sea_orm(primary_key, auto_generate)]`
5. 逻辑删除：`#[soft_delete(default = 0, del = 1)]`
6. 租户字段：`#[sea_orm_ext(TENANT)]`

---

## RBAC2 权限模型

```
User → UserRole → Role → RolePermission → Permission
                 → Role → RoleMenu → Menu
                 → RoleHierarchy (角色继承)
超级管理员：跳过权限检查 + 跨租户查看所有数据
```

### 声明式权限校验

```rust
#[sa_check_login]                                // 已登录
#[sa_check_role("admin")]                        // 角色
#[sa_check_permission("user:list")]              // 权限
#[sa_check_permissions_and("edit", "delete")]    // AND 权限
#[sa_check_permissions_or("add", "edit")]        // OR 权限
#[sa_ignore]                                      // 跳过
```

---

## 架构演进规划

### 当前：单体架构

```
┌──────────────────────────────────────────────────────┐
│                    template-admin                     │
│  ┌────────┐  ┌────────┐  ┌────────────┐             │
│  │common  │  │system  │  │migration   │             │
│  │        │  │(DDD)   │  │            │             │
│  └────────┘  └────────┘  └────────────┘             │
└──────────────────────────────────────────────────────┘
```

### 目标：微服务架构

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│ system-svc  │  │  wms-svc    │  │  oms-svc    │
│ (基础管理)   │  │ (仓储管理)   │  │ (订单管理)   │
│  DDD 分层   │  │  DDD 分层   │  │  DDD 分层   │
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       │                │                │
       └────────────────┼────────────────┘
                        │
              ┌─────────▼─────────┐
              │   common (共享)    │
              └───────────────────┘
```

### 拆分技术路径

| 阶段 | 目标 | 技术手段 |
|------|------|---------|
| Phase 1 | 服务拆分 | 每个限界上下文独立 Crate |
| Phase 2 | 通信机制 | summer-grpc (tonic) + summer-stream (Kafka/Redis Stream) |
| Phase 3 | 网关层 | summer-web + 路由转发 |
| Phase 4 | 配置中心 | summer 配置 + 环境变量插值 |
| Phase 5 | 认证统一 | summer-sa-token + JWT 跨服务传递 |
| Phase 6 | 数据隔离 | 每个服务独立 PostgreSQL Schema |

---

## 本地开发

```bash
# 启动基础设施
docker compose up -d postgres redis minio

# 数据库迁移
cd migration && cargo run

# 后端
cargo run

# 前端
cd template-web && pnpm dev
```

---

## API 端点

### 认证
| Method | Path | 说明 |
|--------|------|------|
| POST | `/api/auth/login` | 登录 |
| POST | `/api/auth/register` | 注册 |
| POST | `/api/auth/bind` | 绑定第三方账号 |
| GET | `/api/auth/providers` | 获取认证提供者列表 |

### 系统管理
| Method | Path | 说明 |
|--------|------|------|
| GET/POST | `/api/system/users` | 用户管理 |
| GET/POST | `/api/system/roles` | 角色管理 |
| GET/POST | `/api/system/menus` | 菜单管理 |
| GET/POST | `/api/system/depts` | 部门管理 |
| GET/POST | `/api/system/tenants` | 租户管理 |
| GET/POST | `/api/system/dicts` | 字典管理 |
| GET/POST | `/api/system/configs` | 系统配置 |
| POST | `/api/files/upload` | 文件上传 |