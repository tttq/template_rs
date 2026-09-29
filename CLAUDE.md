# CLAUDE.md — template_rs 开发规范

> 适用范围：整个仓库。后端（`template-admin/`）见 §1~§11，前端（`template-web/`）见 §12~§17，
> 跨端陷阱清单见 §18。
>
> **提交前必跑**：后端 `cargo check --workspace`；前端 `pnpm build`（`vue-tsc` 查不出模板标签不平衡）。

## 0. 通用原则

1. **面向用户的消息一律中文**。后端用 `@key` + locales 翻译（§6），前端直接写中文或走 i18n key（§15）。
2. **只做被要求的改动**：不顺手重构、不改与本次无关的格式与注释、不删除历史遗留代码（除非确认无用）。
3. **保持分层与既有风格**：新增代码照抄同层既有文件的写法，不要引入第二套模式。
4. **本仓库只承载「基础管理」**：认证 + 系统管理。不要引入与模板无关的业务域（AI 中转、采购、报销、支付等）代码与依赖。
5. **可验证再交付**：改后端跑 `cargo check`，改前端跑 `pnpm build`，改迁移就跑一次 `cargo run -p migration`（可用独立库验证）。

---

# 一、后端（template-admin）

## 1. 定位与技术栈

单进程 Rust 服务，对外提供系统管理 API（含认证）。

| 维度 | 选型 |
|---|---|
| 语言/版本 | Rust Edition 2024，tokio 异步运行时 |
| Web 框架 | `summer` 0.7 + `summer-web` 0.7（Axum），路由宏 / 中间件 / 提取器 |
| ORM | `sea-orm` 2.0 + `sea-orm-ext` 0.0.4（Auto-Fill / Soft-Delete / 多租户 / ID 生成 / 动态租户） |
| 认证鉴权 | `summer-sa-token` 0.7（JWT HS256 + Redis 存储 + refresh token） |
| 缓存 | `summer-redis` 0.7（会话 / 限流 / 权限变更广播） |
| 邮件 | `summer-mail` 0.7（SMTP，`[mail]` 段自动装配） |
| Excel | `fast-excel` 0.3（列表同步导出 + 导出中心异步任务） |
| 对象存储 | `rust-s3` 0.35（S3 兼容：RustFS / MinIO）；或本地磁盘 |
| 数据库 | PostgreSQL 17 |

## 2. 目录结构与分层

```
template-admin/
├── common/            # 共享内核：响应/分页/错误/加密/用户上下文/i18n/通知/邮件/Excel/模板渲染
├── config/app.toml    # 配置（支持 ${ENV:default} 插值）
├── locales/           # 后端 i18n（zh-CN.json / en-US.json）
├── migration/         # SQL 脚本 + 自定义运行器（不是 sea-orm Migrator）
├── src/main.rs        # 入口：插件装配 + 路由挂载 + 后台任务
├── src/config.rs      # sa-token 免登录路径白名单
└── system/            # 系统管理（唯一有 domain 层的模块）
    ├── entity/        # SeaORM 实体
    ├── domain/        # 仓储 trait（依赖倒置）
    ├── application/   # DTO + AppService（业务编排 / 校验 / 事务）
    ├── infrastructure/# 仓储实现 + 附件存储驱动
    └── interface/     # HTTP handler（#[nest] + 路由宏）
```

| 层 | 职责 | 约定 |
|---|---|---|
| `entity` | 表映射 | `lib.rs` 顶部必须保留 `extern crate sea_orm_ext as summer_sea_orm_ext;` |
| `application` | 业务编排 | `#[derive(Clone, Service)]` + `#[inject(component)] db: DbConn` |
| `interface` | HTTP | `#[nest("/xxx")] mod controller { ... }` + 路由宏 + 权限宏 |
| `infrastructure` | 外部依赖实现 | 仓储实现、附件存储驱动 |

依赖方向：`interface → application → domain ← infrastructure`，不可反向；`common` 与 `entity` 被各层共享。

## 3. 启动 / 构建 / 迁移

