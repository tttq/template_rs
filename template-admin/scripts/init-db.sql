-- ============================================================================
-- Template Admin 数据库完整初始化脚本
-- ============================================================================
-- 用途：一键初始化 template 数据库，包含所有表结构、初始数据、菜单权限、角色权限
-- 执行方式：psql -d template -f init-db.sql
-- 或通过 scripts/init-db.ps1 自动调用
--
-- 内容：
--   1. 所有 auth_sys_* 表结构（11 张表）
--   2. 索引
--   3. 初始租户、用户、角色数据
--   4. 完整菜单数据（30 个权限点，与后端 #[sa_check_permission] 宏一一对应）
--   5. 角色-菜单关联（admin 角色拥有全部权限）
--   6. 用户-角色关联（admin 用户绑定 admin 角色）
--   7. 字典数据（性别、状态、菜单类型）
--   8. 系统配置数据
--   9. 使用 pgcrypto 扩展 hash admin 密码（无需依赖 Rust bcrypt）
--
-- 幂等性：脚本可重复执行，已存在的表会被 DROP 重建，数据会重置
-- ============================================================================

-- 启用 pgcrypto 扩展（用于 bcrypt 密码 hash）
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- ============================================================================
-- 1. 清理旧表（按依赖逆序）
-- ============================================================================
DROP TABLE IF EXISTS auth_sys_attachment CASCADE;
DROP TABLE IF EXISTS auth_sys_client CASCADE;
DROP TABLE IF EXISTS auth_sys_login_log CASCADE;
DROP TABLE IF EXISTS auth_sys_resource CASCADE;
DROP TABLE IF EXISTS auth_sys_role_resource CASCADE;
DROP TABLE IF EXISTS auth_sys_third_user CASCADE;
DROP TABLE IF EXISTS auth_sys_token CASCADE;
DROP TABLE IF EXISTS tools_move_car CASCADE;
DROP TABLE IF EXISTS auth_sys_dict_item CASCADE;
DROP TABLE IF EXISTS auth_sys_dict_type CASCADE;
DROP TABLE IF EXISTS auth_sys_config CASCADE;
DROP TABLE IF EXISTS auth_sys_role_menu CASCADE;
DROP TABLE IF EXISTS auth_sys_user_role CASCADE;
DROP TABLE IF EXISTS auth_sys_menu CASCADE;
DROP TABLE IF EXISTS auth_sys_role CASCADE;
DROP TABLE IF EXISTS auth_sys_user CASCADE;
DROP TABLE IF EXISTS auth_sys_dept CASCADE;
DROP TABLE IF EXISTS auth_sys_tenant CASCADE;
DROP TABLE IF EXISTS auth_sys_tenant_user CASCADE;
DROP TABLE IF EXISTS auth_sys_dict CASCADE;
DROP TABLE IF EXISTS sea_orm_migration CASCADE;

-- ============================================================================
-- 2. 创建表结构
-- ============================================================================

-- 租户表
CREATE TABLE auth_sys_tenant (
    id VARCHAR(64) PRIMARY KEY,
    tenant_name VARCHAR(100) NOT NULL,
    tenant_code VARCHAR(50) NOT NULL UNIQUE,
    mode VARCHAR(20) NOT NULL DEFAULT 'table',
    database_type VARCHAR(20),
    database_url VARCHAR(500),
    database_name VARCHAR(100),
    status INT NOT NULL DEFAULT 1,
    contact_name VARCHAR(50),
    contact_phone VARCHAR(20),
    contact_email VARCHAR(100),
    expire_time TIMESTAMPTZ,
    remark VARCHAR(500),
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by VARCHAR(50),
    create_id VARCHAR(50),
    update_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_by VARCHAR(50),
    update_id VARCHAR(50),
    version INT NOT NULL DEFAULT 1,
    delete_flag INT NOT NULL DEFAULT 0
);

-- 部门表
CREATE TABLE auth_sys_dept (
    id VARCHAR(64) PRIMARY KEY,
    parent_id VARCHAR(64) NOT NULL DEFAULT '0',
    dept_name VARCHAR(100) NOT NULL,
    dept_sort INT NOT NULL DEFAULT 0,
    status INT NOT NULL DEFAULT 1,
    leader VARCHAR(50),
    phone VARCHAR(20),
    email VARCHAR(100),
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by VARCHAR(50),
    create_id VARCHAR(50),
    update_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_by VARCHAR(50),
    update_id VARCHAR(50),
    version INT NOT NULL DEFAULT 1,
    delete_flag INT NOT NULL DEFAULT 0,
    tenant_id VARCHAR(50)
);

