<div align="center">

# Template Admin

**基于 Rust + Vue 3 的企业级后台管理模板**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Vue](https://img.shields.io/badge/Vue-3.5-brightgreen.svg)](https://vuejs.org/)

[English](#english) | [中文](#中文)

</div>

---

<a id="中文"></a>

## ✨ 项目简介

Template Admin 是一套全栈后台管理解决方案，后端基于 **Summer-rs + Sea-ORM 2.0** 采用 DDD 分层架构，前端基于 **Vue 3 + Ant Design Vue + TypeScript**，开箱即用。

![](dashboard_final.png)

## 🏗️ 技术栈

### 后端

| 技术 | 版本 | 说明 |
|------|------|------|
| Rust | Edition 2024 | tokio 异步运行时 |
| Summer-rs | 0.5.x | 类 SpringBoot 的 Rust 应用框架 |
| Summer-web (Axum) | 0.5.0 | Web 框架，路由宏/中间件/提取器 |
| Sea-ORM | 2.0.x | 异步 ORM |
| sea-orm-ext | - | Auto-Fill / Soft-Delete / Multi-Tenant / IdGenerator |
| Summer-sa-token | 0.5.3 | 权限认证框架 |
| Summer-redis | 0.5.0 | Redis 集成 |
| PostgreSQL | 17 | 关系型数据库 |
| Redis | 7 | 缓存 / Session 存储 |
| MinIO | latest | 对象存储 |

### 前端

| 技术 | 版本 | 说明 |
|------|------|------|
| Vue | 3.5 | 渐进式前端框架 |
| TypeScript | 5.6 | 类型安全 |
| Ant Design Vue | 4.2 | UI 组件库 |
| Vite | 6.0 | 构建工具 |
| Pinia | 2.2 | 状态管理 |
| Vue Router | 4.4 | 路由管理 |
| Vue I18n | 9.14 | 国际化 |
| Axios | 1.7 | HTTP 客户端 |
| Day.js | 1.11 | 日期处理 |

## 📸 功能截图

| Dashboard | 用户管理 |
|-----------|---------|
| ![](dashboard_final.png) | ![](user_mgmt.png) |

| 菜单管理 | 系统管理 |
|---------|---------|
| ![](menu_mgmt.png) | ![](dashboard_new.png) |

## 🌟 核心特性

- 🔐 **RBAC2 双层权限** — summer-sa-token（HTTP 级）+ sea-orm RBAC 引擎（数据级），声明式注解校验
- 🏢 **多租户** — 表隔离 / 数据库隔离双模式，`TenantGuard` RAII 自动管理
- 📝 **审计自动填充** — `CommonPlugin` 注册 `AuditFieldFillHandler`，create_by/update_by/version 等字段零代码填充
- 🗑️ **逻辑删除** — `DeriveSoftDelete` 宏，DELETE 自动转 UPDATE，`find_active()` 自动过滤
- 🔑 **主键自动生成** — 雪花算法 / UUID，`#[sea_orm(primary_key, auto_generate)]` 声明即启用
- 📦 **批量操作** — 宏自动生成 `insert_many_with_fill` / `update_many_with_fill` / `delete_many_soft`
- 🌍 **国际化** — 前后端均支持中英文切换
- 🎨 **主题切换** — 亮色/暗色主题，字体大小调节
- 📁 **文件存储** — 支持 MinIO / 阿里云 / 腾讯云 / 华为云 / 自定义
- 🔗 **OAuth2 授权** — 通用 AuthProvider trait，配置添加任意 OAuth2 提供者

## 📁 项目结构

```
template_rs/
├── template-admin/                    # 后端 Workspace
│   ├── common/                        # 共享内核：错误/返回/分页 + StpUtil 封装 + CommonPlugin
│   ├── system/
│   │   ├── entity/                    # Sea-ORM 实体（cli 生成 + sea-orm-ext 注解）
│   │   ├── domain/                    # 领域层：仓储 trait + 领域服务
│   │   ├── application/               # 应用层：DTO + 应用服务
│   │   ├── infrastructure/            # 基础设施：仓储实现 + 中间件
│   │   └── interface/                 # 接口层：HTTP handlers
│   ├── migration/                     # 数据库迁移（sea-orm-migration）
│   ├── config/app.toml                # 配置文件
│   ├── locales/                       # 后端 i18n 资源
│   ├── src/main.rs                    # 入口（插件注册 + 路由挂载）
│   └── docker-compose.yml             # PostgreSQL + Redis + MinIO
│
└── template-web/                      # 前端
    ├── src/
    │   ├── api/                       # API 请求封装
    │   ├── components/                # 公共组件（ProTable/ProForm/DictSelect/UserPicker）
    │   ├── layouts/                   # 布局（BasicLayout/BlankLayout）
    │   ├── locales/                   # 前端 i18n
    │   ├── router/                    # 路由配置 + 权限守卫
    │   ├── stores/                    # Pinia 状态管理
    │   ├── styles/                    # 全局样式
    │   └── views/                     # 页面视图
    ├── package.json
    └── vite.config.ts
```

### DDD 分层依赖

```
system/interface → system/application → system/domain ← system/infrastructure
                                                ↓              ↓
                                            common        system/entity
```

> 依赖方向不可反向。新建业务服务（如 wms）时，依赖 `common` 即可获得全部公共能力。

## 🚀 快速开始

### 环境要求

- Rust 1.85+ (Edition 2024)
- Node.js 18+
- pnpm 8+
- Docker & Docker Compose

### 1. 启动基础设施

```bash
cd template-admin
docker compose up -d
```

启动后服务：
- PostgreSQL: `localhost:5432` (用户: postgres / 密码: 123456 / 数据库: template)
- Redis: `localhost:6379`
- MinIO: `localhost:9000` (控制台: `localhost:9001`)

### 2. 数据库迁移

```bash
cd template-admin/migration
cargo run
```

### 3. 启动后端

```bash
cd template-admin
cargo run
```

后端启动在 `http://localhost:8080`

### 4. 启动前端

```bash
cd template-web
pnpm install
pnpm dev
```

前端启动在 `http://localhost:3000`，自动代理 `/api` 请求到后端

## ⚙️ 配置说明

后端配置文件：`template-admin/config/app.toml`

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
token_style = "Jwt"
token_prefix = "Bearer "

[redis]
uri = "redis://localhost:6379"

[common]
default_user = "system"

[sea-orm-ext.tenant]
enabled = true
mode = "table"
default_tenant_id = 1
header_name = "x-tenant-id"
ignore_if_missing = true

[storage]
enabled = false
provider = "minio"
```

前端环境变量：`template-web/.env.development`

```
VITE_API_BASE_URL=/api
VITE_APP_TITLE=Template Admin
```

## 📡 API 端点

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

## 🧩 后端开发指南

### 新增实体检查清单

1. `system/entity/src/` — 新增 Sea-ORM 实体（含 Derive 宏 + `#[serde(rename_all = "camelCase")]`）
2. `system/entity/src/lib.rs` — 导出模块
3. `migration/src/` — 新增迁移脚本
4. `migration/src/lib.rs` — 注册迁移
5. `system/domain/src/` — 新增仓储 trait
6. `system/infrastructure/src/persistence/` — 新增仓储实现
7. `system/application/src/` — 新增 DTO + 应用服务
8. `system/interface/src/handlers/` — 新增 HTTP handler

### 实体定义示例

```rust
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
    #[sea_orm_ext(update)]
    pub update_time: DateTimeUtc,
    #[sea_orm_ext(update)]
    pub update_by: Option<String>,
    #[sea_orm_ext(insert_update)]
    pub version: i32,
    #[soft_delete(default = 0, del = 1)]
    pub delete_flag: i32,
    #[sea_orm_ext(TENANT)]
    pub tenant_id: Option<String>,
}
```

### Handler 示例

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

### 声明式权限校验

```rust
#[sa_check_login]                             // 已登录
#[sa_check_role("admin")]                     // 角色校验
#[sa_check_permission("user:list")]           // 权限校验
#[sa_check_permissions_and("edit", "delete")] // AND 权限
#[sa_check_permissions_or("add", "edit")]     // OR 权限
#[sa_ignore]                                   // 跳过校验
```

### sea-orm-cli 生成实体

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

## 🎨 前端开发指南

### 目录约定

| 目录 | 说明 |
|------|------|
| `src/api/` | API 请求封装，按模块拆分 |
| `src/components/` | 公共组件（ProTable、ProForm、DictSelect、UserPicker） |
| `src/views/` | 页面视图，按功能模块组织 |
| `src/stores/` | Pinia 状态管理 |
| `src/locales/` | 国际化资源（zh-CN / en-US） |
| `src/layouts/` | 页面布局组件 |

### 新增页面

1. 在 `src/views/` 下创建页面组件
2. 在 `src/router/index.ts` 中注册路由
3. 在 `src/api/` 中封装对应 API
4. 在 `src/locales/` 中添加国际化文本

### 权限控制

路由级别权限通过 `meta.permission` 字段控制，组件级别使用 `v-if="userStore.hasPermission('xxx')"` 控制。

## 📦 sea-orm-ext 能力一览

| 能力 | Derive 宏 | 注解 | 说明 |
|------|-----------|------|------|
| Auto-Fill | `DeriveAutoFill` | `#[sea_orm_ext(insert/update/insert_update)]` | INSERT/UPDATE 时自动填充字段 |
| ID 生成 | 内置 | `#[sea_orm(primary_key, auto_generate)]` | 主键为空时自动调用 IdGenerator |
| Soft-Delete | `DeriveSoftDelete` | `#[soft_delete(default=0, del=1)]` | DELETE 转 UPDATE，`find_active()` 过滤 |
| Multi-Tenant | `DeriveTenant` | `#[sea_orm_ext(TENANT)]` | 表隔离 / 数据库隔离 |
| 组合宏 | `DeriveAutoFillSoftDeleteTenant` | 组合注解 | 同时启用全部能力 |

## 🏛️ 架构演进

### 当前：单体架构

```
┌──────────────────────────────────────────┐
│              template-admin               │
│  ┌────────┐  ┌────────┐  ┌────────────┐ │
│  │ common │  │ system │  │ migration  │ │
│  │        │  │ (DDD)  │  │            │ │
│  └────────┘  └────────┘  └────────────┘ │
└──────────────────────────────────────────┘
```

### 目标：微服务架构

```
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│ system-svc  │  │  wms-svc    │  │  oms-svc    │
│ (基础管理)   │  │ (仓储管理)   │  │ (订单管理)   │
│  DDD 分层   │  │  DDD 分层   │  │  DDD 分层   │
└──────┬──────┘  └──────┬──────┘  └──────┬──────┘
       └────────────────┼────────────────┘
                        │
              ┌─────────▼─────────┐
              │   common (共享)    │
              └───────────────────┘
```

| 阶段 | 目标 | 技术手段 |
|------|------|---------|
| Phase 1 | 服务拆分 | 每个限界上下文独立 Crate |
| Phase 2 | 通信机制 | summer-grpc (tonic) + summer-stream (Kafka/Redis Stream) |
| Phase 3 | 网关层 | summer-web + 路由转发 |
| Phase 4 | 配置中心 | summer 配置 + 环境变量插值 |
| Phase 5 | 认证统一 | summer-sa-token + JWT 跨服务传递 |
| Phase 6 | 数据隔离 | 每个服务独立 PostgreSQL Schema |

## 📄 License

[Apache License 2.0](LICENSE)