| 目的 | 命令（在 `template-admin/` 下执行） |
|---|---|
| 编译检查（提交前必跑） | `cargo check --workspace` |
| 运行后端（默认 :8080） | `cargo run` |
| 初始化 / 重建数据库 | `cargo run -p migration` |
| 核对已建表与账号/菜单数量 | `cargo run -p migration verify` |
| 生产构建 | `cargo build --release` |

基础设施：`docker compose up -d`（PostgreSQL + Redis + MinIO）。

连接串优先级：`DATABASE_URL` 环境变量 → `config/app.toml` 的 `[sea-orm-ext].uri` 兜底默认值。

## 4. 路由注册（铁律）

采用 **Spring Controller 风格**：`#[nest]` 前缀 + 路由宏，`auto_router()` 通过 `inventory` 跨 crate 自动发现。

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
    ) -> Result<impl IntoResponse, WebError> { ... }
}
```

- 新增 handler：**建文件 + 在 `handlers/mod.rs` 加一行 `pub mod xxx_handler;`** 即可，路由自动注册；
- **不要**写 `routes()` 聚合函数，也**不要**在 `main.rs` 手工 `nest`；
- **不要**用 `#[auto_config(WebConfigurator)]`：它只改写裸表达式 `App::new()`，本项目用 `let mut app = App::new();` 绑定会静默失效；
- 新增独立 crate（如 `wms-interface`）必须在 `src/main.rs` 用 `use wms_interface as _;` 链接，否则 `inventory` 收集不到其路由；
- `#[nest]` 只写子路径（`/system`），**`/api` 由 `[web] global_prefix` 统一叠加，不要写进 nest**。

## 5. 认证与鉴权

| 宏 | 用途 |
|---|---|
| `#[sa_check_permission("user:list")]` | 管理端接口：校验权限码（对应 `auth_sys_menu.permission`） |
| `#[sa_check_login]` | 仅要求登录（个人中心类接口） |
| `#[sa_ignore]` | 开放端点（登录 / 注册 / 验证码） |

- 免登录端点必须**同时**加进 `src/config.rs` 的排除清单（`#[sa_ignore]` 不参与路径认证判断，否则未登录一律 401）；
  当前白名单：`/api/auth/{login,register,register-roles,locate,refresh-token,providers,captcha,captcha-image/**,send-code,wechat/qr-url}`、`/api/health`；
- 权限码命名 `模块:动作`（`user:add`、`notification:template:edit`），必须与**菜单表 `permission`、前端 `v-permission` 三处一致**；
- 取当前用户一律用 `common::user::get_current_user_id()`（已剥离客户端后缀，见 §9）。

## 6. 统一响应与国际化

```rust
#[derive(Serialize)] #[serde(rename_all = "camelCase")]
pub struct ApiResponse<T> { pub code: i32, pub message: String, pub data: Option<T> }
```

- handler 统一写法：`Ok(match service.x().await { Ok(v) => Json(ApiResponse::success(v)), Err(e) => Json(ApiResponse::error(500, &e.to_string())) })`；
- 状态码语义：业务错误 **HTTP 200 + `code:500`**；未登录 401；无权限 403；两步验证 428（由登录页处理）；
  401/403 由 `system_interface::routes::with_error_handler` 重写为中文 `ApiResponse`，**不要**在业务代码里返回裸状态码；
- 错误类型 `common::error::AppError`：`NotFound/Unauthorized/Forbidden/BadRequest/Internal/DbErr/TwoFactorRequired`；
- **后端消息统一用 `@key`**，由 `with_i18n` 中间件按请求 `Accept-Language` 翻译：

```rust
return Err(AppError::NotFound("@user_not_found".to_string()));
```

新增消息**必须同时**在 `locales/zh-CN.json` 与 `locales/en-US.json` 补 key，否则前端会原样显示 `@user_not_found`。

## 7. 实体与数据库约定

- 通用审计字段：`create_time/create_by/create_id`、`update_time/update_by/update_id`、`version`（乐观锁）、`delete_flag`（软删）、`tenant_id`（租户）；
- **`create_by` 存用户名，`create_id` 存用户 ID**——按用户过滤业务行时必须用 `create_id`（详见 §18）；
- 自动填充：带租户表用 `#[derive(DeriveAutoFillSoftDeleteTenant)]`，无租户表用 `#[derive(DeriveAutoFillSoftDelete)]`；
  字段用 `#[sea_orm_ext(insert)]` / `(update)` / `(insert_update)` + `#[sea_orm_ext(version)]` 标注；
