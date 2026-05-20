use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Statement};

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
                tenant_id VARCHAR(50)
            );

            CREATE TABLE auth_sys_role_menu (
                id BIGINT PRIMARY KEY,
                role_id BIGINT NOT NULL,
                menu_id BIGINT NOT NULL,
                create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
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
            VALUES (1, '默认租户', 'default', 1, NOW(), NOW(), 1, 0);

            INSERT INTO auth_sys_user (id, user_name, nick_name, pass_word, email, status, admin_flag, create_time, update_time, version, delete_flag)
            VALUES (1, 'admin', '超级管理员', 'admin123', 'admin@example.com', 1, 1, NOW(), NOW(), 1, 0);

            INSERT INTO auth_sys_role (id, parent_id, role_name, role_code, role_sort, status, create_time, update_time, version, delete_flag)
            VALUES (1, 0, '超级管理员', 'admin', 1, 1, NOW(), NOW(), 1, 0);

            INSERT INTO auth_sys_role (id, parent_id, role_name, role_code, role_sort, status, create_time, update_time, version, delete_flag)
            VALUES (2, 0, '普通用户', 'user', 2, 1, NOW(), NOW(), 1, 0);

            INSERT INTO auth_sys_user_role (id, user_id, role_id, create_time)
            VALUES (1, 1, 1, NOW());

            INSERT INTO auth_sys_menu (id, parent_id, menu_name, menu_type, path, component, icon, sort_order, permission, status, visible, create_time, update_time, version, delete_flag)
            VALUES
                (1, 0, '仪表盘', 'menu', '/dashboard', 'dashboard/index', 'DashboardOutlined', 1, NULL, 1, 1, NOW(), NOW(), 1, 0),
                (2, 0, '系统管理', 'dir', '/system', NULL, 'SettingOutlined', 2, NULL, 1, 1, NOW(), NOW(), 1, 0),
                (3, 2, '用户管理', 'menu', '/system/user', 'system/user/index', 'UserOutlined', 1, 'user:list', 1, 1, NOW(), NOW(), 1, 0),
                (4, 2, '角色管理', 'menu', '/system/role', 'system/role/index', 'TeamOutlined', 2, 'role:list', 1, 1, NOW(), NOW(), 1, 0),
                (5, 2, '菜单管理', 'menu', '/system/menu', 'system/menu/index', 'MenuOutlined', 3, 'menu:list', 1, 1, NOW(), NOW(), 1, 0),
                (6, 2, '部门管理', 'menu', '/system/dept', 'system/dept/index', 'ApartmentOutlined', 4, 'dept:list', 1, 1, NOW(), NOW(), 1, 0),
                (7, 2, '租户管理', 'menu', '/system/tenant', 'system/tenant/index', 'BankOutlined', 5, 'tenant:list', 1, 1, NOW(), NOW(), 1, 0),
                (8, 2, '字典管理', 'menu', '/system/dict', 'system/dict/index', 'BookOutlined', 6, 'dict:list', 1, 1, NOW(), NOW(), 1, 0),
                (9, 2, '参数配置', 'menu', '/system/config', 'system/config/index', 'ToolOutlined', 7, 'config:list', 1, 1, NOW(), NOW(), 1, 0),
                (10, 3, '用户新增', 'button', NULL, NULL, NULL, 1, 'user:add', 1, 1, NOW(), NOW(), 1, 0),
                (11, 3, '用户编辑', 'button', NULL, NULL, NULL, 2, 'user:edit', 1, 1, NOW(), NOW(), 1, 0),
                (12, 3, '用户删除', 'button', NULL, NULL, NULL, 3, 'user:delete', 1, 1, NOW(), NOW(), 1, 0),
                (13, 4, '角色新增', 'button', NULL, NULL, NULL, 1, 'role:add', 1, 1, NOW(), NOW(), 1, 0),
                (14, 4, '角色编辑', 'button', NULL, NULL, NULL, 2, 'role:edit', 1, 1, NOW(), NOW(), 1, 0),
                (15, 4, '角色删除', 'button', NULL, NULL, NULL, 3, 'role:delete', 1, 1, NOW(), NOW(), 1, 0),
                (16, 5, '菜单新增', 'button', NULL, NULL, NULL, 1, 'menu:add', 1, 1, NOW(), NOW(), 1, 0),
                (17, 5, '菜单编辑', 'button', NULL, NULL, NULL, 2, 'menu:edit', 1, 1, NOW(), NOW(), 1, 0),
                (18, 5, '菜单删除', 'button', NULL, NULL, NULL, 3, 'menu:delete', 1, 1, NOW(), NOW(), 1, 0),
                (19, 6, '部门新增', 'button', NULL, NULL, NULL, 1, 'dept:add', 1, 1, NOW(), NOW(), 1, 0),
                (20, 6, '部门编辑', 'button', NULL, NULL, NULL, 2, 'dept:edit', 1, 1, NOW(), NOW(), 1, 0),
                (21, 6, '部门删除', 'button', NULL, NULL, NULL, 3, 'dept:delete', 1, 1, NOW(), NOW(), 1, 0),
                (22, 7, '租户新增', 'button', NULL, NULL, NULL, 1, 'tenant:add', 1, 1, NOW(), NOW(), 1, 0),
                (23, 7, '租户编辑', 'button', NULL, NULL, NULL, 2, 'tenant:edit', 1, 1, NOW(), NOW(), 1, 0),
                (24, 7, '租户删除', 'button', NULL, NULL, NULL, 3, 'tenant:delete', 1, 1, NOW(), NOW(), 1, 0),
                (25, 8, '字典新增', 'button', NULL, NULL, NULL, 1, 'dict:add', 1, 1, NOW(), NOW(), 1, 0),
                (26, 8, '字典编辑', 'button', NULL, NULL, NULL, 2, 'dict:edit', 1, 1, NOW(), NOW(), 1, 0),
                (27, 8, '字典删除', 'button', NULL, NULL, NULL, 3, 'dict:delete', 1, 1, NOW(), NOW(), 1, 0),
                (28, 9, '配置新增', 'button', NULL, NULL, NULL, 1, 'config:add', 1, 1, NOW(), NOW(), 1, 0),
                (29, 9, '配置编辑', 'button', NULL, NULL, NULL, 2, 'config:edit', 1, 1, NOW(), NOW(), 1, 0),
                (30, 9, '配置删除', 'button', NULL, NULL, NULL, 3, 'config:delete', 1, 1, NOW(), NOW(), 1, 0);

            INSERT INTO auth_sys_role_menu (id, role_id, menu_id, create_time) VALUES
                (1, 1, 1, NOW()), (2, 1, 2, NOW()), (3, 1, 3, NOW()), (4, 1, 4, NOW()),
                (5, 1, 5, NOW()), (6, 1, 6, NOW()), (7, 1, 7, NOW()), (8, 1, 8, NOW()),
                (9, 1, 9, NOW()), (10, 1, 10, NOW()), (11, 1, 11, NOW()), (12, 1, 12, NOW()),
                (13, 1, 13, NOW()), (14, 1, 14, NOW()), (15, 1, 15, NOW()), (16, 1, 16, NOW()),
                (17, 1, 17, NOW()), (18, 1, 18, NOW()), (19, 1, 19, NOW()), (20, 1, 20, NOW()),
                (21, 1, 21, NOW()), (22, 1, 22, NOW()), (23, 1, 23, NOW()), (24, 1, 24, NOW()),
                (25, 1, 25, NOW()), (26, 1, 26, NOW()), (27, 1, 27, NOW()), (28, 1, 28, NOW()),
                (29, 1, 29, NOW()), (30, 1, 30, NOW());

            INSERT INTO auth_sys_dict_type (id, dict_name, dict_type, status, create_time, update_time, version, delete_flag)
            VALUES
                (1, '性别', 'sys_gender', 1, NOW(), NOW(), 1, 0),
                (2, '状态', 'sys_status', 1, NOW(), NOW(), 1, 0),
                (3, '菜单类型', 'sys_menu_type', 1, NOW(), NOW(), 1, 0);

            INSERT INTO auth_sys_dict_item (id, dict_type_id, dict_label, dict_value, sort_order, status, create_time, update_time, version, delete_flag)
            VALUES
                (1, 1, '男', 'male', 1, 1, NOW(), NOW(), 1, 0),
                (2, 1, '女', 'female', 2, 1, NOW(), NOW(), 1, 0),
                (3, 2, '启用', '1', 1, 1, NOW(), NOW(), 1, 0),
                (4, 2, '禁用', '0', 2, 1, NOW(), NOW(), 1, 0),
                (5, 3, '目录', 'dir', 1, 1, NOW(), NOW(), 1, 0),
                (6, 3, '菜单', 'menu', 2, 1, NOW(), NOW(), 1, 0),
                (7, 3, '按钮', 'button', 3, 1, NOW(), NOW(), 1, 0);

            INSERT INTO auth_sys_config (id, config_name, config_key, config_value, config_type, create_time, update_time, version, delete_flag)
            VALUES
                (1, '系统名称', 'sys.name', 'Template Admin', 'string', NOW(), NOW(), 1, 0),
                (2, '默认主题', 'sys.theme', 'light', 'string', NOW(), NOW(), 1, 0),
                (3, '分页大小', 'sys.page.size', '10', 'number', NOW(), NOW(), 1, 0);
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