-- 用户表
CREATE TABLE auth_sys_user (
    id VARCHAR(64) PRIMARY KEY,
    user_name VARCHAR(100) NOT NULL UNIQUE,
    nick_name VARCHAR(100),
    pass_word VARCHAR(200) NOT NULL,
    email VARCHAR(100),
    phone VARCHAR(20),
    avatar VARCHAR(500),
    status INT NOT NULL DEFAULT 1,
    admin_flag INT NOT NULL DEFAULT 0,
    dept_id VARCHAR(64),
    identity_type VARCHAR(50) DEFAULT 'username',
    identity_value VARCHAR(200),
    third_party_id VARCHAR(200),
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by VARCHAR(50),
    create_id VARCHAR(50),
    update_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_by VARCHAR(50),
    update_id VARCHAR(50),
    version INT NOT NULL DEFAULT 1,
    delete_flag INT NOT NULL DEFAULT 0,
    tenant_id VARCHAR(50)
);

-- 角色表
CREATE TABLE auth_sys_role (
    id VARCHAR(64) PRIMARY KEY,
    parent_id VARCHAR(64) NOT NULL DEFAULT '0',
    role_name VARCHAR(100) NOT NULL,
    role_code VARCHAR(50) NOT NULL UNIQUE,
    role_sort INT NOT NULL DEFAULT 0,
    status INT NOT NULL DEFAULT 1,
    remark VARCHAR(500),
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by VARCHAR(50),
    create_id VARCHAR(50),
    update_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_by VARCHAR(50),
    update_id VARCHAR(50),
    version INT NOT NULL DEFAULT 1,
    delete_flag INT NOT NULL DEFAULT 0,
    tenant_id VARCHAR(50)
);

-- 菜单表
CREATE TABLE auth_sys_menu (
    id VARCHAR(64) PRIMARY KEY,
    parent_id VARCHAR(64) NOT NULL DEFAULT '0',
    menu_name VARCHAR(100) NOT NULL,
    menu_type VARCHAR(20) NOT NULL DEFAULT 'menu',
    path VARCHAR(200),
    component VARCHAR(200),
    icon VARCHAR(100),
    sort_order INT NOT NULL DEFAULT 0,
    permission VARCHAR(100),
    status INT NOT NULL DEFAULT 1,
    visible INT NOT NULL DEFAULT 1,
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by VARCHAR(50),
    create_id VARCHAR(50),
    update_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_by VARCHAR(50),
    update_id VARCHAR(50),
    version INT NOT NULL DEFAULT 1,
    delete_flag INT NOT NULL DEFAULT 0,
    tenant_id VARCHAR(50)
);

-- 用户-角色关联表
CREATE TABLE auth_sys_user_role (
    id VARCHAR(64) PRIMARY KEY,
    user_id VARCHAR(64) NOT NULL,
    role_id VARCHAR(64) NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version INT NOT NULL DEFAULT 1,
    delete_flag INT NOT NULL DEFAULT 0,
    tenant_id VARCHAR(50)
);

-- 角色-菜单关联表
CREATE TABLE auth_sys_role_menu (
    id VARCHAR(64) PRIMARY KEY,
    role_id VARCHAR(64) NOT NULL,
    menu_id VARCHAR(64) NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version INT NOT NULL DEFAULT 1,
    delete_flag INT NOT NULL DEFAULT 0,
    tenant_id VARCHAR(50)
);

-- 字典类型表
CREATE TABLE auth_sys_dict_type (
    id VARCHAR(64) PRIMARY KEY,
    dict_name VARCHAR(100) NOT NULL,
    dict_type VARCHAR(100) NOT NULL UNIQUE,
    status INT NOT NULL DEFAULT 1,
    remark VARCHAR(500),
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by VARCHAR(50),
    create_id VARCHAR(50),
    update_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_by VARCHAR(50),
    update_id VARCHAR(50),
    version INT NOT NULL DEFAULT 1,
    delete_flag INT NOT NULL DEFAULT 0,
    tenant_id VARCHAR(50)
);