- **软删语义**：`#[soft_delete(default = 0, del = 1)]` 会让宏生成带 `delete_flag = 0` 过滤的 `Entity::find()` / `find_by_id()`；
  需要连已删数据一起查时用 `find_with_deleted()` / `find_by_id_with_deleted()`；
  真正**物理删行**必须用 `Entity::delete_many()` —— `delete_by_id` / `ActiveModel::delete` 会被改写成 `UPDATE delete_flag = 1`；
- 所有实体与 DTO 加 `#[serde(rename_all = "camelCase")]`；
- 时间统一 `DateTimeUtc` + `serde(with = "common::datetime_format")`，出参格式 `YYYY-MM-DD HH:MM:SS`。

## 8. 分页与查询 / 事务

**分页**

- 有分页的接口一律用 `common::pagination::PageQuery`（直接作形参或 `#[serde(flatten)]`）：它把 `page` 钳制 ≥1、`page_size` 钳制到 1..=200；
- 模块自带 `page`/`page_size` 的查询 DTO 必须加钳制，否则 `page=0` 会在 `fetch_page(page - 1)` 下溢 panic：

```rust
#[serde(default = "common::pagination::default_page", deserialize_with = "common::pagination::de_page")]
pub page: u64,
#[serde(default = "common::pagination::default_page_size", deserialize_with = "common::pagination::de_page_size")]
pub page_size: u64,
```

- 返回统一 `PageResult { items, total, page, page_size, total_pages }`；
- **多字段关键字搜索用 OR**（`.filter(name.contains(kw).or(code.contains(kw)))`），不要写成两次 `.filter`（AND 会导致搜不到）；
- 列表里的关联展示名要**批量查一次做映射**，不要 N+1。

**事务**

同一业务动作涉及多次写库时必须包在一个事务里（`let tx = self.db.inner().begin().await?; ... tx.commit().await?;`）：

- 「先清空再插入」的绑定类接口必须**先去重 id**（重复 id 触发唯一约束会让清空后无法回滚，角色会丢光菜单）；
- 事务**覆盖不到的外部副作用**（Redis、SSE、邮件、对象存储）放在 `commit()` **之后**执行；
- table 模式主库与 database 模式租户库**无法共用事务**，跨库流程要么补偿、要么保证提交顺序可恢复。

## 9. 多租户与客户端维度

**多租户**

- 模式由 `TENANT_MODE` 决定：`table`（表内 `tenant_id` 隔离，查询自动注入 `WHERE tenant_id = ?`）/ `database`（每租户独立库）；
- **种子数据必须带 `tenant_id`**：表隔离模式下 `tenant_id IS NULL` 的行对所有租户域查询都不可见，
  且登录 token 的 `tenantId` 取自 `auth_sys_user.tenant_id`，为空会导致整个系统列表全空（真实踩过的坑，见 `m20260929_000004_system_modules.sql` §11）；
- 免登录端点（无租户上下文）读租户域表必须加 `#[ignore_tenant]` 或包在 `TenantIgnoreGuard::new()` 作用域内，
  否则查询会变成 `WHERE tenant_id = NULL` 而恒为空集；
- 跨租户操作（如租户注册表、客户端注册表）同样用上述两者之一显式放行。

**客户端（多端）维度**

客户端是权限体系的顶级维度：客户端 →（其下）菜单/功能权限 → 角色 → 用户角色绑定。

| 对象 | 归属 | 说明 |
|---|---|---|
| `auth_sys_client` | 全局（主库） | `client_code`（OAuth2 client_id，创建后不可改）+ `client_secret` |
| `auth_sys_menu.client_id` | 客户端 | 目录/菜单/按钮都挂在客户端下 |
| `auth_sys_role.client_id` | 客户端 | 角色只能在所属客户端内绑定菜单 |
| `role_code` 唯一性 | `(client_id, role_code)` | 不同客户端可以各有一套 `admin` 角色 |

