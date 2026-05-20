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
DROP TABLE IF EXISTS auth_sys_dict CASCADE;
DROP TABLE IF EXISTS sea_orm_migration CASCADE;

CREATE TABLE auth_sys_tenant (
    id BIGINT PRIMARY KEY,
    tenant_name VARCHAR(100) NOT NULL,
    tenant_code VARCHAR(50) NOT NULL UNIQUE,
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

CREATE TABLE auth_sys_dept (
    id BIGINT PRIMARY KEY,
    parent_id BIGINT NOT NULL DEFAULT 0,
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

CREATE TABLE auth_sys_user (
    id BIGINT PRIMARY KEY,
    user_name VARCHAR(100) NOT NULL UNIQUE,
    nick_name VARCHAR(100),
    pass_word VARCHAR(200) NOT NULL,
    email VARCHAR(100),
    phone VARCHAR(20),
    avatar VARCHAR(500),
    status INT NOT NULL DEFAULT 1,
    admin_flag INT NOT NULL DEFAULT 0,
    dept_id BIGINT,
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

CREATE TABLE auth_sys_role (
    id BIGINT PRIMARY KEY,
    parent_id BIGINT NOT NULL DEFAULT 0,
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

CREATE TABLE auth_sys_menu (
    id BIGINT PRIMARY KEY,
    parent_id BIGINT NOT NULL DEFAULT 0,
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

CREATE TABLE auth_sys_user_role (
    id BIGINT PRIMARY KEY,
    user_id BIGINT NOT NULL,
    role_id BIGINT NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version INT NOT NULL DEFAULT 1,
    delete_flag INT NOT NULL DEFAULT 0,
    tenant_id VARCHAR(50)
);

CREATE TABLE auth_sys_role_menu (
    id BIGINT PRIMARY KEY,
    role_id BIGINT NOT NULL,
    menu_id BIGINT NOT NULL,
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version INT NOT NULL DEFAULT 1,
    delete_flag INT NOT NULL DEFAULT 0,
    tenant_id VARCHAR(50)
);

CREATE TABLE auth_sys_dict_type (
    id BIGINT PRIMARY KEY,
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

CREATE TABLE auth_sys_dict_item (
    id BIGINT PRIMARY KEY,
    dict_type_id BIGINT NOT NULL,
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

CREATE TABLE auth_sys_config (
    id BIGINT PRIMARY KEY,
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

CREATE INDEX idx_user_role_user_id ON auth_sys_user_role(user_id);
CREATE INDEX idx_user_role_role_id ON auth_sys_user_role(role_id);
CREATE INDEX idx_role_menu_role_id ON auth_sys_role_menu(role_id);
CREATE INDEX idx_role_menu_menu_id ON auth_sys_role_menu(menu_id);
CREATE INDEX idx_dict_item_type_id ON auth_sys_dict_item(dict_type_id);

INSERT INTO auth_sys_tenant (id, tenant_name, tenant_code, status, create_time, update_time, version, delete_flag)
VALUES (1, 'default', 'default', 1, NOW(), NOW(), 1, 0);

INSERT INTO auth_sys_user (id, user_name, nick_name, pass_word, email, status, admin_flag, create_time, update_time, version, delete_flag)
VALUES (1, 'admin', 'Administrator', 'admin123', 'admin@example.com', 1, 1, NOW(), NOW(), 1, 0);

INSERT INTO auth_sys_role (id, parent_id, role_name, role_code, role_sort, status, create_time, update_time, version, delete_flag)
VALUES (1, 0, 'Administrator', 'admin', 1, 1, NOW(), NOW(), 1, 0);

INSERT INTO auth_sys_role (id, parent_id, role_name, role_code, role_sort, status, create_time, update_time, version, delete_flag)
VALUES (2, 0, 'User', 'user', 2, 1, NOW(), NOW(), 1, 0);

INSERT INTO auth_sys_user_role (id, user_id, role_id, create_time, version, delete_flag)
VALUES (1, 1, 1, NOW(), 1, 0);

INSERT INTO auth_sys_menu (id, parent_id, menu_name, menu_type, path, component, icon, sort_order, permission, status, visible, create_time, update_time, version, delete_flag)
VALUES
    (1, 0, 'Dashboard', 'menu', '/dashboard', 'dashboard/index', 'DashboardOutlined', 1, NULL, 1, 1, NOW(), NOW(), 1, 0),
    (2, 0, 'System', 'dir', '/system', NULL, 'SettingOutlined', 2, NULL, 1, 1, NOW(), NOW(), 1, 0),
    (3, 2, 'User', 'menu', '/system/user', 'system/user/index', 'UserOutlined', 1, 'user:list', 1, 1, NOW(), NOW(), 1, 0),
    (4, 2, 'Role', 'menu', '/system/role', 'system/role/index', 'TeamOutlined', 2, 'role:list', 1, 1, NOW(), NOW(), 1, 0),
    (5, 2, 'Menu', 'menu', '/system/menu', 'system/menu/index', 'MenuOutlined', 3, 'menu:list', 1, 1, NOW(), NOW(), 1, 0),
    (6, 2, 'Dept', 'menu', '/system/dept', 'system/dept/index', 'ApartmentOutlined', 4, 'dept:list', 1, 1, NOW(), NOW(), 1, 0),
    (7, 2, 'Tenant', 'menu', '/system/tenant', 'system/tenant/index', 'BankOutlined', 5, 'tenant:list', 1, 1, NOW(), NOW(), 1, 0),
    (8, 2, 'Dict', 'menu', '/system/dict', 'system/dict/index', 'BookOutlined', 6, 'dict:list', 1, 1, NOW(), NOW(), 1, 0),
    (9, 2, 'Config', 'menu', '/system/config', 'system/config/index', 'ToolOutlined', 7, 'config:list', 1, 1, NOW(), NOW(), 1, 0),
    (10, 3, 'User Add', 'button', NULL, NULL, NULL, 1, 'user:add', 1, 1, NOW(), NOW(), 1, 0),
    (11, 3, 'User Edit', 'button', NULL, NULL, NULL, 2, 'user:edit', 1, 1, NOW(), NOW(), 1, 0),
    (12, 3, 'User Delete', 'button', NULL, NULL, NULL, 3, 'user:delete', 1, 1, NOW(), NOW(), 1, 0),
    (13, 4, 'Role Add', 'button', NULL, NULL, NULL, 1, 'role:add', 1, 1, NOW(), NOW(), 1, 0),
    (14, 4, 'Role Edit', 'button', NULL, NULL, NULL, 2, 'role:edit', 1, 1, NOW(), NOW(), 1, 0),
    (15, 4, 'Role Delete', 'button', NULL, NULL, NULL, 3, 'role:delete', 1, 1, NOW(), NOW(), 1, 0),
    (16, 5, 'Menu Add', 'button', NULL, NULL, NULL, 1, 'menu:add', 1, 1, NOW(), NOW(), 1, 0),
    (17, 5, 'Menu Edit', 'button', NULL, NULL, NULL, 2, 'menu:edit', 1, 1, NOW(), NOW(), 1, 0),
    (18, 5, 'Menu Delete', 'button', NULL, NULL, NULL, 3, 'menu:delete', 1, 1, NOW(), NOW(), 1, 0),
    (19, 6, 'Dept Add', 'button', NULL, NULL, NULL, 1, 'dept:add', 1, 1, NOW(), NOW(), 1, 0),
    (20, 6, 'Dept Edit', 'button', NULL, NULL, NULL, 2, 'dept:edit', 1, 1, NOW(), NOW(), 1, 0),
    (21, 6, 'Dept Delete', 'button', NULL, NULL, NULL, 3, 'dept:delete', 1, 1, NOW(), NOW(), 1, 0),
    (22, 7, 'Tenant Add', 'button', NULL, NULL, NULL, 1, 'tenant:add', 1, 1, NOW(), NOW(), 1, 0),
    (23, 7, 'Tenant Edit', 'button', NULL, NULL, NULL, 2, 'tenant:edit', 1, 1, NOW(), NOW(), 1, 0),
    (24, 7, 'Tenant Delete', 'button', NULL, NULL, NULL, 3, 'tenant:delete', 1, 1, NOW(), NOW(), 1, 0),
    (25, 8, 'Dict Add', 'button', NULL, NULL, NULL, 1, 'dict:add', 1, 1, NOW(), NOW(), 1, 0),
    (26, 8, 'Dict Edit', 'button', NULL, NULL, NULL, 2, 'dict:edit', 1, 1, NOW(), NOW(), 1, 0),
    (27, 8, 'Dict Delete', 'button', NULL, NULL, NULL, 3, 'dict:delete', 1, 1, NOW(), NOW(), 1, 0),
    (28, 9, 'Config Add', 'button', NULL, NULL, NULL, 1, 'config:add', 1, 1, NOW(), NOW(), 1, 0),
    (29, 9, 'Config Edit', 'button', NULL, NULL, NULL, 2, 'config:edit', 1, 1, NOW(), NOW(), 1, 0),
    (30, 9, 'Config Delete', 'button', NULL, NULL, NULL, 3, 'config:delete', 1, 1, NOW(), NOW(), 1, 0);

INSERT INTO auth_sys_role_menu (id, role_id, menu_id, create_time, version, delete_flag) VALUES
    (1, 1, 1, NOW(), 1, 0), (2, 1, 2, NOW(), 1, 0), (3, 1, 3, NOW(), 1, 0), (4, 1, 4, NOW(), 1, 0),
    (5, 1, 5, NOW(), 1, 0), (6, 1, 6, NOW(), 1, 0), (7, 1, 7, NOW(), 1, 0), (8, 1, 8, NOW(), 1, 0),
    (9, 1, 9, NOW(), 1, 0), (10, 1, 10, NOW(), 1, 0), (11, 1, 11, NOW(), 1, 0), (12, 1, 12, NOW(), 1, 0),
    (13, 1, 13, NOW(), 1, 0), (14, 1, 14, NOW(), 1, 0), (15, 1, 15, NOW(), 1, 0), (16, 1, 16, NOW(), 1, 0),
    (17, 1, 17, NOW(), 1, 0), (18, 1, 18, NOW(), 1, 0), (19, 1, 19, NOW(), 1, 0), (20, 1, 20, NOW(), 1, 0),
    (21, 1, 21, NOW(), 1, 0), (22, 1, 22, NOW(), 1, 0), (23, 1, 23, NOW(), 1, 0), (24, 1, 24, NOW(), 1, 0),
    (25, 1, 25, NOW(), 1, 0), (26, 1, 26, NOW(), 1, 0), (27, 1, 27, NOW(), 1, 0), (28, 1, 28, NOW(), 1, 0),
    (29, 1, 29, NOW(), 1, 0), (30, 1, 30, NOW(), 1, 0);

INSERT INTO auth_sys_dict_type (id, dict_name, dict_type, status, create_time, update_time, version, delete_flag)
VALUES
    (1, 'Gender', 'sys_gender', 1, NOW(), NOW(), 1, 0),
    (2, 'Status', 'sys_status', 1, NOW(), NOW(), 1, 0),
    (3, 'Menu Type', 'sys_menu_type', 1, NOW(), NOW(), 1, 0);

INSERT INTO auth_sys_dict_item (id, dict_type_id, dict_label, dict_value, sort_order, status, create_time, update_time, version, delete_flag)
VALUES
    (1, 1, 'Male', 'male', 1, 1, NOW(), NOW(), 1, 0),
    (2, 1, 'Female', 'female', 2, 1, NOW(), NOW(), 1, 0),
    (3, 2, 'Enabled', '1', 1, 1, NOW(), NOW(), 1, 0),
    (4, 2, 'Disabled', '0', 2, 1, NOW(), NOW(), 1, 0),
    (5, 3, 'Directory', 'dir', 1, 1, NOW(), NOW(), 1, 0),
    (6, 3, 'Menu', 'menu', 2, 1, NOW(), NOW(), 1, 0),
    (7, 3, 'Button', 'button', 3, 1, NOW(), NOW(), 1, 0);

INSERT INTO auth_sys_config (id, config_name, config_key, config_value, config_type, create_time, update_time, version, delete_flag)
VALUES
    (1, 'System Name', 'sys.name', 'Template Admin', 'string', NOW(), NOW(), 1, 0),
    (2, 'Default Theme', 'sys.theme', 'light', 'string', NOW(), NOW(), 1, 0),
    (3, 'Page Size', 'sys.page.size', '10', 'number', NOW(), NOW(), 1, 0);
