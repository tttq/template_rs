# Database 模式多租户技术架构与设计文档

> **项目**: template-admin (后端) + template-web (前端)
> **文档版本**: v1.1
> **最后更新**: 2026-05-21

---

## 1. 概述

本文档描述 `template-admin` 在 **Database 隔离模式** 下的多租户完整实现方案，涵盖租户生命周期管理、登录认证、权限校验、数据库动态切换等核心流程。

### 1.1 两种隔离模式对比

| 维度 | Table 模式（当前默认） | Database 模式（本文重点） |
|------|----------------------|--------------------------|
| 隔离粒度 | 同库同表，`tenant_id` 字段区分 | 每租户独立数据库，物理隔离 |
| 数据安全 | 逻辑隔离，依赖代码过滤 | 物理隔离，天然防串号 |
| 连接管理 | 共享一个 `DbConn` | 每租户独立 `DatabaseConnection` |
| 扩展性 | 受单库性能上限 | 可按租户独立扩容 |
| 运维复杂度 | 低 | 高（需管理多库迁移/备份） |
| 适用场景 | 租户数量多、数据量小 | 租户数量少、数据安全要求高 |

### 1.2 核心挑战

Database 模式下需解决以下关键问题：

1. **租户识别**：登录前如何确定用户属于哪个租户？
2. **动态数据源**：请求到达后如何路由到正确的租户数据库？
3. **租户生命周期**：新增租户时如何自动创建数据库并初始化 Schema？
4. **权限隔离**：不同租户的角色/权限如何独立管理？
5. **连接池管理**：如何高效管理多个租户的数据库连接？

---

## 2. 架构设计

### 2.1 整体架构

```
┌─────────────────────────────────────────────────────────────────┐
│                         HTTP Request                            │
│                    (携带 x-tenant-id 或 Token)                   │
└────────────────────────────┬────────────────────────────────────┘
                             │
                    ┌────────▼────────┐
                    │  TenantLayer    │  Axum 中间件
                    │  (sea-orm-ext)  │  解析租户ID → 设置上下文
                    └────────┬────────┘
                             │
                    ┌────────▼────────┐
                    │  SaTokenLayer   │  认证中间件
                    │  (summer-sa)    │  解析 Token → 注入登录信息
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────┐
              │              │              │
     ┌────────▼──────┐      │      ┌───────▼────────┐
     │  主库 (Master) │      │      │ 租户库 (Tenant) │
     │               │      │      │                │
     │ auth_sys_     │      │      │ auth_sys_      │
     │   tenant      │      │      │   user         │
     │               │      │      │   role         │
     │ (租户注册表)   │      │      │   menu         │
     └───────────────┘      │      │   user_role    │
                            │      │   role_menu    │
                            │      │   ...          │
                            │      └────────────────┘
                            │
                    ┌───────▼────────┐
                    │  Handler 层    │
                    │  通过 TenantDb │
                    │  获取租户连接  │
                    └───────┬────────┘
                            │
                    ┌───────▼────────┐
                    │  Service 层    │
                    │  tenant_db()   │
                    │  自动路由      │
                    └────────────────┘
```

### 2.2 双库架构

Database 模式采用 **主库 + 租户库** 的双库架构：

| 数据库 | 存放内容 | 连接来源 |
|--------|---------|---------|
| **主库 (Master)** | `auth_sys_tenant`（租户注册表） | `Component(DbConn)` — 全局共享 |
| **租户库 (Tenant-N)** | `auth_sys_user/role/menu/user_role/role_menu/dept/dict_type/dict_item/config` | `TenantDb` 提取器 — 按租户 ID 动态获取 |

**关键原则**：主库只存租户元信息，所有业务数据在租户库中。租户库之间完全物理隔离。

### 2.3 请求处理流程

```
1. 请求到达 → TenantMiddleware
   ├── 从 SaTokenContext 提取 tenantId（JWT extra_data）
   ├── 或从 x-tenant-id 请求头提取
   ├── set_tenant_context(TenantContext { tenant_id })
   ├── 若 Database 模式 → get_tenant_database() 获取连接
   │   └── 注入到 request.extensions_mut()
   └── 请求继续传递

2. Handler 处理
   ├── 需要租户库 → TenantDb(db): TenantDb 提取器
   ├── 需要主库   → Component(db): Component<DbConn>
   └── 两者可同时使用

3. 请求结束 → TenantMiddleware::call 返回
   └── clear_tenant_context() 清除上下文
```

---

## 3. 租户生命周期管理

### 3.1 新增租户流程

```
管理员在租户管理界面点击"新增租户"
         │
         ▼
┌─────────────────────┐
│ 1. 填写租户信息      │
│   - tenantName      │
│   - tenantCode      │
│   - mode = database │
│   - databaseType    │
│   - databaseUrl     │
│   - databaseName    │
│   - 管理员账号/密码  │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ 2. 测试连接          │
│   POST /tenants/     │
│     test-connection  │
│   验证数据库服务器    │
│   是否可达           │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ 3. 创建数据库        │
│   POST /tenants/     │
│     create-database  │
│   CREATE DATABASE    │
│   "tenant_xxx"       │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ 4. 初始化 Schema     │
│   POST /tenants/     │
│     init-database    │
│   执行 init_schema   │
│   .sql 建表          │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ 5. 初始化管理员      │
│   在租户库中插入     │
│   默认管理员账号      │
│   + 默认角色 + 菜单  │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ 6. 注册租户记录      │
│   在主库插入          │
│   auth_sys_tenant    │
└────────┬────────────┘
         │
         ▼
┌─────────────────────┐
│ 7. 注册数据库连接    │
│   连接租户库并加入    │
│   ConnectionStore    │
└─────────────────────┘
```

### 3.2 新增租户的一体化 API

当前已有分步 API（test-connection → create-database → init-database），建议新增一体化接口：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTenantFullDto {
    pub tenant_name: String,
    pub tenant_code: String,
    pub database_type: String,
    pub database_url: String,
    pub database_name: String,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub contact_email: Option<String>,
    pub expire_time: Option<DateTime<Utc>>,
    pub remark: Option<String>,
    pub admin_user_name: String,
    pub admin_pass_word: String,
    pub admin_nick_name: Option<String>,
}
```

Service 层伪代码：

```rust
pub async fn create_tenant_full(&self, dto: CreateTenantFullDto) -> Result<TenantVo, AppError> {
    // 1. 检查 tenant_code 唯一性（主库）
    let existing = tenant::Entity::find()
        .filter(tenant::Column::TenantCode.eq(&dto.tenant_code))
        .one(&self.db)
        .await?;
    if existing.is_some() {
        return Err(AppError::BadRequest("租户编码已存在".to_string()));
    }

    // 2. 测试数据库连接
    let db_url = build_database_url(&dto.database_type, &dto.database_url, None);
    let conn = Database::connect(&db_url).await
        .map_err(|e| AppError::BadRequest(format!("连接数据库服务器失败: {}", e)))?;
    conn.ping().await
        .map_err(|e| AppError::BadRequest(format!("连接测试失败: {}", e)))?;

    // 3. 创建数据库
    let backend = get_database_backend(&dto.database_type);
    let sql = format!("CREATE DATABASE \"{}\"", dto.database_name);
    conn.execute_unprepared(&sql).await
        .map_err(|e| AppError::BadRequest(format!("创建数据库失败: {}", e)))?;

    // 4. 连接新库并初始化 Schema
    let tenant_db_url = build_database_url(&dto.database_type, &dto.database_url, Some(&dto.database_name));
    let tenant_conn = Database::connect(&tenant_db_url).await
        .map_err(|e| AppError::BadRequest(format!("连接租户数据库失败: {}", e)))?;

    for statement in INIT_SQL.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            tenant_conn.execute_unprepared(trimmed).await
                .map_err(|e| AppError::BadRequest(format!("初始化Schema失败: {}", e)))?;
        }
    }

    // 5. 初始化管理员 + 默认角色 + 菜单
    self.init_tenant_admin(&tenant_conn, &dto).await?;

    // 6. 在主库注册租户记录
    let tenant_dto = CreateTenantDto {
        tenant_name: dto.tenant_name.clone(),
        tenant_code: dto.tenant_code.clone(),
        mode: "database".to_string(),
        database_type: Some(dto.database_type.clone()),
        database_url: Some(dto.database_url.clone()),
        database_name: Some(dto.database_name.clone()),
        status: Some(1),
        contact_name: dto.contact_name.clone(),
        contact_phone: dto.contact_phone.clone(),
        contact_email: dto.contact_email.clone(),
        expire_time: dto.expire_time.clone(),
        remark: dto.remark.clone(),
    };
    let tenant_model = tenant_dto.into_active_model().insert(&self.db).await?;

    // 7. 注册数据库连接到 ConnectionStore
    let tenant_id_value = Value::String(Some(tenant_model.id.clone()));
    let store = sea_orm_ext::get_tenant_store()
        .ok_or_else(|| AppError::Internal("租户连接存储未初始化".to_string()))?;
    store.insert(tenant_id_value, tenant_conn)?;

    Ok(tenant_model.into())
}
```

### 3.3 租户初始化数据

新增租户时，需在租户库中初始化以下数据：

```sql
-- 1. 默认管理员角色
INSERT INTO auth_sys_role (id, role_name, role_code, role_sort, status)
VALUES ('1', '管理员', 'admin', 1, 1);