- 登录可携带 `clientCode` + `clientSecret`：校验通过后只加载该客户端的角色与菜单，非超管在该端无角色直接拒绝登录；
- 会话 `login_id = 用户ID#客户端编码`，同一账号在不同端拥有隔离的权限快照；业务取用户 ID 用 `common::user::get_current_user_id()`；
- 权限变更（`auth/permission_sync.rs`）会刷新该账号的全部在线会话并通过 SSE 通知浏览器重建菜单；
- 前端通过 `VITE_CLIENT_CODE` / `VITE_CLIENT_SECRET` 声明自己的客户端身份。

## 10. 三个横切设施

**附件中心**（`auth_sys_attachment` + `StorageService`）

- 存储驱动 `[attachment.storage].driver`：`local`（本地磁盘）| `rustfs`（S3 兼容，RustFS / MinIO 均可）；
  **该段是嵌套表，summer 的 `get_by_prefix` 不拆分 `.`**，代码按顶层键 `attachment` 整体反序列化（见 `AttachmentSettings`）；
  前缀写错会静默退化为 `driver=local`，文件写进 `./uploads/attachment`——启动日志会打印「附件存储驱动 = …」，未知 driver 直接 panic；
- `[web.middlewares].limit_payload` 决定框架层请求体上限（当前 64MB）；业务上限是 `AttachmentService::MAX_FILE_SIZE`（50MB），
  改体积上限需同步四处：框架 `body_limit`、业务常量、生产 nginx `client_max_body_size`、前端组件的 `maxSize`；
- 业务删除要级联清理附件（`AttachmentService::delete_by_biz(...)`），不要只软删业务行留下孤儿对象；
- 对象存储/删除路径**不要用 `let _ = ...` 吞错**，失败要上抛并记日志。

**通知中心**（`auth_sys_notification` + `auth_sys_notification_template`）

- 站内信 + 邮件双通道；标题/正文由模板渲染（`${varName}` 占位，见 `common::template`）；
- 全局发送器在 `main.rs` 的 `run_startup_tasks()` 里注册（`install_as_global`），业务侧用 `common::notify::{notify, notify_template, send_email_template}` 调用；
- 邮箱验证码依赖模板编码：`email_code_register` / `email_code_reset` / `email_code_login` / `email_code_change_password` / `email_code_change_email`，
  新增场景要同步插模板种子数据，否则发码会失败；
- `[mail].stub = true` 时只打印验证码不真实投递（本地联调）。

**导出中心**（`auth_sys_export_task`）

- 分流：功能页 `GET /system/xxx/export/count` 取条数 → ≤ `common::xlsx::MAX_SYNC_ROWS`（10 万）同步下载（同时在导出中心登记一条 success 记录）
  → 超阈值提交 `POST /api/system/exports` 异步生成，用户在「导出中心」查看与下载；
- 新增一个可异步导出的列表只需在对应 `service.rs` 里写一个行函数 + `fast_excel::export_task! { task_type = "xxx", ... }`，
  链接期自动注册，**不需要**改 `main.rs`、也不需要执行器；
- 当前已注册 task_type：`user` / `role` / `menu` / `dept` / `tenant` / `dict` / `config` / `client`；
- 任务归属按 `create_id` 过滤（`create_by` 是用户名，用它过滤会恒为空集）。

## 11. 后端常见任务

**新增 CRUD 模块**（照抄 `system/dept` 这样的小模块最省事）：

1. `entity/src/xxx.rs` 实体 + `entity/src/lib.rs` 导出；
2. `domain/src/xxx_repository.rs`（需要依赖倒置时）+ `infrastructure/src/persistence/xxx_repository_impl.rs`；
3. `application/src/xxx/{mod.rs,dto.rs,service.rs}`：DTO + `XxxAppService`，异步导出按 §10 注册；
4. `interface/src/handlers/xxx_handler.rs`：`#[nest("/system")] mod controller` + 路由 + 权限宏，并在 `handlers/mod.rs` 声明；
5. `migration/src/` 补建表 + 菜单/按钮权限 + 管理员角色绑定 SQL；
6. `cargo check --workspace` 后跑一次迁移验证。