-- 字典项表
CREATE TABLE auth_sys_dict_item (
    id VARCHAR(64) PRIMARY KEY,
    dict_type_id VARCHAR(64) NOT NULL,
    dict_label VARCHAR(100) NOT NULL,
    dict_value VARCHAR(100) NOT NULL,
    sort_order INT NOT NULL DEFAULT 0,
    status INT NOT NULL DEFAULT 1,
    remark VARCHAR(500),
    css_class VARCHAR(50),
    list_class VARCHAR(50),
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by VARCHAR(50),
    create_id VARCHAR(50),
    update_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_by VARCHAR(50),
    update_id VARCHAR(50),
    version INT NOT NULL DEFAULT 1,
    delete_flag INT NOT NULL DEFAULT 0,
    tenant_id VARCHAR(50)
);

-- 系统配置表
CREATE TABLE auth_sys_config (
    id VARCHAR(64) PRIMARY KEY,
    config_name VARCHAR(100) NOT NULL,
    config_key VARCHAR(100) NOT NULL UNIQUE,
    config_value VARCHAR(500) NOT NULL,
    config_type VARCHAR(20) NOT NULL DEFAULT 'string',
    remark VARCHAR(500),
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by VARCHAR(50),
    create_id VARCHAR(50),
    update_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_by VARCHAR(50),
    update_id VARCHAR(50),
    version INT NOT NULL DEFAULT 1,
    delete_flag INT NOT NULL DEFAULT 0,
    tenant_id VARCHAR(50)
);

-- 租户用户关联表
CREATE TABLE auth_sys_tenant_user (
    id VARCHAR(64) PRIMARY KEY,
    user_name VARCHAR(100) NOT NULL,
    tenant_id VARCHAR(64) NOT NULL,
    tenant_code VARCHAR(50) NOT NULL,
    user_id VARCHAR(64) NOT NULL,
    identity_type VARCHAR(50) DEFAULT 'username',
    identity_value VARCHAR(200),
    status INT NOT NULL DEFAULT 1,
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by VARCHAR(50),
    update_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_by VARCHAR(50)
);

-- ============================================================================
-- 3. 创建索引
-- ============================================================================
CREATE UNIQUE INDEX idx_tenant_user_name ON auth_sys_tenant_user(user_name);
CREATE INDEX idx_tenant_user_tenant ON auth_sys_tenant_user(tenant_id);
CREATE INDEX idx_tenant_user_identity ON auth_sys_tenant_user(identity_type, identity_value);
CREATE INDEX idx_user_role_user_id ON auth_sys_user_role(user_id);
CREATE INDEX idx_user_role_role_id ON auth_sys_user_role(role_id);
CREATE INDEX idx_role_menu_role_id ON auth_sys_role_menu(role_id);
CREATE INDEX idx_role_menu_menu_id ON auth_sys_role_menu(menu_id);
CREATE INDEX idx_dict_item_type_id ON auth_sys_dict_item(dict_type_id);

-- ============================================================================
-- 4. 初始化租户数据
-- ============================================================================
INSERT INTO auth_sys_tenant (id, tenant_name, tenant_code, mode, status, create_time, update_time, version, delete_flag) VALUES
    ('1876543210000000001', '默认租户', 'default', 'table', 1, NOW(), NOW(), 1, 0),
    ('1876543210000000002', '独立租户', 'independent', 'database', 'postgres', 'postgres://postgres:root@localhost:5432/template3', 'template3', 1, NOW(), NOW(), 1, 0);

-- ============================================================================
-- 5. 初始化用户数据（密码先用占位符，后面用 pgcrypto hash）
-- ============================================================================
INSERT INTO auth_sys_user (id, user_name, nick_name, pass_word, email, status, admin_flag, identity_type, create_time, update_time, version, delete_flag) VALUES
    ('1876543210000000002', 'admin', '超级管理员', 'PLACEHOLDER_WILL_BE_HASHED', 'admin@example.com', 1, 1, 'username', NOW(), NOW(), 1, 0);

-- ============================================================================
-- 6. 初始化租户用户关联
-- ============================================================================
INSERT INTO auth_sys_tenant_user (id, user_name, tenant_id, tenant_code, user_id, identity_type, status, create_time, update_time) VALUES
    ('1876543210000000090', 'admin', '1876543210000000001', 'default', '1876543210000000002', 'username', 1, NOW(), NOW());