-- 2. 管理员用户
INSERT INTO auth_sys_user (id, user_name, nick_name, pass_word, status, admin_flag)
VALUES ('1', '{admin_user_name}', '{admin_nick_name}', '{admin_pass_word}', 1, 1);

-- 3. 用户-角色关联
INSERT INTO auth_sys_user_role (id, user_id, role_id)
VALUES ('1', '1', '1');

-- 4. 默认菜单（根据业务需要插入）
-- ... 省略具体菜单数据
```

建议将初始化数据模板化，放在 `init_data.sql` 中，通过参数替换生成最终 SQL。

### 3.4 租户停用/启用

```rust
pub async fn toggle_tenant_status(&self, id: String, status: i32) -> Result<(), AppError> {
    let model = tenant::Entity::find()
        .filter(tenant::Column::Id.eq(&id))
        .one(&self.db)
        .await?
        .ok_or_else(|| AppError::NotFound("租户不存在".to_string()))?;

    let mut am: tenant::ActiveModel = model.into();
    am.status = Set(status);
    am.update(&self.db).await?;

    if status == 0 {
        // 停用时从 ConnectionStore 移除连接
        if let Some(store) = sea_orm_ext::get_tenant_store() {
            let _ = store.remove(&Value::String(Some(id)));
        }
    } else {
        // 启用时重新注册连接
        self.register_tenant_connection(&id).await?;
    }

    Ok(())
}
```

---

## 4. 登录认证设计（方案 C + E）

### 4.1 设计目标

- **用户无感**：用户只需输入用户名/密码，无需感知租户概念
- **信息安全**：不暴露租户列表，防止租户枚举攻击
- **可扩展**：预留第三方登录（微信、钉钉、企业微信等）扩展点
- **数据同步**：租户库用户变更时自动同步主库索引

### 4.2 核心架构

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              前端登录流程                                     │
│                                                                             │
│   ┌─────────────┐      ┌─────────────┐      ┌─────────────┐              │
│   │  步骤 1/2   │ ───▶ │  步骤 2/2   │ ───▶ │  登录成功   │              │
│   │  输入用户名  │      │  输入密码    │      │  跳转主页   │              │
│   └──────┬──────┘      └──────┬──────┘      └─────────────┘              │
│          │                    │                                             │
│          ▼                    ▼                                             │
│   POST /auth/locate     POST /auth/login                                   │
│   { userName }          { userName, passWord, tenantCode }                │
│          │                    │                                             │
└──────────┼────────────────────┼─────────────────────────────────────────────┘
           │                    │
           ▼                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                              后端处理流程                                     │
│                                                                             │
│   locate (定位)                     login (登录)                            │
│   ─────────────                     ─────────────                           │
│   ┌──────────────┐                  ┌──────────────┐                      │
│   │  主库查询     │                  │  主库查询     │                      │
│   │ tenant_user  │                  │ tenant_user  │                      │
│   │  索引表       │                  │   索引表      │                      │
│   └──────┬───────┘                  └──────┬───────┘                      │
│          │                                │                                 │
│          ▼                                ▼                                 │
│   ┌──────────────┐                  ┌──────────────┐                      │
│   │  主库查询     │                  │ 主库查询租户   │                      │
│   │  tenant表    │                  │  验证状态     │                      │
│   │  获取租户名   │                  └──────┬───────┘                      │
│   └──────────────┘                         │                               │
│                                           ▼                                │
│                                  ┌──────────────┐                         │
│                                  │ Connection   │                         │
│                                  │ Store        │                         │
│                                  │ 获取租户库连接 │                         │
│                                  └──────┬───────┘                         │
│                                         │                                  │
│                                         ▼                                  │
│                                  ┌──────────────┐                         │
│                                  │  租户库查询   │                         │
│                                  │  验证密码    │                         │
│                                  │  查询权限    │                         │
│                                  └──────┬───────┘                         │
│                                         │                                  │
│                                         ▼                                  │
│                                  ┌──────────────┐                         │
│                                  │  SaToken     │                         │
│                                  │  写入Redis   │                         │
│                                  │  返回Token   │                         │
│                                  └──────────────┘                         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

### 4.3 数据库设计

#### 4.3.1 主库用户索引表（核心）

```sql
-- 主库：用户-租户索引表（方案 C + E 的核心基础设施）
CREATE TABLE IF NOT EXISTS auth_sys_tenant_user (
    id VARCHAR(64) PRIMARY KEY,
    user_name VARCHAR(100) NOT NULL,          -- 用户名（唯一，全局唯一）
    tenant_id VARCHAR(64) NOT NULL,           -- 所属租户ID
    tenant_code VARCHAR(50) NOT NULL,         -- 所属租户编码
    user_id VARCHAR(64) NOT NULL,            -- 租户库中用户ID
    identity_type VARCHAR(50) DEFAULT 'username',  -- 认证类型: username/email/phone/wechat/dingtalk
    identity_value VARCHAR(200),              -- 认证标识值: 邮箱/手机/微信openid等
    status INT NOT NULL DEFAULT 1,             -- 1:正常 0:禁用
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_time TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 用户名唯一索引（全局唯一，用于用户名登录）
CREATE UNIQUE INDEX idx_tenant_user_name ON auth_sys_tenant_user(user_name);

-- 租户ID索引（按租户查询用户列表）
CREATE INDEX idx_tenant_user_tenant ON auth_sys_tenant_user(tenant_id);

-- 认证类型+标识值索引（用于第三方登录）
CREATE INDEX idx_tenant_user_identity ON auth_sys_tenant_user(identity_type, identity_value);
```

**设计要点**：
- `user_name` 全局唯一，作为用户名登录的主键
- `identity_type + identity_value` 支持多类型登录（用户名/邮箱/微信等）
- `tenant_id + user_id` 组合指向租户库中的实际用户记录
- 租户库中用户 CRUD 时，必须同步维护此索引表

#### 4.3.2 租户用户表扩展

```sql
-- 租户库 auth_sys_user 表增加认证相关字段
ALTER TABLE auth_sys_user ADD COLUMN IF NOT EXISTS identity_type VARCHAR(50) DEFAULT 'username';
ALTER TABLE auth_sys_user ADD COLUMN IF NOT EXISTS identity_value VARCHAR(200);
ALTER TABLE auth_sys_user ADD COLUMN IF NOT EXISTS third_party_id VARCHAR(200);  -- 第三方唯一标识
```

### 4.4 接口设计

#### 4.4.1 登录前置接口：定位租户

```
POST /auth/locate
Content-Type: application/json

{
  "userName": "admin"
}
```

**请求**：
| 字段 | 类型 | 必填 | 说明 |
|------|------|:----:|------|
| userName | String | 是 | 用户名/邮箱/手机号 |

**响应（成功）**：
```json
{
  "code": 200,
  "message": "success",
  "data": {
    "tenantName": "ACME 公司",
    "tenantCode": "acme",
    "tenantLogo": "https://cdn.example.com/logo-acme.png",
    "userName": "admin",
    "loginType": "password"  // password | wechat | dingtalk
  }
}
```

**响应（失败）**：
```json
{
  "code": 401,
  "message": "用户名或密码错误"  // 统一错误信息，不区分是否存在
}
```

**设计说明**：
- 用户名不存在时，也返回相同错误，防止用户名枚举
- 仅返回租户名称用于确认，不返回 tenantId 等内部信息
- 可扩展返回 loginType，告知前端可用的登录方式

#### 4.4.2 登录接口

```
POST /auth/login
Content-Type: application/json

{
  "userName": "admin",
  "passWord": "123456",
  "tenantCode": "acme",
  "loginType": "password"
}
```

**请求**：
| 字段 | 类型 | 必填 | 说明 |
|------|------|:----:|------|
| userName | String | 是 | 用户名 |
| passWord | String | 条件 | 密码（password登录时必填） |
| tenantCode | String | 是 | 租户编码 |
| loginType | String | 是 | 登录类型：password / wechat / dingtalk |
| code | String | 条件 | 微信授权码（wechat登录时必填） |

**响应**：
```json
{
  "code": 200,
  "message": "success",
  "data": {
    "token": "xxxxx",
    "tokenName": "Authorization",
    "tokenPrefix": "Bearer ",
    "expireTime": 1747824000
  }
}
```

#### 4.4.3 第三方登录接口

```
POST /auth/third-party/callback
Content-Type: application/json

{
  "provider": "wechat",
  "code": "微信授权码",
  "state": "租户编码或随机数"
}
```

**说明**：第三方登录需先跳转到第三方授权，授权后回调本系统。

### 4.5 后端实现

#### 4.5.1 登录服务分层架构

```
AuthAppService
├── locate_tenant()         # 定位租户（前置接口）
├── login()                 # 统一登录入口
├── login_password()        # 密码登录
├── login_third_party()     # 第三方登录
├── register()              # 用户注册
└── get_user_info()         # 获取用户信息
```

#### 4.5.2 核心实现

```rust
// ===== AuthAppService =====

impl AuthAppService {
    /// 步骤1: 定位租户（前置接口）
    pub async fn locate_tenant(&self, dto: LocateTenantDto) -> Result<LocateTenantVo, AppError> {
        // 1. 从主库索引表查找用户
        let index = tenant_user::Entity::find()
            .filter(tenant_user::Column::UserName.eq(&dto.user_name))
            .filter(tenant_user::Column::Status.eq(1))
            .one(&self.db)
            .await?;

        // 2. 用户不存在时，假装成功（防止枚举）
        let index = match index {
            Some(i) => i,
            None => {
                // 模拟查询延迟，防止时序攻击
                tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
            }
        };

        // 3. 查询租户信息
        let tenant = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(&index.tenant_id))
            .filter(tenant::Column::Status.eq(1))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

        // 4. 检查租户过期
        if let Some(expire) = tenant.expire_time {
            if expire < chrono::Utc::now() {
                return Err(AppError::Unauthorized("租户已过期".to_string()));
            }
        }

        Ok(LocateTenantVo {
            tenant_name: tenant.tenant_name,
            tenant_code: tenant.tenant_code,
            tenant_logo: tenant.logo_url,
            user_name: index.user_name,
            login_type: Some(index.identity_type.clone()),
        })
    }

    /// 步骤2: 统一登录入口
    pub async fn login(&self, dto: LoginDto) -> Result<TokenVo, AppError> {
        let login_type = dto.login_type.as_deref().unwrap_or("password");

        match login_type {
            "password" => self.login_password(dto).await,
            "wechat" => self.login_third_party("wechat", dto.code.as_deref(), &dto.state).await,
            "dingtalk" => self.login_third_party("dingtalk", dto.code.as_deref(), &dto.state).await,
            _ => Err(AppError::BadRequest("不支持的登录方式".to_string())),
        }
    }

    /// 密码登录
    async fn login_password(&self, dto: LoginDto) -> Result<TokenVo, AppError> {
        let user_name = dto.user_name.as_deref()
            .ok_or_else(|| AppError::BadRequest("用户名不能为空".to_string()))?;
        let tenant_code = dto.tenant_code.as_deref()
            .ok_or_else(|| AppError::BadRequest("租户编码不能为空".to_string()))?;

        // 1. 查询租户
        let tenant = tenant::Entity::find()
            .filter(tenant::Column::TenantCode.eq(tenant_code))
            .filter(tenant::Column::Status.eq(1))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

        // 2. 检查租户过期
        if let Some(expire) = tenant.expire_time {
            if expire < chrono::Utc::now() {
                return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
            }
        }

        // 3. 获取租户库连接
        let tenant_db = self.get_tenant_database(&tenant.id).await?;

        // 4. 验证用户（租户库）
        let user_model = user::Entity::find()
            .filter(user::Column::UserName.eq(user_name))
            .one(&tenant_db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

        // 5. 验证密码
        if user_model.pass_word != dto.pass_word {
            return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
        }

        // 6. 验证账号状态
        if user_model.status != 1 {
            return Err(AppError::Unauthorized("账号已被禁用".to_string()));
        }

        // 7. 查询权限并登录
        self.do_login(&tenant, &user_model, &tenant_db).await
    }

    /// 第三方登录
    async fn login_third_party(
        &self,
        provider: &str,
        code: Option<&str>,
        state: &Option<String>,
    ) -> Result<TokenVo, AppError> {
        let code = code.ok_or_else(|| AppError::BadRequest("授权码不能为空".to_string()))?;
        let tenant_code = state.as_deref().unwrap_or("");

        // 1. 根据 provider 获取第三方 openid
        let openid = self.get_third_party_openid(provider, code).await?;

        // 2. 从主库索引表查找用户
        let index = tenant_user::Entity::find()
            .filter(tenant_user::Column::IdentityType.eq(provider))
            .filter(tenant_user::Column::IdentityValue.eq(&openid))
            .filter(tenant_user::Column::Status.eq(1))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("该账号未绑定".to_string()))?;

        // 3. 查询租户
        let tenant = tenant::Entity::find()
            .filter(tenant::Column::Id.eq(&index.tenant_id))
            .filter(tenant::Column::Status.eq(1))
            .one(&self.db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

        // 4. 如果有租户编码参数，验证一致性
        if !tenant_code.is_empty() && tenant.tenant_code != tenant_code {
            return Err(AppError::Unauthorized("用户名或密码错误".to_string()));
        }

        // 5. 获取租户库连接
        let tenant_db = self.get_tenant_database(&tenant.id).await?;

        // 6. 查询用户
        let user_model = user::Entity::find()
            .filter(user::Column::Id.eq(&index.user_id))
            .one(&tenant_db)
            .await?
            .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

        if user_model.status != 1 {
            return Err(AppError::Unauthorized("账号已被禁用".to_string()));
        }

        // 7. 查询权限并登录
        self.do_login(&tenant, &user_model, &tenant_db).await
    }

    /// 执行登录（公共逻辑）
    async fn do_login(
        &self,
        tenant: &tenant::Model,
        user_model: &user::Model,
        tenant_db: &DatabaseConnection,
    ) -> Result<TokenVo, AppError> {
        // 1. 查询角色和权限
        let (role_codes, permissions) = self.load_user_permissions(tenant_db, &user_model.id).await?;

        // 2. 生成 Token extra_data
        let user_id_str = user_model.id.to_string();
        let extra = serde_json::json!({
            "userId": user_id_str,
            "userName": user_model.user_name,
            "nickName": user_model.nick_name,
            "tenantId": tenant.id,
            "tenantCode": tenant.tenant_code,
            "tenantMode": "database",
        });

        // 3. 写入 SaToken
        let token_value = StpUtil::login_with_extra(&user_id_str, extra.clone()).await
            .map_err(|e| AppError::Internal(format!("登录失败: {}", e)))?;

        StpUtil::set_roles(&user_id_str, role_codes).await.ok();
        StpUtil::set_permissions(&user_id_str, permissions).await.ok();

        // 4. 返回 Token
        let token = token_value.as_str().to_string();
        let token_info = StpUtil::get_token_info(&token_value).await
            .map_err(|e| AppError::Internal(format!("获取令牌信息失败: {}", e)))?;
        let expire_time = token_info.expire_time.map(|t| t.timestamp());

        Ok(TokenVo {
            token,
            token_name: self.sa_token_config.token_name.clone(),
            token_prefix: self.sa_token_config.token_prefix.clone().unwrap_or_default(),
            refresh_token: None,
            expire_time,
            refresh_expire_time: None,
        })
    }

    /// 获取租户数据库连接
    async fn get_tenant_database(&self, tenant_id: &str) -> Result<DatabaseConnection, AppError> {
        let tenant_id_value = Value::String(Some(tenant_id.to_string()));
        sea_orm_ext::get_database_for_tenant(&tenant_id_value)?
            .ok_or_else(|| AppError::Internal("租户数据库连接未找到".to_string()))
    }

    /// 加载用户权限
    async fn load_user_permissions(
        &self,
        db: &DatabaseConnection,
        user_id: &str,
    ) -> Result<(Vec<String>, Vec<String>), AppError> {
        // 查询用户角色
        let user_roles = user_role::Entity::find()
            .filter(user_role::Column::UserId.eq(user_id))
            .all(db)
            .await?;

        let role_ids: Vec<String> = user_roles.iter().map(|r| r.role_id.clone()).collect();

        // 查询角色信息
        let roles = role::Entity::find()
            .filter(role::Column::Id.is_in(role_ids.clone()))
            .all(db)
            .await?;

        let role_codes: Vec<String> = roles.iter().map(|r| r.role_code.clone()).collect();

        // 查询角色菜单
        let role_menus = role_menu::Entity::find()
            .filter(role_menu::Column::RoleId.is_in(role_ids))
            .all(db)
            .await?;

        let menu_ids: Vec<String> = role_menus.iter().map(|rm| rm.menu_id.clone()).collect();

        // 查询菜单权限
        let menus = menu::Entity::find()
            .filter(menu::Column::Id.is_in(menu_ids))
            .filter(menu::Column::Status.eq(1))
            .all(db)
            .await?;

        let permissions: Vec<String> = menus.iter()
            .filter_map(|m| m.permission.clone())
            .collect();

        Ok((role_codes, permissions))
    }
}
```

### 4.6 用户索引同步机制

#### 4.6.1 同步策略

```
┌─────────────────────────────────────────────────────────────────┐
│                    用户操作与索引同步流程                          │
│                                                                 │
│  ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐ │
│  │ 用户注册  │───▶│ 用户登录  │───▶│ 用户修改  │───▶│ 用户删除  │ │
│  └────┬─────┘    └────┬─────┘    └────┬─────┘    └────┬─────┘ │
│       │                │                │                │      │
│       ▼                │                ▼                ▼      │
│  ┌──────────┐         │         ┌──────────┐    ┌──────────┐  │
│  │ 租户库    │         │         │ 租户库    │    │ 租户库    │  │
│  │ 创建用户  │         │         │ 更新用户  │    │ 删除用户  │  │
│  └────┬─────┘         │         └────┬─────┘    └────┬─────┘  │
│       │                │                │                │      │
│       ▼                │                ▼                ▼      │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              同步主库 tenant_user 索引表                │   │
│  │                                                         │   │
│  │  INSERT ──────────── UPDATE ─────────── DELETE          │   │
│  │                                                         │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 4.6.2 同步实现

```rust
/// 租户用户服务 - 封装用户CRUD与索引同步
pub struct TenantUserService {
    #[inject(component)]
    master_db: DbConn,  // 主库连接
}

impl TenantUserService {
    /// 创建用户（租户库 + 同步主库索引）
    pub async fn create_user(
        &self,
        tenant_db: &DatabaseConnection,
        tenant_id: &str,
        tenant_code: &str,
        dto: CreateUserDto,
    ) -> Result<user::Model, AppError> {
        // 1. 在租户库创建用户
        let user_model = user::ActiveModel {
            user_name: Set(dto.user_name.clone()),
            pass_word: Set(dto.pass_word),
            nick_name: Set(dto.nick_name),
            email: Set(dto.email.clone()),
            phone: Set(dto.phone.clone()),
            identity_type: Set("username".to_string()),
            identity_value: Set(dto.email.or(dto.phone)),
            status: Set(1),
            admin_flag: Set(0),
            dept_id: Set(None),
            ..Default::default()
        };
        let result = user_model.insert(tenant_db).await?;

        // 2. 同步主库索引
        self.sync_to_master_index(
            tenant_id,
            tenant_code,
            &result.id,
            &result.user_name,
            "username",
            result.email.as_deref(),
        ).await?;

        Ok(result)
    }

    /// 删除用户（租户库 + 同步主库索引）
    pub async fn delete_user(
        &self,
        tenant_db: &DatabaseConnection,
        user_name: &str,
    ) -> Result<(), AppError> {
        // 1. 从租户库删除用户
        let user = user::Entity::find()
            .filter(user::Column::UserName.eq(user_name))
            .one(tenant_db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        user.delete(tenant_db).await?;

        // 2. 从主库索引表删除记录
        tenant_user::Entity::delete_many()
            .filter(tenant_user::Column::UserName.eq(user_name))
            .exec(&self.master_db)
            .await?;

        Ok(())
    }

    /// 绑定第三方账号
    pub async fn bind_third_party(
        &self,
        tenant_db: &DatabaseConnection,
        master_db: &DbConn,
        tenant_id: &str,
        tenant_code: &str,
        user_id: &str,
        user_name: &str,
        provider: &str,
        openid: &str,
    ) -> Result<(), AppError> {
        // 1. 更新租户库用户
        let user = user::Entity::find_by_id(user_id.to_string())
            .one(tenant_db)
            .await?
            .ok_or_else(|| AppError::NotFound("用户不存在".to_string()))?;

        let mut am: user::ActiveModel = user.into();
        am.third_party_id = Set(Some(openid.to_string()));
        am.update(tenant_db).await?;

        // 2. 更新/新增主库索引
        let existing = tenant_user::Entity::find()
            .filter(tenant_user::Column::IdentityType.eq(provider))
            .filter(tenant_user::Column::IdentityValue.eq(openid))
            .one(master_db)
            .await?;

        if existing.is_some() {
            // 已存在则更新
            let mut am: tenant_user::ActiveModel = existing.unwrap().into();
            am.user_name = Set(user_name.to_string());
            am.tenant_id = Set(tenant_id.to_string());
            am.tenant_code = Set(tenant_code.to_string());
            am.user_id = Set(user_id.to_string());
            am.identity_value = Set(Some(openid.to_string()));
            am.update(master_db).await?;
        } else {
            // 不存在则新增
            let am = tenant_user::ActiveModel {
                id: Set(common::snowflake_id()),
                user_name: Set(user_name.to_string()),
                tenant_id: Set(tenant_id.to_string()),
                tenant_code: Set(tenant_code.to_string()),
                user_id: Set(user_id.to_string()),
                identity_type: Set(provider.to_string()),
                identity_value: Set(Some(openid.to_string())),
                status: Set(1),
                ..Default::default()
            };
            am.insert(master_db).await?;
        }

        Ok(())
    }

    /// 同步到主库索引
    async fn sync_to_master_index(
        &self,
        tenant_id: &str,
        tenant_code: &str,
        user_id: &str,
        user_name: &str,
        identity_type: &str,
        identity_value: Option<&str>,
    ) -> Result<(), AppError> {
        // 检查是否已存在
        let existing = tenant_user::Entity::find()
            .filter(tenant_user::Column::UserName.eq(user_name))
            .one(&self.master_db)
            .await?;

        if existing.is_some() {
            // 更新
            let mut am: tenant_user::ActiveModel = existing.unwrap().into();
            am.tenant_id = Set(tenant_id.to_string());
            am.tenant_code = Set(tenant_code.to_string());
            am.user_id = Set(user_id.to_string());
            am.identity_type = Set(identity_type.to_string());
            am.identity_value = Set(identity_value.map(|s| s.to_string()));
            am.update(&self.master_db).await?;
        } else {
            // 新增
            let am = tenant_user::ActiveModel {
                id: Set(common::snowflake_id()),
                user_name: Set(user_name.to_string()),
                tenant_id: Set(tenant_id.to_string()),
                tenant_code: Set(tenant_code.to_string()),
                user_id: Set(user_id.to_string()),
                identity_type: Set(identity_type.to_string()),
                identity_value: Set(identity_value.map(|s| s.to_string())),
                status: Set(1),
                ..Default::default()
            };
            am.insert(&self.master_db).await?;
        }

        Ok(())
    }
}
```

### 4.7 第三方登录扩展

#### 4.7.1 扩展设计

```
┌─────────────────────────────────────────────────────────────────┐
│                    第三方登录扩展架构                              │
│                                                                 │
│   ┌─────────┐   ┌─────────┐   ┌─────────┐   ┌─────────┐      │
│   │  微信    │   │  钉钉    │   │ 企业微信 │   │  飞书    │      │
│   │ WeChat  │   │ DingTalk│   │  Work   │   │  Feishu │      │
│   └────┬────┘   └────┬────┘   └────┬────┘   └────┬────┘      │
│        │              │              │              │           │
│        └──────────────┴──────────────┴──────────────┘           │
│                             │                                   │
│                    ┌────────▼────────┐                          │
│                    │ ThirdPartyAuth │                          │
│                    │    Service     │  ← 抽象第三方登录服务       │
│                    └────────┬────────┘                          │
│                             │                                   │
│         ┌───────────────────┼───────────────────┐               │
│         │                   │                   │               │
│  ┌──────▼──────┐    ┌──────▼──────┐    ┌──────▼──────┐       │
│  │ get_openid  │    │ get_userinfo│    │   bind     │       │
│  │ 获取openid  │    │ 获取用户信息 │    │   绑定     │       │
│  └─────────────┘    └─────────────┘    └─────────────┘       │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 4.7.2 第三方登录流程

```
┌─────────────────────────────────────────────────────────────────┐
│                     微信扫码登录完整流程                            │
│                                                                 │
│  1. 前端请求授权链接                                               │
│     GET /auth/third-party/authorize?provider=wechat&state=acme   │
│                    │                                              │
│                    ▼                                              │
│  2. 后端生成授权URL，跳转微信授权页                                 │
│     https://open.weixin.qq.com/connect/qrconnect?appid=xxx...    │
│                    │                                              │
│                    ▼                                              │
│  3. 用户扫码授权                                                   │
│                    │                                              │
│                    ▼                                              │
│  4. 微信回调本系统                                                 │
│     GET /auth/third-party/callback?code=xxx&state=acme           │
│                    │                                              │
│                    ▼                                              │
│  5. 后端用code换取openid                                          │
│     POST https://api.weixin.qq.com/sns/oauth2/access_token       │
│                    │                                              │
│                    ▼                                              │
│  6. 查询主库索引表，查找 openid 对应的用户                          │
│     SELECT * FROM auth_sys_tenant_user                            │
│     WHERE identity_type='wechat' AND identity_value='openid'     │
│                    │                                              │
│         ┌──────────┴──────────┐                                  │
│         │                     │                                  │
│    找到用户               未找到用户                                │
│         │                     │                                  │
│         ▼                     ▼                                  │
│  ┌──────────────┐    ┌──────────────┐                           │
│  │ 直接登录       │    │ 绑定已有账号  │                           │
│  │ 返回Token     │    │ 或注册新账号  │                           │
│  └──────────────┘    └──────────────┘                           │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

#### 4.7.3 第三方登录实现

```rust
/// 第三方认证服务
pub struct ThirdPartyAuthService {
    #[inject(component)]
    master_db: DbConn,
}

impl ThirdPartyAuthService {
    /// 获取授权URL
    pub fn get_authorize_url(&self, provider: &str, state: &str) -> Result<String, AppError> {
        match provider {
            "wechat" => Ok(self.get_wechat_authorize_url(state)),
            "dingtalk" => Ok(self.get_dingtalk_authorize_url(state)),
            _ => Err(AppError::BadRequest("不支持的第三方登录".to_string())),
        }
    }

    /// 处理第三方回调
    pub async fn handle_callback(
        &self,
        provider: &str,
        code: &str,
        state: &str,
    ) -> Result<ThirdPartyCallbackVo, AppError> {
        // 1. 用 code 换取 openid
        let openid = self.get_openid(provider, code).await?;

        // 2. 查询主库索引表
        let index = tenant_user::Entity::find()
            .filter(tenant_user::Column::IdentityType.eq(provider))
            .filter(tenant_user::Column::IdentityValue.eq(&openid))
            .filter(tenant_user::Column::Status.eq(1))
            .one(&self.master_db)
            .await?;

        match index {
            Some(idx) => {
                // 已绑定用户，直接返回登录信息
                Ok(ThirdPartyCallbackVo {
                    need_bind: false,
                    openid,
                    user_name: Some(idx.user_name),
                    tenant_code: Some(idx.tenant_code),
                })
            }
            None => {
                // 未绑定，需要用户选择租户并绑定
                // state 可以是租户编码（如果前端指定了）
                Ok(ThirdPartyCallbackVo {
                    need_bind: true,
                    openid,
                    user_name: None,
                    tenant_code: Some(state.to_string()),
                })
            }
        }
    }

    /// 绑定第三方账号
    pub async fn bind_account(
        &self,
        provider: &str,
        openid: &str,
        tenant_code: &str,
        user_name: &str,
        pass_word: &str,
    ) -> Result<TokenVo, AppError> {
        // 1. 查询租户
        let tenant = tenant::Entity::find()
            .filter(tenant::Column::TenantCode.eq(tenant_code))
            .filter(tenant::Column::Status.eq(1))
            .one(&self.master_db)
            .await?
            .ok_or_else(|| AppError::BadRequest("租户不存在".to_string()))?;

        // 2. 获取租户库连接
        let tenant_db = self.get_tenant_database(&tenant.id).await?;

        // 3. 验证用户或创建新用户
        let user = user::Entity::find()
            .filter(user::Column::UserName.eq(user_name))
            .one(&tenant_db)
            .await?;

        let user_model = if let Some(u) = user {
            // 验证密码
            if u.pass_word != pass_word {
                return Err(AppError::Unauthorized("密码错误".to_string()));
            }
            u
        } else {
            // 注册新用户
            self.register_with_third_party(&tenant_db, &tenant, user_name, pass_word, provider, openid).await?
        };

        // 4. 绑定第三方账号
        let mut am: user::ActiveModel = user_model.into();
        am.third_party_id = Set(Some(openid.to_string()));
        am.update(&tenant_db).await?;

        // 5. 同步主库索引
        self.sync_third_party_index(&tenant, &user_model, provider, openid).await?;

        // 6. 执行登录
        // ... 调用 AuthAppService::do_login
        todo!()
    }

    async fn get_openid(&self, provider: &str, code: &str) -> Result<String, AppError> {
        match provider {
            "wechat" => self.get_wechat_openid(code).await,
            "dingtalk" => self.get_dingtalk_openid(code).await,
            _ => Err(AppError::BadRequest("不支持的第三方登录".to_string())),
        }
    }

    async fn get_wechat_openid(&self, code: &str) -> Result<String, AppError> {
        let appid = "YOUR_WECHAT_APPID";
        let secret = "YOUR_WECHAT_SECRET";
        let url = format!(
            "https://api.weixin.qq.com/sns/oauth2/access_token?appid={}&secret={}&code={}&grant_type=authorization_code",
            appid, secret, code
        );

        let resp = reqwest::get(&url).await
            .map_err(|e| AppError::Internal(format!("微信接口调用失败: {}", e)))?
            .json::<WechatAccessTokenResponse>().await
            .map_err(|e| AppError::Internal(format!("解析微信响应失败: {}", e)))?;

        Ok(resp.openid)
    }

    async fn sync_third_party_index(
        &self,
        tenant: &tenant::Model,
        user: &user::Model,
        provider: &str,
        openid: &str,
    ) -> Result<(), AppError> {
        // 先删除旧的绑定记录（同一 openid）
        tenant_user::Entity::delete_many()
            .filter(tenant_user::Column::IdentityType.eq(provider))
            .filter(tenant_user::Column::IdentityValue.eq(openid))
            .exec(&self.master_db)
            .await?;

        // 插入新记录
        let am = tenant_user::ActiveModel {
            id: Set(common::snowflake_id()),
            user_name: Set(user.user_name.clone()),
            tenant_id: Set(tenant.id.clone()),
            tenant_code: Set(tenant.tenant_code.clone()),
            user_id: Set(user.id.clone()),
            identity_type: Set(provider.to_string()),
            identity_value: Set(Some(openid.to_string())),
            status: Set(1),
            ..Default::default()
        };
        am.insert(&self.master_db).await?;

        Ok(())
    }
}
```

### 4.8 安全设计

#### 4.8.1 防枚举攻击

| 防护点 | 措施 |
|--------|------|
| locate 接口 | 用户不存在时返回相同错误"用户名或密码错误"，防止枚举用户名 |
| 登录接口 | 无论租户不存在、用户不存在、密码错误，统一返回"用户名或密码错误" |
| 延迟响应 | 用户不存在时模拟 100ms 延迟，防止时序攻击 |
| IP 限流 | 对 locate/login 接口做 IP 级限流（如每分钟 10 次） |

#### 4.8.2 Token 安全

| 设计 | 说明 |
|------|------|
| extra_data 包含 tenantId | 后续请求通过 TenantIdProvider 从 Token 提取租户 ID |
| Redis 存储 | Token、角色、权限存 Redis，支持分布式和登出失效 |
| 过期时间 | 可配置，建议 2 小时 |
| refresh_token | 可选，支持滑动过期 |

---

---

## 5. 权限与角色校验

### 5.1 权限模型

Database 模式下，每个租户库拥有独立的权限数据：

```
租户库 A                    租户库 B
┌──────────────┐           ┌──────────────┐
│ auth_sys_user│           │ auth_sys_user│
│ auth_sys_role│           │ auth_sys_role│
│ auth_sys_menu│           │ auth_sys_menu│
│ user_role    │           │ user_role    │
│ role_menu    │           │ role_menu    │
└──────────────┘           └──────────────┘
  完全独立                    完全独立
```

**核心原则**：角色和权限存储在租户库中，通过 Sa-Token 的 Redis 存储实现运行时校验。

### 5.2 权限加载流程

```
1. 用户登录 → 从租户库查询角色/权限
2. StpUtil::set_roles() / set_permissions() → 写入 Redis
   Key: {prefix}:role:{userId}  → ["admin", "editor"]
   Key: {prefix}:perm:{userId}  → ["user:list", "user:add"]
3. 后续请求 → SaTokenLayer 从 Redis 读取角色/权限
4. #[sa_check_permission("user:list")] → 校验通过/拒绝
```

### 5.3 租户间权限隔离

**关键问题**：不同租户可能有相同的 `userId`（雪花ID碰撞概率极低，但需防范）。

**解决方案**：Sa-Token 的 loginId 加租户前缀：

```rust
// 登录时使用复合 loginId
let composite_login_id = format!("{}:{}", tenant_id, user_id_str);
StpUtil::login_with_extra(&composite_login_id, extra).await?;
```

或者保持 `userId` 作为 loginId，因为：
- 雪花算法生成的 ID 全局唯一
- Redis 中 roles/permissions key 已包含 userId
- 不同租户的权限数据天然隔离（存储在各自租户库中）

**推荐**：保持 userId 作为 loginId，依赖雪花 ID 全局唯一性。若需更强隔离，可使用复合 loginId。

### 5.4 超级管理员

主库管理员（平台管理员）需要跨租户管理能力：

```rust
// 平台管理员登录 — 使用主库
async fn login_platform_admin(&self, dto: LoginDto) -> Result<TokenVo, AppError> {
    let _guard = TenantIgnoreGuard::new();

    let user_model = user::Entity::find()
        .filter(user::Column::UserName.eq(dto.user_name.as_deref().unwrap_or("")))
        .filter(user::Column::AdminFlag.eq(1))
        .one(&self.db)
        .await?
        .ok_or_else(|| AppError::Unauthorized("用户名或密码错误".to_string()))?;

    // ... 验证密码

    let extra = serde_json::json!({
        "userId": user_model.id,
        "userName": user_model.user_name,
        "tenantId": null,          // 无租户归属
        "tenantMode": "platform",  // 平台管理员标记
    });

    let token_value = StpUtil::login_with_extra(&user_model.id.to_string(), extra).await?;
    // ...
}
```

---

## 6. 数据库动态切换

### 6.1 现有机制

`sea-orm-ext` 已提供完整的 Database 模式支持：

| 组件 | 作用 |
|------|------|
| `TenantMiddleware` | 从请求中解析 tenantId，设置上下文，注入连接 |
| `TenantDb` 提取器 | 从 request.extensions 或 ConnectionStore 获取租户连接 |
| `tenant_db()` 函数 | 根据当前上下文自动获取租户连接 |
| `ConnectionStore` | 管理所有租户的数据库连接 |
| `TenantPlugin` | 启动时初始化连接池 |

### 6.2 Handler 中使用租户连接

#### 方式一：TenantDb 提取器（推荐）

```rust
use sea_orm_ext::TenantDb;

#[get("/users")]
#[sa_check_permission("user:list")]
async fn list_users(
    TenantDb(db): TenantDb,  // 自动获取当前租户的数据库连接
) -> Result<impl IntoResponse, WebError> {
    let users = user::Entity::find().all(&db).await?;
    // ...
}
```

#### 方式二：tenant_db() 函数

```rust
use sea_orm_ext::tenant_db;

#[get("/users")]
#[sa_check_permission("user:list")]
async fn list_users(
    Component(default_db): Component<DbConn>,  // 主库连接作为 fallback
) -> Result<impl IntoResponse, WebError> {
    let db = tenant_db(&default_db)?;  // 自动路由到租户库
    let users = user::Entity::find().all(&db).await?;
    // ...
}
```

#### 方式三：同时使用主库和租户库

```rust
#[post("/tenants")]
#[sa_check_permission("tenant:add")]
async fn create_tenant(
    Component(master_db): Component<DbConn>,  // 主库：写租户记录
    TenantDb(tenant_db): TenantDb,            // 租户库：写业务数据
    Json(dto): Json<CreateDto>,
) -> Result<impl IntoResponse, WebError> {
    // 在主库创建租户记录
    let tenant = tenant::ActiveModel::from(dto.clone()).insert(&master_db).await?;

    // 在租户库创建业务数据
    let resource = resource::ActiveModel::from(dto).insert(&tenant_db).await?;

    Ok(Json(ApiResponse::success((tenant, resource))))
}
```

### 6.3 动态注册新租户连接

新增租户后，需将连接注册到 `ConnectionStore`：

```rust
use sea_orm_ext::{get_tenant_store, set_tenant_context, TenantContext};
use sea_query::Value;

async fn register_tenant_connection(tenant_id: &str, database_url: &str) -> Result<(), AppError> {
    let conn = sea_orm::Database::connect(database_url).await
        .map_err(|e| AppError::Internal(format!("连接租户数据库失败: {}", e)))?;

    let store = get_tenant_store()
        .ok_or_else(|| AppError::Internal("租户连接存储未初始化".to_string()))?;

    store.insert(Value::String(Some(tenant_id.to_string())), conn)?;

    tracing::info!("租户 {} 数据库连接已注册", tenant_id);
    Ok(())
}
```

### 6.4 TenantIdProvider 适配

当前项目使用 `SaTokenTenantIdProvider`，从 JWT Token 的 extra_data 中提取 tenantId：

```rust
// common/src/tenant_provider.rs（现有实现）
impl TenantIdProvider for SaTokenTenantIdProvider {
    fn get_tenant_id(&self) -> Option<Value> {
        crate::user::get_current_tenant_id()
            .map(|id| Value::String(Some(id)))
    }
}
```

Database 模式下需确保 `TenantMiddleware` 在 `SaTokenLayer` 之后执行（内层），这样 Token 已解析，tenantId 可用。

当前 `TenantPlugin` 已处理此顺序：

```rust
// sea-orm-ext/src/plugin/tenant.rs
let insert_pos = if app.get_component::<SaTokenLayerMarker>().is_some() {
    // SaTokenLayer 已注册，TenantLayer 放在它之后（内层，更低优先级）
    // 执行顺序：SaTokenLayer(外) → TenantLayer(内) → Handler
    // 即：先解析Token → 再设置租户上下文
    len - 1
} else {
    0
};
```

---

## 7. 注册流程设计

### 7.1 租户自注册

Database 模式下，租户自注册需完成：

```
1. 填写租户信息 + 管理员账号
2. 系统自动创建数据库
3. 初始化 Schema + 默认数据
4. 注册租户记录 + 连接
5. 自动登录
```

```rust
#[post("/register/tenant")]
#[sa_ignore]
async fn register_tenant(
    Component(service): Component<TenantAppService>,
    Component(auth_service): Component<AuthAppService>,
    Json(dto): Json<RegisterTenantDto>,
) -> impl IntoResponse {
    match service.register_tenant_full(dto.clone()).await {
        Ok(tenant) => {
            // 注册成功后自动登录
            let login_dto = LoginDto {
                user_name: Some(dto.admin_user_name),
                pass_word: dto.admin_pass_word,
                tenant_id: Some(tenant.id.clone()),
                ..Default::default()
            };
            match auth_service.login(login_dto).await {
                Ok(token) => Json(ApiResponse::success(token)),
                Err(e) => Json(ApiResponse::error(500, &format!("注册成功但自动登录失败: {}", e))),
            }
        }
        Err(e) => Json(ApiResponse::error(500, &e.to_string())),
    }
}
```

### 7.2 租户内用户注册

租户内普通用户注册，需在租户库中操作：

```rust
pub async fn register(&self, dto: RegisterDto) -> Result<UserInfoVo, AppError> {
    let tenant_mode = sea_orm_ext::get_tenant_mode();

    if tenant_mode == Some(TenantMode::Database) {
        let tenant_id = dto.tenant_id.as_deref()
            .ok_or_else(|| AppError::BadRequest("tenantId必传".to_string()))?;

        let tenant_db = sea_orm_ext::get_database_for_tenant(
            &Value::String(Some(tenant_id.to_string()))
        )?.ok_or_else(|| AppError::Internal("租户数据库连接未找到".to_string()))?;

        // 检查用户名唯一性（租户库内）
        let existing = user::Entity::find()
            .filter(user::Column::UserName.eq(&dto.user_name))
            .one(&tenant_db)
            .await?;
        if existing.is_some() {
            return Err(AppError::BadRequest("用户名已存在".to_string()));
        }

        // 创建用户
        let user_model = user::ActiveModel {
            user_name: Set(dto.user_name),
            pass_word: Set(dto.pass_word),
            nick_name: Set(dto.nick_name),
            email: Set(dto.email),
            phone: Set(dto.phone),
            status: Set(1),
            admin_flag: Set(0),
            dept_id: Set(None),
            ..Default::default()
        };
        let result = user_model.insert(&tenant_db).await?;

        // 同步主库用户索引
        self.sync_tenant_user_index(tenant_id, &result).await?;

        Ok(UserInfoVo::from_model(result))
    } else {
        // Table 模式原有逻辑
        // ...
    }
}
```

---

## 8. 配置

### 8.1 Database 模式配置

```toml
[sea-orm-ext-tenant]
enabled = true
mode = "database"
database_source = "config"       # "config" | "custom"
default_tenant_id = 1

# database_source = "config" 时，启动时自动连接以下数据库
[[sea-orm-ext-tenant.databases]]
tenant_id = 1
[sea-orm-ext-tenant.databases.database]
url = "postgres://postgres:123456@localhost:5432/tenant_1"
max_connections = 10
min_connections = 1
connect_timeout_secs = 30
acquire_timeout_secs = 30

[[sea-orm-ext-tenant.databases]]
tenant_id = 2
[sea-orm-ext-tenant.databases.database]
url = "postgres://postgres:123456@localhost:5432/tenant_2"
max_connections = 10
min_connections = 1
```

### 8.2 Custom 数据源提供者

当 `database_source = "custom"` 时，需实现 `TenantDatabaseProvider`：

```rust
use sea_orm_ext::TenantDatabaseProvider;
use sea_orm::ConnectOptions;
use sea_query::Value;
use std::collections::HashMap;

pub struct DatabaseTenantProvider {
    db: DbConn,
}

impl DatabaseTenantProvider {
    pub fn new(db: DbConn) -> Self {
        Self { db }
    }
}

impl TenantDatabaseProvider for DatabaseTenantProvider {
    fn provide(&self) -> HashMap<Value, ConnectOptions> {
        let mut map = HashMap::new();

        // 从主库 auth_sys_tenant 表读取所有租户的数据库配置
        // 注意：provide() 是同步的，需要用 block_on 或在初始化时预加载
        let tenants = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                tenant::Entity::find()
                    .filter(tenant::Column::Mode.eq("database"))
                    .filter(tenant::Column::Status.eq(1))
                    .all(&self.db)
                    .await
            })
        }).unwrap_or_default();

        for t in tenants {
            if let (Some(db_type), Some(db_url), Some(db_name)) =
                (&t.database_type, &t.database_url, &t.database_name)
            {
                let url = build_database_url(db_type, db_url, Some(db_name));
                let mut opt = ConnectOptions::new(url);
                opt.max_connections(10)
                    .min_connections(1)
                    .connect_timeout(std::time::Duration::from_secs(30))
                    .acquire_timeout(std::time::Duration::from_secs(30));

                let tenant_id = t.id.parse::<i64>().unwrap_or(0);
                map.insert(Value::BigInt(Some(tenant_id)), opt);
            }
        }

        map
    }
}
```

注册方式：

```rust
// main.rs
let provider = Arc::new(DatabaseTenantProvider::new(db.clone()));
let config = TenantPluginConfig {
    enabled: true,
    mode: "database".to_string(),
    database_source: Some("custom".to_string()),
    tenant_database_provider: Some(provider),
    ..Default::default()
};
```

---

## 9. 迁移管理

### 9.1 Schema 版本控制

每个租户库需要独立的迁移管理。建议方案：

| 方案 | 描述 | 优点 | 缺点 |
|------|------|------|------|
| **统一迁移** | 所有租户库执行同一套迁移脚本 | 简单，一致性高 | 需遍历所有租户 |
| **版本标记** | 主库记录每个租户的 Schema 版本 | 可追踪 | 额外维护 |

**推荐统一迁移方案**：

```rust
pub async fn migrate_all_tenants(&self) -> Result<(), AppError> {
    let store = sea_orm_ext::get_tenant_store()
        .ok_or_else(|| AppError::Internal("租户连接存储未初始化".to_string()))?;

    let tenants = tenant::Entity::find()
        .filter(tenant::Column::Mode.eq("database"))
        .filter(tenant::Column::Status.eq(1))
        .all(&self.db)
        .await?;

    for t in &tenants {
        let tenant_id_value = Value::String(Some(t.id.clone()));
        if let Some(conn) = store.get(&tenant_id_value) {
            // 对每个租户库执行迁移
            run_migrations(&conn).await?;
            tracing::info!("租户 {} 迁移完成", t.tenant_name);
        }
    }

    Ok(())
}
```

### 9.2 租户库 Schema 版本表

在 `init_schema.sql` 中增加版本记录表：

```sql
CREATE TABLE IF NOT EXISTS _schema_version (
    version VARCHAR(50) PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    description VARCHAR(500)
);
```

---

## 10. 安全设计

### 10.1 租户越权防护

| 层级 | 防护措施 |
|------|---------|
| **中间件层** | `TenantMiddleware` 从 Token extra_data 提取 tenantId，不可伪造 |
| **连接层** | Database 模式下物理隔离，无法访问其他租户数据库 |
| **Token 层** | JWT 签名保证 extra_data 不可篡改 |
| **API 层** | `#[sa_check_permission]` 校验权限 |

### 10.2 Token 中租户信息

```json
{
  "userId": "1876543210000000001",
  "userName": "admin",
  "nickName": "管理员",
  "tenantId": "1876543210000000001",
  "tenantMode": "database"
}
```

- `tenantId`：标识用户所属租户，用于路由到正确数据库
- `tenantMode`：标识隔离模式，用于区分处理逻辑
- JWT 签名保证这些字段不可被客户端篡改

### 10.3 租户过期检查

```rust
// 在 TenantMiddleware 或登录时检查
if let Some(expire) = tenant.expire_time {
    if expire < chrono::Utc::now() {
        return Err(AppError::Forbidden("租户已过期".to_string()));
    }
}
```

---

## 11. 需要修改的代码清单

### 11.1 新增文件

| 文件 | 说明 |
|------|------|
| `system/entity/src/tenant_user.rs` | 主库租户用户索引实体 |
| `system/application/src/tenant/init_data.sql` | 租户初始化数据模板 |
| `system/domain/src/tenant_user_repository.rs` | 租户用户索引仓储 trait |

### 11.2 修改文件

| 文件 | 修改内容 |
|------|---------|
| `system/application/src/auth/dto.rs` | `LoginDto` 增加 `tenant_id` 字段 |
| `system/application/src/auth/service.rs` | 登录逻辑增加 Database 模式分支 |
| `system/application/src/tenant/service.rs` | 增加 `create_tenant_full`、`register_tenant_connection` 等方法 |
| `system/application/src/tenant/dto.rs` | 增加 `CreateTenantFullDto`、`RegisterTenantDto` 等 |
| `system/interface/src/handlers/auth_handler.rs` | 增加租户列表接口、租户注册接口 |
| `system/interface/src/handlers/tenant_handler.rs` | 增加一体化创建接口 |
| `common/src/tenant_provider.rs` | 适配 Database 模式下 tenantId 类型（String → i64 兼容） |
| `config/app.toml` | 切换为 `mode = "database"` 并配置租户数据库 |

### 11.3 sea-orm-ext 可能的增强

| 增强 | 说明 |
|------|------|
| `HashMapConnectionStore` 支持 String 类型 tenantId | 当前仅支持整数类型，需扩展 |
| `TenantDatabaseProvider` 异步版本 | `provide()` 当前是同步的 |
| 动态连接注册 API | 运行时添加/移除租户连接 |

---

## 12. 实施路线图

| 阶段 | 目标 | 交付物 |
|------|------|--------|
| **Phase 1** | 基础 Database 模式支持 | 配置切换、TenantDb 提取器、登录改造 |
| **Phase 2** | 租户生命周期管理 | 一体化创建、自动初始化、连接动态注册 |
| **Phase 3** | 权限隔离完善 | 租户内权限独立、平台管理员跨租户 |
| **Phase 4** | 运维工具 | 统一迁移、Schema 版本管理、租户监控 |
| **Phase 5** | 高级特性 | 租户自注册、租户过期自动停用、连接池动态调整 |

---

## 附录 A：Database 模式下各场景数据流

### A.1 用户登录（方案 B：租户编码）

```
前端: POST /auth/login { userName, passWord, tenantCode }
  │
  ├─ SaTokenLayer: 尚未登录，无 Token
  │
  ├─ TenantMiddleware: tenantId = null（登录前无租户上下文）
  │
  ├─ Handler: do_login()
  │
  └─ Service: login_database_mode()
      ├─ 主库: 根据 tenantCode 查找租户 (Component<DbConn>)
      │   └─ 验证租户状态 + 过期时间
      ├─ ConnectionStore: 根据 tenantId 获取租户库连接
      ├─ 租户库: 查找用户 + 验证密码 (tenant_db)
      ├─ 租户库: 查询角色/权限 (tenant_db)
      ├─ StpUtil::login_with_extra() → Redis
      │   └─ extra_data 包含 tenantId + tenantCode + tenantMode
      └─ 返回 Token

后续请求: GET /users (Authorization: Bearer {token})
  │
  ├─ SaTokenLayer: 解析 Token → SaTokenContext
  │   └─ extra_data.tenantId 可用
  │
  ├─ TenantMiddleware: 从 SaTokenTenantIdProvider 获取 tenantId
  │   ├─ set_tenant_context(tenantId)
  │   └─ get_tenant_database() → 注入 TenantDb
  │
  ├─ Handler: list_users(TenantDb(db))
  │
  └─ Service: 查询租户库用户
      └─ user::Entity::find().all(&db)  // db = 租户库连接
```

### A.2 业务操作

```
前端: GET /users (Authorization: Bearer {token})
  │
  ├─ SaTokenLayer: 解析 Token → SaTokenContext
  │   └─ extra_data.tenantId 可用
  │
  ├─ TenantMiddleware: 从 SaTokenTenantIdProvider 获取 tenantId
  │   ├─ set_tenant_context(tenantId)
  │   └─ get_tenant_database() → 注入 TenantDb
  │
  ├─ Handler: list_users(TenantDb(db))
  │
  └─ Service: 查询租户库用户
      └─ user::Entity::find().all(&db)  // db = 租户库连接
```

### A.3 租户管理（平台管理员）

```
前端: POST /tenants (Authorization: Bearer {platform_token})
  │
  ├─ SaTokenLayer: 解析 Token → tenantId = null (平台管理员)
  │
  ├─ TenantMiddleware: tenantId = null → 使用 default_tenant_id
  │   └─ 不注入 TenantDb（平台操作不需要租户库）
  │
  ├─ Handler: create_tenant(Component<DbConn>)
  │
  └─ Service: create_tenant_full()
      ├─ 主库: 创建租户记录 (Component<DbConn>)
      ├─ 新建连接: 创建数据库 + 初始化
      └─ ConnectionStore: 注册新连接
```

## 附录 B：ConnectionStore 的 tenantId 类型问题

当前 `HashMapConnectionStore` 内部使用 `HashMap<i64, DatabaseConnection>`，仅支持整数类型 tenantId。但项目中 `auth_sys_tenant.id` 为 `String` 类型（雪花ID字符串）。

**解决方案**（二选一）：

1. **修改 tenant.id 为 i64**：将 `auth_sys_tenant.id` 从 `String` 改为 `i64`，与 ConnectionStore 一致
2. **扩展 ConnectionStore**：在 sea-orm-ext 中增加 String 类型支持

```rust
// 方案2：扩展 HashMapConnectionStore
pub struct HashMapConnectionStore {
    connections: RwLock<HashMap<String, DatabaseConnection>>,  // i64 → String
}
```

**推荐方案 1**，因为雪花ID本质是 i64，使用 String 存储是冗余的。统一为 i64 可与 ConnectionStore 无缝对接。