**新增接口**：在已有 handler 的 `mod controller` 内加函数即可（自动注册）；按 §5 选权限宏，按 §6 用 `@key` 消息。

**改表结构**：直接改 migration SQL（脚本是唯一权威定义），然后 `cargo run -p migration` 重建。
运行器按 `;` 切分语句并先剥离整行 `--` 注释，因此：每个语句必须**幂等**（`IF NOT EXISTS` / `ON CONFLICT` / `DROP ... IF EXISTS`），
语句内**不要出现字符串字面量里的 `;`**。

**新增后台任务**：在 `main.rs` 里 `tokio::spawn`，循环内 `tokio::time::sleep`；依赖组件用 `wait_component_ready::<T>()` 轮询等待，
注意幂等与失败重试（参考 `run_session_index_cleanup`）。

---

# 二、前端（template-web）

## 12. 命令与配置

| 目的 | 命令（在 `template-web/` 下） |
|---|---|
| 开发（:3000，代理 `/api` → 127.0.0.1:8080） | `pnpm dev` |
| **提交前必跑**：类型检查 + 构建 | `pnpm build`（= `vue-tsc -b && vite build`） |
| Lint（自动修复） | `pnpm lint` |
| 预览构建产物 | `pnpm preview` |

⚠️ `vue-tsc` **发现不了模板标签不平衡**（多余/缺失的 `</a-card>`），改过模板就必须 `pnpm build`。

环境变量（`.env.development` / `.env.production`）：

| 变量 | 说明 |
|---|---|
| `VITE_API_BASE_URL` | 请求层 baseURL，默认 `/api`（开发由 Vite 代理，生产由 nginx 反代） |
| `VITE_APP_TITLE` | 站点标题 |
| `VITE_CLIENT_CODE` / `VITE_CLIENT_SECRET` | 当前前端所属客户端（对应「系统管理 → 客户端管理」），登录时提交 |

技术栈：Vue 3.5 + `<script setup lang="ts">` + TypeScript 5.6 / Vite 6 / Ant Design Vue 4 / Pinia / Vue Router 4 / vue-i18n 9 / Axios（`json-bigint` 防雪花 ID 精度丢失）/ Less。

## 13. 请求层 / 路由 / 权限

**`src/api/request.ts` 是唯一 HTTP 出口**（页面里不要直接 `fetch`，导出等 blob 场景除外）：

- 请求拦截：带 `Accept-Language`（后端按此返回对应语言消息）、`Authorization: Bearer <token>`、`x-tenant-id`；
  **开放认证端点**（`/auth/login`、`/auth/captcha`、`/auth/register`、`/auth/register-roles`、`/auth/send-code`、`/auth/locate`、`/auth/providers`、`/auth/refresh-token`、`/auth/wechat/qr-url`）不带残留旧 token；
- 响应拦截：`code !== 200` 弹 `message.error` 并 reject；HTTP 401 走 refresh token 队列（并发挂起、刷新后重放）；
  428（两步验证）不弹全局提示，交由登录页处理；错误对象带 `code` 供调用方分支；
- 返回值已是 `data` 本体，页面直接拿业务对象。

**路由**

- 静态路由（`router/index.ts`）：`/login`、`/login/wechat/callback`、`/profile-setup`、`/`（BasicLayout：`dashboard`、`profile`、`system/export`）、`/403`、`/:pathMatch(.*)*`（404 兜底）；
- 动态路由（`router/dynamic.ts`）：登录后按 `GET /auth/user-info` 的菜单树生成；`dir` 只递归、`menu` 用 `component` 字段（如 `system/user/index`）
  从 `import.meta.glob('@/views/**/*.vue')` 查表懒加载、`button` 不生成路由；
  ⇒ **新增菜单页只需建 `views/<path>/index.vue` + 菜单表配 `path`/`component`，不用改路由文件**；
- 守卫要点：未登录跳 `/login?redirect=`；已登录但菜单未注册 → 拉用户信息 + `setupDynamicRoutes()` 后
  `next({ path, query, hash, replace: true })` 重新解析（**不要展开整个 `to`**，刷新时 `to.name` 可能是兜底 404）；
  `profileCompleted === false` 强制跳 `/profile-setup`；`meta.permission` 不满足且非 admin → `/403`。

