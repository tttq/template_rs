CREATE TABLE IF NOT EXISTS auth_sys_tenant (
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

CREATE TABLE IF NOT EXISTS auth_sys_dept (
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

CREATE TABLE IF NOT EXISTS auth_sys_user (
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

-- 客户端表（权限体系顶级维度：客户端 → 其下的菜单/功能权限 → 角色）。
-- 客户端注册表是全局表（主库 auth_sys_client），此处仅保证租户库结构一致，
-- 不写入种子数据；登录与客户端管理接口均从主库读取（见 common/sea-orm-ext 的 TenantIgnoreGuard 用法）。
CREATE TABLE IF NOT EXISTS auth_sys_client (
    id VARCHAR(64) PRIMARY KEY,
    client_code VARCHAR(64) NOT NULL UNIQUE,
    client_secret VARCHAR(200) NOT NULL,
    client_name VARCHAR(100) NOT NULL,
    client_type VARCHAR(20) NOT NULL DEFAULT 'web',
    logo VARCHAR(500),
    home_path VARCHAR(200),
    sort_order INT NOT NULL DEFAULT 0,
    status INT NOT NULL DEFAULT 1,
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

CREATE TABLE IF NOT EXISTS auth_sys_role (
    id VARCHAR(64) PRIMARY KEY,
    parent_id VARCHAR(64) NOT NULL DEFAULT '0',
    -- 所属客户端（全局注册表 auth_sys_client 的主键）；默认归属「管理后台」客户端
    client_id VARCHAR(64) NOT NULL DEFAULT '1876543210000000300',
    role_name VARCHAR(100) NOT NULL,
    role_code VARCHAR(50) NOT NULL,
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

CREATE TABLE IF NOT EXISTS auth_sys_menu (
    id VARCHAR(64) PRIMARY KEY,
    parent_id VARCHAR(64) NOT NULL DEFAULT '0',
    -- 所属客户端（客户端为权限体系顶级维度，菜单/功能权限挂在客户端下）
    client_id VARCHAR(64) NOT NULL DEFAULT '1876543210000000300',
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

CREATE TABLE IF NOT EXISTS auth_sys_user_role (
    id VARCHAR(64) PRIMARY KEY,
    user_id VARCHAR(64) NOT NULL,
    role_id VARCHAR(64) NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tenant_id VARCHAR(50)
);

CREATE TABLE IF NOT EXISTS auth_sys_role_menu (
    id VARCHAR(64) PRIMARY KEY,
    role_id VARCHAR(64) NOT NULL,
    menu_id VARCHAR(64) NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tenant_id VARCHAR(50)
);

CREATE TABLE IF NOT EXISTS auth_sys_dict_type (
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

CREATE TABLE IF NOT EXISTS auth_sys_dict_item (
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

CREATE TABLE IF NOT EXISTS auth_sys_config (
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

CREATE INDEX IF NOT EXISTS idx_user_role_user_id ON auth_sys_user_role(user_id);
CREATE INDEX IF NOT EXISTS idx_user_role_role_id ON auth_sys_user_role(role_id);
CREATE INDEX IF NOT EXISTS idx_role_menu_role_id ON auth_sys_role_menu(role_id);
CREATE INDEX IF NOT EXISTS idx_role_menu_menu_id ON auth_sys_role_menu(menu_id);
CREATE INDEX IF NOT EXISTS idx_dict_item_type_id ON auth_sys_dict_item(dict_type_id);
CREATE INDEX IF NOT EXISTS idx_sys_menu_client ON auth_sys_menu(client_id);
CREATE INDEX IF NOT EXISTS idx_sys_role_client ON auth_sys_role(client_id);
-- 角色编码在客户端内唯一：不同客户端可以各有一套 admin/user 角色
CREATE UNIQUE INDEX IF NOT EXISTS idx_sys_role_client_code ON auth_sys_role(client_id, role_code)
