-- 菜单权限同步迁移：将 auth_sys_menu 的 permission 字段与后端 handler 的 #[sa_check_permission] 宏完全对齐
-- 使用 ON CONFLICT (id) DO UPDATE 实现幂等 upsert，已存在的记录只更新业务字段（不重置审计字段）
-- 同步内容：
--   - 7 个模块（user/role/menu/dept/tenant/dict/config）的 list/add/edit/delete 权限点
--   - 服务监控 monitor:server
--   - 代码生成 tools:generator
-- 权限点与后端 handler 一一对应，共 30 个权限点

-- 顶级菜单
INSERT INTO auth_sys_menu (id, parent_id, menu_name, menu_type, path, component, icon, sort_order, permission, status, visible, create_time, update_time, version, delete_flag)
VALUES
    ('1876543210000000010', '0', '仪表盘', 'menu', '/dashboard', 'dashboard/index', 'DashboardOutlined', 1, NULL, 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000011', '0', '系统管理', 'dir', '/system', NULL, 'SettingOutlined', 2, NULL, 1, 1, NOW(), NOW(), 1, 0)
ON CONFLICT (id) DO UPDATE SET
    parent_id = EXCLUDED.parent_id,
    menu_name = EXCLUDED.menu_name,
    menu_type = EXCLUDED.menu_type,
    path = EXCLUDED.path,
    component = EXCLUDED.component,
    icon = EXCLUDED.icon,
    sort_order = EXCLUDED.sort_order,
    permission = EXCLUDED.permission,
    status = EXCLUDED.status,
    visible = EXCLUDED.visible,
    update_time = NOW();

-- 系统管理下的菜单项（menu 类型，对应前端路由）
INSERT INTO auth_sys_menu (id, parent_id, menu_name, menu_type, path, component, icon, sort_order, permission, status, visible, create_time, update_time, version, delete_flag)
VALUES
    ('1876543210000000012', '1876543210000000011', '用户管理', 'menu', '/system/user', 'system/user/index', 'UserOutlined', 1, 'user:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000013', '1876543210000000011', '角色管理', 'menu', '/system/role', 'system/role/index', 'TeamOutlined', 2, 'role:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000014', '1876543210000000011', '菜单管理', 'menu', '/system/menu', 'system/menu/index', 'MenuOutlined', 3, 'menu:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000015', '1876543210000000011', '部门管理', 'menu', '/system/dept', 'system/dept/index', 'ApartmentOutlined', 4, 'dept:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000016', '1876543210000000011', '租户管理', 'menu', '/system/tenant', 'system/tenant/index', 'BankOutlined', 5, 'tenant:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000017', '1876543210000000011', '字典管理', 'menu', '/system/dict', 'system/dict/index', 'BookOutlined', 6, 'dict:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000018', '1876543210000000011', '参数配置', 'menu', '/system/config', 'system/config/index', 'ToolOutlined', 7, 'config:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000040', '1876543210000000011', '服务监控', 'menu', '/system/monitor', 'system/monitor/index', 'DashboardOutlined', 8, 'monitor:server', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000041', '1876543210000000011', '代码生成', 'menu', '/system/generator', 'system/generator/index', 'CodeOutlined', 9, 'tools:generator', 1, 1, NOW(), NOW(), 1, 0)
ON CONFLICT (id) DO UPDATE SET
    parent_id = EXCLUDED.parent_id,
    menu_name = EXCLUDED.menu_name,
    menu_type = EXCLUDED.menu_type,
    path = EXCLUDED.path,
    component = EXCLUDED.component,
    icon = EXCLUDED.icon,
    sort_order = EXCLUDED.sort_order,
    permission = EXCLUDED.permission,
    status = EXCLUDED.status,
    visible = EXCLUDED.visible,
    update_time = NOW();

-- 按钮权限点（button 类型，对应 sa_check_permission 宏，不渲染菜单不生成路由）
INSERT INTO auth_sys_menu (id, parent_id, menu_name, menu_type, path, component, icon, sort_order, permission, status, visible, create_time, update_time, version, delete_flag)
VALUES
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
    -- 租户管理按钮（test_connection/create_database/init_database/create_tenant_full 复用 tenant:add）
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
    ('1876543210000000039', '1876543210000000018', '配置删除', 'button', NULL, NULL, NULL, 3, 'config:delete', 1, 1, NOW(), NOW(), 1, 0)
ON CONFLICT (id) DO UPDATE SET
    parent_id = EXCLUDED.parent_id,
    menu_name = EXCLUDED.menu_name,
    menu_type = EXCLUDED.menu_type,
    path = EXCLUDED.path,
    component = EXCLUDED.component,
    icon = EXCLUDED.icon,
    sort_order = EXCLUDED.sort_order,
    permission = EXCLUDED.permission,
    status = EXCLUDED.status,
    visible = EXCLUDED.visible,
    update_time = NOW();