**权限**

- 路由级：菜单 `permission` → `meta.permission`；
- 元素级：`v-permission="'user:add'"`（`admin` 角色直通，支持数组任一命中，无权限时**移除元素**）；
- 权限码必须与后端 `#[sa_check_permission]`、菜单表 `permission` 一致。

## 14. 列表页三段式布局（重要）

所有列表页结构必须一致：**搜索条件卡 + 表格卡**，两张卡平铺在根 `<div>`，表格卡加 `class="mt-3"`（12px）。

**路径 A：`ProTable`（推荐，搜索卡/表格卡/高度自适应全内置）**

```vue
<template>
  <ProTable
    ref="tableRef"
    :columns="columns"
    :search-fields="searchFields"
    :api-fn="apiFn"
    :scroll="{ x: 1500 }"
  >
    <template #toolbar>
      <a-button v-permission="'user:add'" type="primary" @click="openModal()">新增</a-button>
    </template>
    <template #bodyCell="{ column, record }">...</template>
  </ProTable>
  <!-- 弹窗放根 div 之外，作为多根模板 -->
</template>
```

- props：`columns` / `searchFields` / `apiFn` / `rowKey`（默认 `id`）/ `scroll` / `rowSelection` / `pageSize`（默认 20）/
  `showIndex`（默认 true，首列跨页连续序号）/ `immediate` / `noFit` / `searchDefaults`；
- expose：`fetchData` / `refresh` / `resetSearch` / `getCurrentPage` / `tableData` / `searchParams`；
- `searchFields` 支持 `input` / `password` / `textarea` / `number` / `select` / `dictSelect` / `userPicker` / `datePicker` / `timeRange` / `rangePicker` / `switch` / `radio` / `checkbox` / `custom`；
- `timeRange` 自动拆成 `xxxStart` / `xxxEnd` 两个参数（字段 `createTime` → `createTimeStart/createTimeEnd`），后端 DTO 必须有对应字段；
- `searchDefaults` 是默认查询条件（「重置」会回到它），传入新对象会自动重查。

**路径 B：手写 `<a-table>` + `useFitTableHeight()`**（行内复杂操作/列定制时用，参考 `views/system/user/index.vue`）：
`const { fitRef, fitY, recompute } = useFitTableHeight()`，`fitRef` 绑在**表格卡根节点**，`:scroll="{ x, y: fitY }"`；
弹窗开合等引起高度变化的场景手动调 `recompute()`。

**必须遵守**

- 不要用外层 `a-card` 把搜索卡与表格卡再包一层（卡中卡）；
- 新增/导出按钮放表格卡 `#extra`（ProTable 用 `#toolbar`），不要与搜索表单挤在同一行；
- 搜索条件不要塞进表格卡内部或 `#extra`；
- 间距用 `global.less` 的工具类（`mt-3` / `mt-16` / `mb-16` / `flex-between`…），不要自定义；
- 弹窗放根 `<div>` 外侧。

## 15. ProForm、国际化与样式

**ProForm**

```vue
<ProForm v-model:model-value="form" :fields="formFields" :label-col="{ span: 6 }" :wrapper-col="{ span: 18 }" />
```

- props：`modelValue` / `fields` / `rules` / `labelCol` / `wrapperCol` / `layout` / `gutter` / `span`；
- `FormField`：`field` / `label` / `type` / `placeholder` / `rules` / `span` / `disabled` / `options` / `dictCode` / `props` / `slotName` / `displayField`；
- **控件专属属性放 `props` 里**（`props: { showSearch: true }`），写在字段顶层不会透传；
- 自定义控件用 `slotName` + `<template #slotName="{ model, field }">`。

**国际化**：通用文案走 `t('common.search')` 这类 key，业务文案可直接写中文（与后端中文提示一致）；
新增 key 要**同时**改 `src/locales/zh-CN.ts` 与 `en-US.ts`。

**样式**：优先用 `global.less` 工具类；页面样式用 `<style scoped lang="less">`，不要写全局选择器覆盖 antd。
主题色一律走 `--brand-*` 令牌（`--brand-primary` / `--brand-gradient` / `--sider-*` 等），**不要写死十六进制色值**。