-- ============================================================================
-- 7. 初始化角色数据
-- ============================================================================
INSERT INTO auth_sys_role (id, parent_id, role_name, role_code, role_sort, status, create_time, update_time, version, delete_flag) VALUES
    ('1876543210000000003', '0', '超级管理员', 'admin', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000004', '0', '普通用户', 'user', 2, 1, NOW(), NOW(), 1, 0);

-- ============================================================================
-- 8. 初始化用户-角色关联
-- ============================================================================
INSERT INTO auth_sys_user_role (id, user_id, role_id, create_time, version, delete_flag) VALUES
    ('1876543210000000005', '1876543210000000002', '1876543210000000003', NOW(), 1, 0);

-- ============================================================================
-- 9. 初始化菜单数据（30 个权限点，与后端 #[sa_check_permission] 宏一一对应）
-- ============================================================================
-- 顶级菜单
INSERT INTO auth_sys_menu (id, parent_id, menu_name, menu_type, path, component, icon, sort_order, permission, status, visible, create_time, update_time, version, delete_flag) VALUES
    -- 顶级
    ('1876543210000000010', '0', '仪表盘', 'menu', '/dashboard', 'dashboard/index', 'DashboardOutlined', 1, NULL, 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000011', '0', '系统管理', 'dir', '/system', NULL, 'SettingOutlined', 2, NULL, 1, 1, NOW(), NOW(), 1, 0),
    -- 系统管理下的菜单项
    ('1876543210000000012', '1876543210000000011', '用户管理', 'menu', '/system/user', 'system/user/index', 'UserOutlined', 1, 'user:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000013', '1876543210000000011', '角色管理', 'menu', '/system/role', 'system/role/index', 'TeamOutlined', 2, 'role:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000014', '1876543210000000011', '菜单管理', 'menu', '/system/menu', 'system/menu/index', 'MenuOutlined', 3, 'menu:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000015', '1876543210000000011', '部门管理', 'menu', '/system/dept', 'system/dept/index', 'ApartmentOutlined', 4, 'dept:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000016', '1876543210000000011', '租户管理', 'menu', '/system/tenant', 'system/tenant/index', 'BankOutlined', 5, 'tenant:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000017', '1876543210000000011', '字典管理', 'menu', '/system/dict', 'system/dict/index', 'BookOutlined', 6, 'dict:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000018', '1876543210000000011', '参数配置', 'menu', '/system/config', 'system/config/index', 'ToolOutlined', 7, 'config:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000040', '1876543210000000011', '服务监控', 'menu', '/system/monitor', 'system/monitor/index', 'DashboardOutlined', 8, 'monitor:server', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000041', '1876543210000000011', '代码生成', 'menu', '/system/generator', 'system/generator/index', 'CodeOutlined', 9, 'tools:generator', 1, 1, NOW(), NOW(), 1, 0),
    -- 用户管理按钮
    ('1876543210000000019', '1876543210000000012', '用户新增', 'button', NULL, NULL, NULL, 1, 'user:add', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000020', '1876543210000000012', '用户编辑', 'button', NULL, NULL, NULL, 2, 'user:edit', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000021', '1876543210000000012', '用户删除', 'button', NULL, NULL, NULL, 3, 'user:delete', 1, 1, NOW(), NOW(), 1, 0),
    -- 角色管理按钮
    ('1876543210000000022', '1876543210000000013', '角色新增', 'button', NULL, NULL, NULL, 1, 'role:add', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000023', '1876543210000000013', '角色编辑', 'button', NULL, NULL, NULL, 2, 'role:edit', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000024', '1876543210000000013', '角色删除', 'button', NULL, NULL, NULL, 3, 'role:delete', 1, 1, NOW(), NOW(), 1, 0),
    -- 菜单管理按钮
    ('1876543210000000025', '1876543210000000014', '菜单新增', 'button', NULL, NULL, NULL, 1, 'menu:add', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000026', '1876543210000000014', '菜单编辑', 'button', NULL, NULL, NULL, 2, 'menu:edit', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000027', '1876543210000000014', '菜单删除', 'button', NULL, NULL, NULL, 3, 'menu:delete', 1, 1, NOW(), NOW(), 1, 0),
    -- 部门管理按钮
    ('1876543210000000028', '1876543210000000015', '部门新增', 'button', NULL, NULL, NULL, 1, 'dept:add', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000029', '1876543210000000015', '部门编辑', 'button', NULL, NULL, NULL, 2, 'dept:edit', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000030', '1876543210000000015', '部门删除', 'button', NULL, NULL, NULL, 3, 'dept:delete', 1, 1, NOW(), NOW(), 1, 0),
    -- 租户管理按钮
    ('1876543210000000031', '1876543210000000016', '租户新增', 'button', NULL, NULL, NULL, 1, 'tenant:add', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000032', '1876543210000000016', '租户编辑', 'button', NULL, NULL, NULL, 2, 'tenant:edit', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000033', '1876543210000000016', '租户删除', 'button', NULL, NULL, NULL, 3, 'tenant:delete', 1, 1, NOW(), NOW(), 1, 0),
    -- 字典管理按钮
    ('1876543210000000034', '1876543210000000017', '字典新增', 'button', NULL, NULL, NULL, 1, 'dict:add', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000035', '1876543210000000017', '字典编辑', 'button', NULL, NULL, NULL, 2, 'dict:edit', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000036', '1876543210000000017', '字典删除', 'button', NULL, NULL, NULL, 3, 'dict:delete', 1, 1, NOW(), NOW(), 1, 0),
    -- 参数配置按钮
    ('1876543210000000037', '1876543210000000018', '配置新增', 'button', NULL, NULL, NULL, 1, 'config:add', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000038', '1876543210000000018', '配置编辑', 'button', NULL, NULL, NULL, 2, 'config:edit', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000039', '1876543210000000018', '配置删除', 'button', NULL, NULL, NULL, 3, 'config:delete', 1, 1, NOW(), NOW(), 1, 0);

-- ============================================================================
-- 10. 初始化角色-菜单关联（admin 角色拥有全部菜单权限）
-- ============================================================================
INSERT INTO auth_sys_role_menu (id, role_id, menu_id, create_time, version, delete_flag) VALUES
    ('1876543210000000050', '1876543210000000003', '1876543210000000010', NOW(), 1, 0),
    ('1876543210000000051', '1876543210000000003', '1876543210000000011', NOW(), 1, 0),
    ('1876543210000000052', '1876543210000000003', '1876543210000000012', NOW(), 1, 0),
    ('1876543210000000053', '1876543210000000003', '1876543210000000013', NOW(), 1, 0),
    ('1876543210000000054', '1876543210000000003', '1876543210000000014', NOW(), 1, 0),
    ('1876543210000000055', '1876543210000000003', '1876543210000000015', NOW(), 1, 0),
    ('1876543210000000056', '1876543210000000003', '1876543210000000016', NOW(), 1, 0),
    ('1876543210000000057', '1876543210000000003', '1876543210000000017', NOW(), 1, 0),
    ('1876543210000000058', '1876543210000000003', '1876543210000000018', NOW(), 1, 0),
    ('1876543210000000059', '1876543210000000003', '1876543210000000019', NOW(), 1, 0),
    ('1876543210000000060', '1876543210000000003', '1876543210000000020', NOW(), 1, 0),
    ('1876543210000000061', '1876543210000000003', '1876543210000000021', NOW(), 1, 0),
    ('1876543210000000062', '1876543210000000003', '1876543210000000022', NOW(), 1, 0),
    ('1876543210000000063', '1876543210000000003', '1876543210000000023', NOW(), 1, 0),
    ('1876543210000000064', '1876543210000000003', '1876543210000000024', NOW(), 1, 0),
    ('1876543210000000065', '1876543210000000003', '1876543210000000025', NOW(), 1, 0),
    ('1876543210000000066', '1876543210000000003', '1876543210000000026', NOW(), 1, 0),
    ('1876543210000000067', '1876543210000000003', '1876543210000000027', NOW(), 1, 0),
    ('1876543210000000068', '1876543210000000003', '1876543210000000028', NOW(), 1, 0),
    ('1876543210000000069', '1876543210000000003', '1876543210000000029', NOW(), 1, 0),
    ('1876543210000000070', '1876543210000000003', '1876543210000000030', NOW(), 1, 0),
    ('1876543210000000071', '1876543210000000003', '1876543210000000031', NOW(), 1, 0),
    ('1876543210000000072', '1876543210000000003', '1876543210000000032', NOW(), 1, 0),
    ('1876543210000000073', '1876543210000000003', '1876543210000000033', NOW(), 1, 0),
    ('1876543210000000074', '1876543210000000003', '1876543210000000034', NOW(), 1, 0),
    ('1876543210000000075', '1876543210000000003', '1876543210000000035', NOW(), 1, 0),
    ('1876543210000000076', '1876543210000000003', '1876543210000000036', NOW(), 1, 0),
    ('1876543210000000077', '1876543210000000003', '1876543210000000037', NOW(), 1, 0),
    ('1876543210000000078', '1876543210000000003', '1876543210000000038', NOW(), 1, 0),
    ('1876543210000000079', '1876543210000000003', '1876543210000000039', NOW(), 1, 0),
    ('1876543210000000080', '1876543210000000003', '1876543210000000040', NOW(), 1, 0),
    ('1876543210000000081', '1876543210000000003', '1876543210000000041', NOW(), 1, 0);

-- ============================================================================
-- 11. 初始化字典数据
-- ============================================================================
INSERT INTO auth_sys_dict_type (id, dict_name, dict_type, status, create_time, update_time, version, delete_flag) VALUES
    ('1876543210000000100', '性别', 'sys_gender', 1, NOW(), NOW(), 1, 0),
    ('1876543210000000101', '状态', 'sys_status', 1, NOW(), NOW(), 1, 0),
    ('1876543210000000102', '菜单类型', 'sys_menu_type', 1, NOW(), NOW(), 1, 0);

INSERT INTO auth_sys_dict_item (id, dict_type_id, dict_label, dict_value, sort_order, status, create_time, update_time, version, delete_flag) VALUES
    ('1876543210000000110', '1876543210000000100', '男', 'male', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000111', '1876543210000000100', '女', 'female', 2, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000112', '1876543210000000101', '启用', '1', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000113', '1876543210000000101', '禁用', '0', 2, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000114', '1876543210000000102', '目录', 'dir', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000115', '1876543210000000102', '菜单', 'menu', 2, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000116', '1876543210000000102', '按钮', 'button', 3, 1, NOW(), NOW(), 1, 0);

-- ============================================================================
-- 12. 初始化系统配置
-- ============================================================================
INSERT INTO auth_sys_config (id, config_name, config_key, config_value, config_type, create_time, update_time, version, delete_flag) VALUES
    ('1876543210000000200', '系统名称', 'sys.name', 'Template Admin', 'string', NOW(), NOW(), 1, 0),
    ('1876543210000000201', '默认主题', 'sys.theme', 'light', 'string', NOW(), NOW(), 1, 0),
    ('1876543210000000202', '分页大小', 'sys.page.size', '10', 'number', NOW(), NOW(), 1, 0);

-- ============================================================================
-- 13. 使用 pgcrypto hash admin 密码（无需依赖 Rust bcrypt crate）
-- ============================================================================
-- crypt('admin123', gen_salt('bf', 12)) 生成 bcrypt 格式密码哈希（$2a$ 开头）
-- 与 bcrypt crate 生成的 $2b$ 格式兼容，后端 bcrypt::verify 可正常验证
UPDATE auth_sys_user
SET pass_word = crypt('admin123', gen_salt('bf', 12))
WHERE user_name = 'admin';

-- ============================================================================
-- 14. 验证初始化结果
-- ============================================================================
DO $$
DECLARE
    tenant_count INT;
    user_count INT;
    role_count INT;
    menu_count INT;
    role_menu_count INT;
    dict_type_count INT;
    dict_item_count INT;
    config_count INT;
BEGIN
    SELECT COUNT(*) INTO tenant_count FROM auth_sys_tenant;
    SELECT COUNT(*) INTO user_count FROM auth_sys_user;
    SELECT COUNT(*) INTO role_count FROM auth_sys_role;
    SELECT COUNT(*) INTO menu_count FROM auth_sys_menu;
    SELECT COUNT(*) INTO role_menu_count FROM auth_sys_role_menu;
    SELECT COUNT(*) INTO dict_type_count FROM auth_sys_dict_type;
    SELECT COUNT(*) INTO dict_item_count FROM auth_sys_dict_item;
    SELECT COUNT(*) INTO config_count FROM auth_sys_config;

    RAISE NOTICE '================================================';
    RAISE NOTICE '数据库初始化完成';
    RAISE NOTICE '================================================';
    RAISE NOTICE '租户数: %', tenant_count;
    RAISE NOTICE '用户数: %', user_count;
    RAISE NOTICE '角色数: %', role_count;
    RAISE NOTICE '菜单数: % (含按钮权限点)', menu_count;
    RAISE NOTICE '角色-菜单关联数: %', role_menu_count;
    RAISE NOTICE '字典类型数: %', dict_type_count;
    RAISE NOTICE '字典项数: %', dict_item_count;
    RAISE NOTICE '系统配置数: %', config_count;
    RAISE NOTICE '------------------------------------------------';
    RAISE NOTICE '默认登录账号: admin';
    RAISE NOTICE '默认登录密码: admin123';
    RAISE NOTICE '================================================';
END $$;
