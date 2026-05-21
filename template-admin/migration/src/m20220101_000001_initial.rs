use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
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

            CREATE TABLE auth_sys_user_role (
                id VARCHAR(64) PRIMARY KEY,
                user_id VARCHAR(64) NOT NULL,
                role_id VARCHAR(64) NOT NULL,
                create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                tenant_id VARCHAR(50)
            );

            CREATE TABLE auth_sys_role_menu (
                id VARCHAR(64) PRIMARY KEY,
                role_id VARCHAR(64) NOT NULL,
                menu_id VARCHAR(64) NOT NULL,
                create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                tenant_id VARCHAR(50)
            );

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

            CREATE INDEX idx_user_role_user_id ON auth_sys_user_role(user_id);
            CREATE INDEX idx_user_role_role_id ON auth_sys_user_role(role_id);
            CREATE INDEX idx_role_menu_role_id ON auth_sys_role_menu(role_id);
            CREATE INDEX idx_role_menu_menu_id ON auth_sys_role_menu(menu_id);
            CREATE INDEX idx_dict_item_type_id ON auth_sys_dict_item(dict_type_id);

            INSERT INTO auth_sys_tenant (id, tenant_name, tenant_code, mode, status, create_time, update_time, version, delete_flag)
            VALUES ('1876543210000000001', '默认租户', 'default', 'table', 1, NOW(), NOW(), 1, 0);

            INSERT INTO auth_sys_user (id, user_name, nick_name, pass_word, email, status, admin_flag, create_time, update_time, version, delete_flag, tenant_id)
            VALUES ('1876543210000000001', 'admin', '超级管理员', 'admin123', 'admin@example.com', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001');

            INSERT INTO auth_sys_role (id, parent_id, role_name, role_code, role_sort, status, create_time, update_time, version, delete_flag, tenant_id)
            VALUES ('1876543210000000001', '0', '超级管理员', 'admin', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001');

            INSERT INTO auth_sys_role (id, parent_id, role_name, role_code, role_sort, status, create_time, update_time, version, delete_flag, tenant_id)
            VALUES ('1876543210000000002', '0', '普通用户', 'user', 2, 1, NOW(), NOW(), 1, 0, '1876543210000000001');

            INSERT INTO auth_sys_user_role (id, user_id, role_id, create_time, tenant_id)
            VALUES ('1876543210000000001', '1876543210000000001', '1876543210000000001', NOW(), '1876543210000000001');

            INSERT INTO auth_sys_menu (id, parent_id, menu_name, menu_type, path, component, icon, sort_order, permission, status, visible, create_time, update_time, version, delete_flag, tenant_id)
            VALUES
                ('1876543210000000001', '0', '仪表盘', 'menu', '/dashboard', 'dashboard/index', 'DashboardOutlined', 1, NULL, 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000002', '0', '系统管理', 'dir', '/system', NULL, 'SettingOutlined', 2, NULL, 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000003', '1876543210000000002', '用户管理', 'menu', '/system/user', 'system/user/index', 'UserOutlined', 1, 'user:list', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000004', '1876543210000000002', '角色管理', 'menu', '/system/role', 'system/role/index', 'TeamOutlined', 2, 'role:list', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000005', '1876543210000000002', '菜单管理', 'menu', '/system/menu', 'system/menu/index', 'MenuOutlined', 3, 'menu:list', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000006', '1876543210000000002', '部门管理', 'menu', '/system/dept', 'system/dept/index', 'ApartmentOutlined', 4, 'dept:list', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000007', '1876543210000000002', '租户管理', 'menu', '/system/tenant', 'system/tenant/index', 'BankOutlined', 5, 'tenant:list', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000008', '1876543210000000002', '字典管理', 'menu', '/system/dict', 'system/dict/index', 'BookOutlined', 6, 'dict:list', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000009', '1876543210000000002', '参数配置', 'menu', '/system/config', 'system/config/index', 'ToolOutlined', 7, 'config:list', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000010', '1876543210000000003', '用户新增', 'button', NULL, NULL, NULL, 1, 'user:add', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000011', '1876543210000000003', '用户编辑', 'button', NULL, NULL, NULL, 2, 'user:edit', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000012', '1876543210000000003', '用户删除', 'button', NULL, NULL, NULL, 3, 'user:delete', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000013', '1876543210000000004', '角色新增', 'button', NULL, NULL, NULL, 1, 'role:add', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000014', '1876543210000000004', '角色编辑', 'button', NULL, NULL, NULL, 2, 'role:edit', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000015', '1876543210000000004', '角色删除', 'button', NULL, NULL, NULL, 3, 'role:delete', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000016', '1876543210000000005', '菜单新增', 'button', NULL, NULL, NULL, 1, 'menu:add', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000017', '1876543210000000005', '菜单编辑', 'button', NULL, NULL, NULL, 2, 'menu:edit', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000018', '1876543210000000005', '菜单删除', 'button', NULL, NULL, NULL, 3, 'menu:delete', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000019', '1876543210000000006', '部门新增', 'button', NULL, NULL, NULL, 1, 'dept:add', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000020', '1876543210000000006', '部门编辑', 'button', NULL, NULL, NULL, 2, 'dept:edit', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000021', '1876543210000000006', '部门删除', 'button', NULL, NULL, NULL, 3, 'dept:delete', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000022', '1876543210000000007', '租户新增', 'button', NULL, NULL, NULL, 1, 'tenant:add', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000023', '1876543210000000007', '租户编辑', 'button', NULL, NULL, NULL, 2, 'tenant:edit', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000024', '1876543210000000007', '租户删除', 'button', NULL, NULL, NULL, 3, 'tenant:delete', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000025', '1876543210000000008', '字典新增', 'button', NULL, NULL, NULL, 1, 'dict:add', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000026', '1876543210000000008', '字典编辑', 'button', NULL, NULL, NULL, 2, 'dict:edit', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000027', '1876543210000000008', '字典删除', 'button', NULL, NULL, NULL, 3, 'dict:delete', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000028', '1876543210000000009', '配置新增', 'button', NULL, NULL, NULL, 1, 'config:add', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000029', '1876543210000000009', '配置编辑', 'button', NULL, NULL, NULL, 2, 'config:edit', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000030', '1876543210000000009', '配置删除', 'button', NULL, NULL, NULL, 3, 'config:delete', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001');

            INSERT INTO auth_sys_role_menu (id, role_id, menu_id, create_time, tenant_id) VALUES
                ('1876543210000000001', '1876543210000000001', '1876543210000000001', NOW(), '1876543210000000001'),
                ('1876543210000000002', '1876543210000000001', '1876543210000000002', NOW(), '1876543210000000001'),
                ('1876543210000000003', '1876543210000000001', '1876543210000000003', NOW(), '1876543210000000001'),
                ('1876543210000000004', '1876543210000000001', '1876543210000000004', NOW(), '1876543210000000001'),
                ('1876543210000000005', '1876543210000000001', '1876543210000000005', NOW(), '1876543210000000001'),
                ('1876543210000000006', '1876543210000000001', '1876543210000000006', NOW(), '1876543210000000001'),
                ('1876543210000000007', '1876543210000000001', '1876543210000000007', NOW(), '1876543210000000001'),
                ('1876543210000000008', '1876543210000000001', '1876543210000000008', NOW(), '1876543210000000001'),
                ('1876543210000000009', '1876543210000000001', '1876543210000000009', NOW(), '1876543210000000001'),
                ('1876543210000000010', '1876543210000000001', '1876543210000000010', NOW(), '1876543210000000001'),
                ('1876543210000000011', '1876543210000000001', '1876543210000000011', NOW(), '1876543210000000001'),
                ('1876543210000000012', '1876543210000000001', '1876543210000000012', NOW(), '1876543210000000001'),
                ('1876543210000000013', '1876543210000000001', '1876543210000000013', NOW(), '1876543210000000001'),
                ('1876543210000000014', '1876543210000000001', '1876543210000000014', NOW(), '1876543210000000001'),
                ('1876543210000000015', '1876543210000000001', '1876543210000000015', NOW(), '1876543210000000001'),
                ('1876543210000000016', '1876543210000000001', '1876543210000000016', NOW(), '1876543210000000001'),
                ('1876543210000000017', '1876543210000000001', '1876543210000000017', NOW(), '1876543210000000001'),
                ('1876543210000000018', '1876543210000000001', '1876543210000000018', NOW(), '1876543210000000001'),
                ('1876543210000000019', '1876543210000000001', '1876543210000000019', NOW(), '1876543210000000001'),
                ('1876543210000000020', '1876543210000000001', '1876543210000000020', NOW(), '1876543210000000001'),
                ('1876543210000000021', '1876543210000000001', '1876543210000000021', NOW(), '1876543210000000001'),
                ('1876543210000000022', '1876543210000000001', '1876543210000000022', NOW(), '1876543210000000001'),
                ('1876543210000000023', '1876543210000000001', '1876543210000000023', NOW(), '1876543210000000001'),
                ('1876543210000000024', '1876543210000000001', '1876543210000000024', NOW(), '1876543210000000001'),
                ('1876543210000000025', '1876543210000000001', '1876543210000000025', NOW(), '1876543210000000001'),
                ('1876543210000000026', '1876543210000000001', '1876543210000000026', NOW(), '1876543210000000001'),
                ('1876543210000000027', '1876543210000000001', '1876543210000000027', NOW(), '1876543210000000001'),
                ('1876543210000000028', '1876543210000000001', '1876543210000000028', NOW(), '1876543210000000001'),
                ('1876543210000000029', '1876543210000000001', '1876543210000000029', NOW(), '1876543210000000001'),
                ('1876543210000000030', '1876543210000000001', '1876543210000000030', NOW(), '1876543210000000001');

            INSERT INTO auth_sys_dict_type (id, dict_name, dict_type, status, create_time, update_time, version, delete_flag, tenant_id)
            VALUES
                ('1876543210000000001', '性别', 'sys_gender', 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000002', '状态', 'sys_status', 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000003', '菜单类型', 'sys_menu_type', 1, NOW(), NOW(), 1, 0, '1876543210000000001');

            INSERT INTO auth_sys_dict_item (id, dict_type_id, dict_label, dict_value, sort_order, status, create_time, update_time, version, delete_flag, tenant_id)
            VALUES
                ('1876543210000000001', '1876543210000000001', '男', 'male', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000002', '1876543210000000001', '女', 'female', 2, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000003', '1876543210000000002', '启用', '1', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000004', '1876543210000000002', '禁用', '0', 2, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000005', '1876543210000000003', '目录', 'dir', 1, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000006', '1876543210000000003', '菜单', 'menu', 2, 1, NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000007', '1876543210000000003', '按钮', 'button', 3, 1, NOW(), NOW(), 1, 0, '1876543210000000001');

            INSERT INTO auth_sys_config (id, config_name, config_key, config_value, config_type, create_time, update_time, version, delete_flag, tenant_id)
            VALUES
                ('1876543210000000001', '系统名称', 'sys.name', 'Template Admin', 'string', NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000002', '默认主题', 'sys.theme', 'light', 'string', NOW(), NOW(), 1, 0, '1876543210000000001'),
                ('1876543210000000003', '分页大小', 'sys.page.size', '10', 'number', NOW(), NOW(), 1, 0, '1876543210000000001');
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            DROP TABLE IF EXISTS auth_sys_config CASCADE;
            DROP TABLE IF EXISTS auth_sys_dict_item CASCADE;
            DROP TABLE IF EXISTS auth_sys_dict_type CASCADE;
            DROP TABLE IF EXISTS auth_sys_role_menu CASCADE;
            DROP TABLE IF EXISTS auth_sys_user_role CASCADE;
            DROP TABLE IF EXISTS auth_sys_menu CASCADE;
            DROP TABLE IF EXISTS auth_sys_role CASCADE;
            DROP TABLE IF EXISTS auth_sys_user CASCADE;
            DROP TABLE IF EXISTS auth_sys_dept CASCADE;
            DROP TABLE IF EXISTS auth_sys_tenant CASCADE;
            "#,
        )
        .await?;

        Ok(())
    }
}