**表格单行省略**：长文本列统一用 `a-tooltip` + 全局类 `cell-ellipsis`，空值显示 `-` 且不弹提示；新增列要同步调大 `:scroll="{ x }"`。

## 16. 前端常见任务

**新增菜单页（4 步）**：① `src/api/xxx.ts` 定义接口 → ② 建 `src/views/<path>/index.vue`（三段式）→
③「菜单管理」新增菜单，配 `path` + `component`（如 `system/xxx/index`）+ `permission` → ④ 按钮权限同步 `v-permission` 与菜单表，最后 `pnpm build`。

**新增无菜单的二级页**：在 `router/index.ts` 的 Layout children 加静态路由（参考 `system/export`、`login/wechat/callback`）。

**做深链过滤**（从详情跳列表并预置条件）：目标页读 `useRoute().query.xxx`，拼进 `searchDefaults` 的 `computed`。

**新增业务组件**：优先放 `src/components/`（公共）或页面同级 `components/`（局部）；带副作用的（上传、权限）要写清 props/emits 与卸载清理。

---

# 三、跨端陷阱清单（Do / Don't）

**后端**

- ✅ 提交前必跑 `cargo check --workspace`；改 `common/` 或分页相关再跑 `cargo test -p common`。
- ❌ 不要把 `/api` 写进 `#[nest]`；❌ 不要回退到手工 `routes()` + `.nest()`；❌ 不要用 `#[auto_config]`。
- ❌ 不要删 `extern crate sea_orm_ext as summer_sea_orm_ext;`；❌ 不要漏 `#[serde(rename_all = "camelCase")]`。
- ❌ 不要新增 `@key` 消息却忘记同步 `locales/*.json`（前端会显示 `@xxx`）。
- ❌ 不要在表隔离模式下插入不带 `tenant_id` 的数据（列表会整体查不到）。
- ❌ 不要绕过 `PageQuery` / 分页钳制；❌ 不要写「多字段 AND 关键字」搜索。
- ❌ 不要在同一业务动作里裸写多次写库；❌ 不要在事务内做 Redis/HTTP/对象存储等外部副作用。
- ❌ 不要用 `Entity::delete_by_id` / `ActiveModel::delete` 表达物理删除；真正删行用 `Entity::delete_many()`。
- ❌ 不要在按用户过滤业务行时用 `create_by`（存的是用户名，应使用 `create_id`）。
- ❌ 不要在声明 `#[inject(config)]` 的结构体上写**带点的 config_prefix**（如 `"attachment.storage"`）：嵌套表必须用顶层键整体反序列化。
- ❌ 不要在对象存储/删除路径写 `let _ = ...` 吞掉错误；❌ 不要在业务删除时只软删业务行而不级联清理附件。
- ❌ 不要在 migration SQL 里依赖 `;` 出现在字符串里；❌ 迁移脚本每个语句都要幂等。
- ❌ 不要在生产使用默认 `JWT_SECRET_KEY`、默认客户端密钥或明文 admin 密码。
- ❌ 不要从少位数假定雪花 ID 宽度（如写死 `len() == 19`），位数随生成器纪元变化。

**前端**

- ❌ 改完模板不跑 `pnpm build`（`vue-tsc` 查不出标签不平衡）。
- ❌ 给 `ProTable` / 自定义组件传**未声明的 prop**（会被静默忽略）。
- ❌ 搜索字段名与后端 Query DTO 不一致（尤其 `timeRange` 的 `xxxStart/xxxEnd`）。
- ❌ 用 `a-card` 套两层、把按钮和搜索表单放同一行、把搜索条件塞进表格卡。
- ❌ 在页面里直接 `fetch`（除导出等 blob 场景）；❌ 绕过 `request.ts` 直接 `axios`（会丢 token/租户头与统一错误提示）。
- ❌ 忘记把弹窗放到根 `<div>` 外，或 `<template>` 里残留 `</a-card>`。
- ❌ 用 `Number` 解析接口里的大整数（雪花 ID 会丢精度，`request.ts` 已用 `json-bigint` 处理）。
- ❌ 直接改 `dist/`（构建产物）；❌ 提交 `node_modules`。