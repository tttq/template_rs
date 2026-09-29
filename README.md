<div align="center">

# Template Admin

**基于 Rust + Vue 3 的企业级后台管理模板**

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-2024-orange.svg)](https://www.rust-lang.org/)
[![Vue](https://img.shields.io/badge/Vue-3.5-brightgreen.svg)](https://vuejs.org/)

</div>

---

## ✨ 项目简介

Template Admin 是一套全栈后台管理模板，后端基于 **Summer-rs + Sea-ORM 2.0** 采用 DDD 分层架构，前端基于 **Vue 3 + Ant Design Vue + TypeScript**，开箱即用。

仓库只包含「基础管理」能力（认证 + 系统管理），不含任何业务域代码，可直接作为新项目起点：新增业务模块时依赖 `common` 即可获得鉴权、多租户、审计填充、附件、通知、导出等全部公共能力。

![](dashboard_final.png)

## 🏗️ 技术栈

### 后端

| 技术 | 版本 | 说明 |
|------|------|------|
| Rust | Edition 2024 | tokio 异步运行时 |
| summer | 0.7.1 | 类 SpringBoot 的 Rust 应用框架（插件 / 配置 / DI） |
| summer-web | 0.7.1 | Web 框架（Axum），路由宏 / 中间件 / 提取器 |
| Sea-ORM | 2.0.0 | 异步 ORM |
| sea-orm-ext | 0.0.4 | Auto-Fill / Soft-Delete / 多租户 / 主键生成 / 动态租户 |
| summer-sa-token | 0.7.1 | 权限认证框架（JWT + Redis 存储 + refresh token） |
| summer-redis | 0.7.1 | Redis 集成 |
| summer-mail | 0.7.1 | SMTP 邮件（验证码 / 通知） |
| fast-excel | 0.3.0 | 列表 Excel 同步导出 + 异步导出中心 |
| rust-s3 | 0.35 | 附件中心对象存储驱动（RustFS / MinIO 等 S3 兼容服务） |
| image + font8x8 | 0.25 / 0.2 | 图形验证码生成（点阵字体，无字体文件依赖） |
| inventory | 0.3 | 启动期自动注册（路由 auto_router / 异步导出任务） |
| PostgreSQL | 17 | 关系型数据库 |
| Redis | 7 | 会话 / 缓存 / 限流 / 权限变更广播 |

### 前端

| 技术 | 版本 | 说明 |
|------|------|------|
| Vue | 3.5 | 渐进式前端框架（`<script setup>`） |
| TypeScript | 5.6 | 类型安全 |
| Ant Design Vue | 4.2 | UI 组件库 |
| Vite | 6.0 | 构建工具 |
| Pinia | 2.2 | 状态管理 |
| Vue Router | 4.4 | 路由（静态路由 + 后端菜单动态路由） |
| Vue I18n | 9.14 | 国际化（zh-CN / en-US） |
| Axios | 1.7 | HTTP 客户端（json-bigint 防雪花 ID 精度丢失） |
| Day.js | 1.11 | 日期处理 |

## 🌟 核心特性

- 🔐 **RBAC2 双层权限** — summer-sa-token（HTTP 级）+ 菜单/按钮权限码（数据级），声明式注解校验
- 🖥️ **客户端维度** — 客户端 → 菜单 → 角色 → 用户，同一账号在不同端拥有隔离的权限快照
- 🏢 **多租户** — 表隔离 / 数据库隔离双模式，`TenantGuard` / `#[ignore_tenant]` 显式控制跨租户查询
- 📝 **审计自动填充** — `AuditFieldFillHandler` 通过 `FieldFillHandlerComponent` 注册，create_by/create_id/version 等字段零代码填充
- 🗑️ **逻辑删除** — `DeriveSoftDelete` 宏，DELETE 自动转 UPDATE，`Entity::find()` 自动过滤已删数据
- 🔑 **主键自动生成** — 雪花算法，`#[sea_orm(primary_key, auto_generate)]` 声明即启用
- 📦 **批量操作** — 宏自动生成 `insert_many_with_fill` / `update_many_with_fill` / `delete_many_soft`
- 🌍 **国际化** — 后端 `@key` 消息 + locales 翻译中间件，前端 vue-i18n，均支持中英切换
- 🎨 **主题切换** — 亮色/暗色主题、字体大小调节，统一走 `--brand-*` 主题令牌
- 📎 **附件中心** — 统一上传 / 直读 / 图片压缩 / 级联删除，存储驱动支持本地磁盘与 S3 兼容对象存储
- 🔔 **通知中心** — 站内信 + 邮件双通道，标题/正文由「通知模板」渲染（`${varName}` 占位），顶栏铃铛实时未读数
- 📊 **导出中心** — 列表导出自动分流：≤ 10 万条同步下载，超阈值转异步任务（生成文件落附件中心，可轮询与下载）
- 🔗 **第三方登录** — 微信扫码登录（开放平台网站应用）/ 第三方身份绑定，配置环境变量即生效

## 📦 功能模块

| 分组 | 模块 |
|------|------|
| 认证 | 登录（密码 / 邮箱验证码 / 微信扫码）、注册、图形验证码、邮箱验证码、完善账户资料、个人中心（改昵称/手机号、改密码、换邮箱、微信绑定）、登出、刷新令牌 |
| 系统管理 | 用户、角色、菜单、部门、租户、字典、参数配置、服务监控、代码生成、附件管理、通知中心、通知模板、客户端管理、导出中心 |
| 其他 | 仪表盘、403/404 |

## 📁 项目结构

```
template_rs/
├── template-admin/                    # 后端 Workspace
│   ├── common/                        # 共享内核：响应/分页/错误/加密/用户上下文/i18n/通知/Excel/邮件
│   ├── system/                        # 系统管理（DDD 分层，唯一含 domain 层的模块）
│   │   ├── entity/                    # Sea-ORM 实体（sea-orm-ext 注解）
│   │   ├── domain/                    # 领域层：仓储 trait
│   │   ├── application/               # 应用层：DTO + AppService（业务编排）
│   │   ├── infrastructure/            # 基础设施：仓储实现 + 附件存储驱动
│   │   └── interface/                 # 接口层：HTTP handlers（#[nest] + 路由宏）
│   ├── migration/                     # 数据库脚本 + 自定义运行器（非 sea-orm Migrator）
│   ├── config/app.toml                # 配置（支持 ${ENV:default} 插值）
│   ├── locales/                       # 后端 i18n 资源（zh-CN.json / en-US.json）
│   ├── src/main.rs                    # 入口：插件装配 + 路由挂载 + 后台任务
│   ├── src/config.rs                  # sa-token 免登录路径白名单
│   └── docker-compose.yml             # PostgreSQL + Redis + MinIO
│
└── template-web/                      # 前端
    ├── src/
    │   ├── api/                       # 接口封装（request.ts 为唯一 HTTP 出口）
    │   │   └── system/                # 通知 / 通知模板 / 导出任务
    │   ├── components/                # 业务组件（附件上传/缩略图、微信扫码弹窗、DictSelect、UserPicker）
    │   │   ├── common/                # ProTable / ProForm
    │   │   └── header/                # 顶栏（主题、字号、语言、通知铃铛）
    │   ├── config/                    # 客户端标识（client.ts）
    │   ├── directives/                # v-permission
    │   ├── layouts/                   # BasicLayout / BlankLayout
    │   ├── locales/                   # 前端 i18n（zh-CN / en-US）
    │   ├── router/                    # 静态路由 + 守卫（index.ts）、菜单动态路由（dynamic.ts）
    │   ├── stores/                    # Pinia（user / app）
    │   ├── styles/                    # global.less（工具类 + 主题令牌）
    │   ├── utils/                     # 表格高度自适应 / 树工具 / 下载 / 权限变更 SSE
    │   └── views/                     # 页面（dashboard / login / profile / profile-setup / system / error）
    ├── package.json
    └── vite.config.ts
```

### DDD 分层依赖

```
system/interface → system/application → system/domain ← system/infrastructure
                                                ↓              ↓
                                            common        system/entity
```

> 依赖方向不可反向。新建业务模块（如 wms）时，依赖 `common` 即可获得全部公共能力；
> `domain` 层是可选的——只有需要依赖倒置（多存储实现 / 复杂领域规则）时才引入。

## 🚀 快速开始

### 环境要求

- Rust 1.85+（Edition 2024）
- Node.js 18+ / pnpm 11.15+
- PostgreSQL 17、Redis 7（可用 Docker Compose 一键启动）

### 1. 启动基础设施

```bash
cd template-admin
docker compose up -d
```

启动后服务（默认值，可用环境变量覆盖）：

| 服务 | 地址 | 默认凭据 |
|------|------|---------|
| PostgreSQL | `localhost:5432` | `postgres` / `123456`，库 `template` |
| Redis | `localhost:6379` | — |
| MinIO（可选，作附件对象存储） | `localhost:9000`（控制台 `9001`） | `minioadmin` / `minioadmin` |

> 使用已有的本地 PostgreSQL/Redis 时跳过本步，把连接串写进 `template-admin/.env` 的 `DATABASE_URL` 即可
> （`app.toml` 的兜底默认值是 `postgres://postgres:root@localhost:5432/template2`，按需修改）。

### 2. 数据库迁移

```bash
cd template-admin/migration
cargo run           # 建表 + 初始数据 + 菜单权限 + 系统管理模块数据
cargo run -- verify # 可选：核对已建表与账号/菜单数量
```

### 3. 启动后端

```bash
cd template-admin
cargo run
```

后端启动在 `http://localhost:8080`（健康检查 `GET /api/health`）。

### 4. 启动前端

```bash
cd template-web
pnpm install
pnpm dev
```

前端启动在 `http://localhost:3000`，开发期由 Vite 代理 `/api` → `http://127.0.0.1:8080`。

### 5. 登录

初始账号：`admin` / `admin123`（迁移脚本会把明文替换为 bcrypt 哈希，生产环境请立即修改）。

## ⚙️ 配置说明

### 后端：`template-admin/config/app.toml`

```toml
[web]
port = 8080
global_prefix = "api"                   # /api 前缀由此提供，不要写进 handler 的 #[nest]

[web.middlewares]
limit_payload = { enable = true, body_limit = "64MB" }  # 附件上传需要；缺省即 axum 的 2MB 限制

[sea-orm-ext]
uri = "${DATABASE_URL:postgres://postgres:root@localhost:5432/template2}"
enable_sql_log = true

[sa-token]
token_name = "Authorization"
token_style = "Jwt"
storage_key_prefix = "template"         # Redis key 前缀
enable_refresh_token = true
is_concurrent = true
max_login_count = 1                     # 每客户端只保留 1 个会话，避免 Redis 索引无界膨胀

[sea-orm-ext-tenant]
enabled = true
mode = "${TENANT_MODE:table}"           # table=表隔离 / database=独立库
default_tenant_id = "1876543210000000001"
tenant_id_header = "x-tenant-id"

[attachment.storage]                    # 嵌套表：代码按顶层键 attachment 读取，前缀别写成 attachment.storage
driver = "local"                        # local | rustfs（S3 兼容，RustFS / MinIO 均可）
local_path = "./uploads/attachment"
rustfs_endpoint = "http://localhost:9000"
rustfs_bucket = "template"

[mail]                                  # summer-mail 按此段自动装配 SMTP
host = "smtp.example.com"
port = 465
secure = true
stub = true                             # true=只打日志不发送（本地联调）；真实发送置 false
[mail.auth]
user = ""
password = ""                           # 网易 163 等填「客户端授权码」，不是登录密码
```

### 后端：环境变量（`template-admin/.env`，可复制 `.env.example`）

| 变量 | 说明 |
|------|------|
| `DATABASE_URL` | 数据库连接串（迁移与后端共用） |
| `JWT_SECRET_KEY` | JWT 签名密钥，**生产必须修改** |
| `TENANT_MODE` | `table`（表隔离，默认）/ `database`（独立库） |
| `WECHAT_APPID` / `WECHAT_SECRET` / `WECHAT_OPEN_REDIRECT_URI` | 微信开放平台「网站应用」，配好才可微信扫码登录/绑定 |

### 前端：`template-web/.env.development` / `.env.production`

```
VITE_API_BASE_URL=/api            # 请求层 baseURL，默认同源 /api（生产由 nginx 反代）
VITE_APP_TITLE=Template Admin     # 站点标题
VITE_CLIENT_CODE=web-admin        # 当前前端所属客户端（对应「客户端管理」）
VITE_CLIENT_SECRET=web-admin-secret
```

## 📡 API 端点

> 统一返回 `{ code, message, data }`：业务错误为 HTTP 200 + `code:500`；未登录 401、无权限 403、两步验证 428。
> `message` 支持 `@key` 形式，由后端按请求 `Accept-Language` 翻译为中文/英文。

### 认证（`/api/auth`）

| Method | Path | 说明 |
|--------|------|------|
| POST | `/api/auth/login` | 登录（`loginType`: `password` / `email_code` / `wechat`） |
| POST | `/api/auth/register` | 注册（邮箱 + 邮箱验证码 + 身份 `roleCode`） |
| POST | `/api/auth/locate` | 第一步：按用户名定位租户 |
| GET | `/api/auth/user-info` | 当前用户信息（含菜单树 / 角色 / 权限） |
| POST | `/api/auth/logout` | 登出 |
| POST | `/api/auth/refresh-token` | 刷新令牌 |
| GET | `/api/auth/captcha` | 图形验证码（`captchaId` + PNG data URI） |
| GET | `/api/auth/captcha-image/{id}` | 图形验证码原图（小程序按 URL 加载） |
| POST | `/api/auth/send-code` | 发送邮箱验证码（`scene`: register / reset / login / change_password / change_email） |
| GET | `/api/auth/register-roles` | 注册可选身份（免登录，字典驱动） |
| GET | `/api/auth/wechat/qr-url` | 微信扫码登录二维码地址 + 一次性 state |
| POST | `/api/auth/complete-profile` | 完善账户资料（第三方自动注册的惰性账号） |
| PUT | `/api/auth/profile` | 修改昵称 / 手机号 |
| POST | `/api/auth/change-password` | 修改密码（邮箱验证码） |
| POST | `/api/auth/change-email` | 更换绑定邮箱（新邮箱验证码） |
| GET/POST | `/api/auth/providers` `/api/auth/bind` `/api/auth/bindings` `/api/auth/unbind` | 第三方身份列表 / 绑定 / 查询 / 解绑 |
| GET | `/api/auth/notify/stream` | 权限变更 SSE（前端据此刷新菜单） |

### 系统管理（`/api/system`）

| Method | Path | 说明 |
|--------|------|------|
| GET/POST | `/api/system/users` | 用户管理（含 `/users/export`、`/users/export/count`） |
| GET/POST | `/api/system/roles` | 角色管理（含菜单授权、导出） |
| GET/POST | `/api/system/menus` | 菜单管理（含 `/menus/tree?clientId=`、导出） |
| GET/POST | `/api/system/depts` | 部门管理（含导出） |
| GET/POST | `/api/system/tenants` | 租户管理（含连通性测试 / 建库 / 初始化） |
| GET/POST | `/api/system/dicts/types`、`/api/system/dicts/items` | 字典类型与字典项 |
| GET/POST | `/api/system/configs` | 参数配置（含导出） |
| GET/POST | `/api/system/clients` | 客户端管理（`/clients/options` 供下拉） |
| GET/POST | `/api/system/attachment` | 附件管理（上传 / 列表 / 直读 / 元信息 / 删除 / 批量删除） |
| GET/POST | `/api/system/notification` | 通知中心（我的通知 / 未读数 / 标记已读 / 发送 / 广播 / 接收人候选） |
| GET/POST | `/api/system/notification-template` | 通知模板（增删改查 / 启停 / 预览） |
| GET/POST | `/api/system/exports` | 导出中心（提交 / 列表 / 详情 / 下载） |
| GET | `/api/system/server/info`、`/api/system/redis/info` | 服务监控 |
| GET/POST | `/api/system/tables`、`/api/system/tables/{table}/columns`、`/api/system/preview`、`/api/system/download` | 代码生成（表/字段列表、预览、下载 zip） |
| GET | `/api/health` | 健康检查（数据库 + Redis） |

## 🧩 后端开发指南

> 详细规范（路由铁律、事务、软删语义、多租户、陷阱清单）见 [`CLAUDE.md`](CLAUDE.md)。

### 路由注册：`#[nest]` + 路由宏，自动发现

```rust
use summer_web::{get, nest};
use summer_web::extractor::{Component, Query};

#[nest("/system")]
mod controller {
    use super::*;

    #[get("/users")]
    #[sa_check_permission("user:list")]
    async fn list_users(
        Component(service): Component<UserAppService>,
        Query(query): Query<UserQuery>,
    ) -> Result<impl IntoResponse, WebError> {
        match service.list(query).await {
            Ok(v) => Ok(Json(ApiResponse::success(v))),
            Err(e) => Ok(Json(ApiResponse::error(500, &e.to_string()))),
        }
    }
}
```

- `.nest` 只写子路径（`/system`），`/api` 由 `[web] global_prefix` 统一叠加；
- 新增 handler：建文件 + 在 `handlers/mod.rs` 加一行 `pub mod xxx_handler;`，路由自动注册；
- 权限宏：`#[sa_check_permission("user:list")]`（管理端）、`#[sa_check_login]`（仅登录）、`#[sa_ignore]`（开放端点）。

### 新增实体/模块检查清单

1. `system/entity/src/xxx.rs` — Sea-ORM 实体（Derive 宏 + `#[serde(rename_all = "camelCase")]`），并在 `entity/src/lib.rs` 导出
2. `system/domain/src/` — 需要依赖倒置时加仓储 trait
3. `system/infrastructure/src/persistence/` — 仓储实现
4. `system/application/src/xxx/{mod.rs,dto.rs,service.rs}` — DTO + `XxxAppService`（`#[derive(Clone, Service)]` + `#[inject(component)] db: DbConn`）
5. `system/interface/src/handlers/xxx_handler.rs` — HTTP handler，并在 `handlers/mod.rs` 声明
6. `migration/src/` — 建表 + 菜单/权限/角色绑定 SQL（参照 `m20260929_000004_system_modules.sql`）
7. `cargo check --workspace`

### 实体定义示例

```rust
#[derive(Clone, Debug, Serialize, Deserialize, DeriveEntityModel, DeriveAutoFillSoftDeleteTenant)]
#[sea_orm(table_name = "auth_sys_user")]
#[serde(rename_all = "camelCase")]
pub struct Model {
    #[sea_orm(primary_key, auto_generate)]
    pub id: String,
    pub user_name: String,
    #[sea_orm_ext(insert)]
    pub create_time: DateTimeUtc,
    #[sea_orm_ext(insert)]
    pub create_by: Option<String>,
    #[sea_orm_ext(insert_update)]
    pub version: i32,
    #[soft_delete(default = 0, del = 1)]
    pub delete_flag: i32,
    #[sea_orm_ext(TENANT)]
    pub tenant_id: Option<String>,
}
```

### 声明式权限校验

```rust
#[sa_check_login]                             // 已登录
#[sa_check_role("admin")]                     // 角色校验
#[sa_check_permission("user:list")]           // 权限校验
#[sa_check_permissions_and("edit", "delete")] // AND 权限
#[sa_check_permissions_or("add", "edit")]     // OR 权限
#[sa_ignore]                                  // 跳过校验（需同时加进 src/config.rs 免登录白名单）
```

### 后端国际化

面向用户的消息统一以 `@key` 形式抛出（如 `@user_not_found`），由 `system_interface::routes::with_i18n` 中间件按请求语言翻译：

```rust
return Err(AppError::NotFound("@user_not_found".to_string()));
```

新增消息**必须同时**在 `template-admin/locales/zh-CN.json` 与 `en-US.json` 补 key，否则前端会原样显示 `@user_not_found`。

## 🎨 前端开发指南

> 列表页三段式布局、ProTable/ProForm 用法、路由守卫要点与陷阱清单见 [`CLAUDE.md`](CLAUDE.md)。

| 目录 | 说明 |
|------|------|
| `src/api/` | 接口封装，按模块拆分；`request.ts` 是唯一 HTTP 出口 |
| `src/components/` | 公共组件（ProTable、ProForm、DictSelect、UserPicker、附件上传/预览） |
| `src/views/` | 页面视图，按功能模块组织 |
| `src/stores/` | Pinia 状态管理 |
| `src/locales/` | 国际化资源（zh-CN / en-US） |
| `src/layouts/` | 页面布局组件 |

### 新增菜单页（4 步）

1. `src/api/xxx.ts` 定义接口；
2. 建 `src/views/<path>/index.vue`（列表页按「搜索卡 + 表格卡」三段式）；
3. 在「系统管理 → 菜单管理」新增菜单，配 `path` + `component`（如 `system/xxx/index`）+ `permission`；
4. 新增按钮权限时同步 `v-permission` 与菜单表 `permission`，最后跑 `pnpm build`。

> 菜单页路由**不需要**改 `router/index.ts`：登录后由后端菜单树动态生成。仅「无菜单的二级页」才写静态子路由。

### 权限控制

- 路由级：菜单 `permission` → 动态路由 `meta.permission`，不满足跳 `/403`；
- 元素级：`v-permission="'user:add'"`（`admin` 角色直通，支持数组任一命中，无权限时移除元素）；
- 权限码必须与后端 `#[sa_check_permission]`、菜单表 `permission` **三处一致**。

## 📦 sea-orm-ext 能力一览

| 能力 | Derive 宏 | 注解 | 说明 |
|------|-----------|------|------|
| Auto-Fill | `DeriveAutoFill` | `#[sea_orm_ext(insert/update/insert_update)]` | INSERT/UPDATE 时自动填充字段 |
| ID 生成 | 内置 | `#[sea_orm(primary_key, auto_generate)]` | 主键为空时自动调用 IdGenerator |
| Soft-Delete | `DeriveSoftDelete` | `#[soft_delete(default=0, del=1)]` | DELETE 转 UPDATE，`Entity::find()` 自动过滤 |
| Multi-Tenant | `DeriveTenant` | `#[sea_orm_ext(TENANT)]` | 表隔离（自动注入 `WHERE tenant_id`）/ 数据库隔离 |
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