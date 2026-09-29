-- ============================================================
-- 系统管理补齐迁移（2026-09）：客户端 / 附件中心 / 通知中心 / 通知模板 / 导出中心
--
-- 幂等设计：CREATE TABLE / INDEX IF NOT EXISTS、ALTER ... ADD COLUMN IF NOT EXISTS、
-- 菜单与授权走 ON CONFLICT upsert —— 可重复执行，既适配全新库也适配已初始化的库。
-- 说明：支付中心（auth_sys_pay_order / auth_sys_pay_refund）不在本次迁移范围。
-- ============================================================

-- ------------------------------------------------------------
-- 0. 重置本迁移新建的表（与 initial.sql 的 drop + create 口径一致，保证可重复执行）
-- ------------------------------------------------------------
DROP TABLE IF EXISTS auth_sys_export_task CASCADE;
DROP TABLE IF EXISTS auth_sys_notification_template CASCADE;
DROP TABLE IF EXISTS auth_sys_notification CASCADE;
DROP TABLE IF EXISTS auth_sys_attachment CASCADE;
DROP TABLE IF EXISTS auth_sys_user_identity CASCADE;
DROP TABLE IF EXISTS auth_sys_client CASCADE;

-- ------------------------------------------------------------
-- 1. 客户端表：权限体系顶级维度（菜单与角色都挂在客户端下）
--    全局表（与 auth_sys_tenant 同类，不参与租户隔离）
-- ------------------------------------------------------------
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

INSERT INTO auth_sys_client (id, client_code, client_secret, client_name, client_type, home_path, sort_order, status, remark, create_time, update_time, version, delete_flag)
VALUES ('1876543210000000300', 'web-admin', 'web-admin-secret', '管理后台', 'web', '/dashboard', 1, 1, 'PC 管理端（template-web），默认密钥请在正式环境修改', NOW(), NOW(), 1, 0)
ON CONFLICT (id) DO NOTHING;

-- ------------------------------------------------------------
-- 2. 角色 / 菜单补 client_id 列（权限按客户端维度收敛）
-- ------------------------------------------------------------
ALTER TABLE auth_sys_role ADD COLUMN IF NOT EXISTS client_id VARCHAR(64) NOT NULL DEFAULT '1876543210000000300';
ALTER TABLE auth_sys_menu ADD COLUMN IF NOT EXISTS client_id VARCHAR(64) NOT NULL DEFAULT '1876543210000000300';
CREATE INDEX IF NOT EXISTS idx_sys_role_client ON auth_sys_role(client_id);
CREATE INDEX IF NOT EXISTS idx_sys_menu_client ON auth_sys_menu(client_id);

-- 角色编码唯一性按客户端收敛：同一编码可在不同客户端各建一个角色
ALTER TABLE auth_sys_role DROP CONSTRAINT IF EXISTS auth_sys_role_role_code_key;

-- ------------------------------------------------------------
-- 3. 第三方身份绑定表（一个账号可绑定多种第三方身份，互不覆盖）
--    微信类（wechat_mp / wechat）统一存 unionid，跨端识别依赖它
-- ------------------------------------------------------------
CREATE TABLE IF NOT EXISTS auth_sys_user_identity (
    id VARCHAR(64) PRIMARY KEY,
    user_id VARCHAR(64) NOT NULL,
    provider VARCHAR(50) NOT NULL,
    identity_value VARCHAR(200) NOT NULL,
    nick_name VARCHAR(100),
    status INT NOT NULL DEFAULT 1,
    create_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by VARCHAR(50),
    create_id VARCHAR(50),
    update_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_by VARCHAR(50),
    update_id VARCHAR(50),
    version INT NOT NULL DEFAULT 0,
    delete_flag INT NOT NULL DEFAULT 0,
    tenant_id VARCHAR(50)
);
CREATE INDEX IF NOT EXISTS idx_user_identity_user ON auth_sys_user_identity(user_id);

-- ------------------------------------------------------------
-- 4. 附件中心（所有业务文件/图片统一落库，多文件=同 biz_type+biz_id 的多行）
-- ------------------------------------------------------------
CREATE TABLE IF NOT EXISTS auth_sys_attachment (
    id            VARCHAR(32) PRIMARY KEY,
    category      VARCHAR(32) NOT NULL,
    biz_type      VARCHAR(64) NOT NULL,
    biz_id        VARCHAR(32),
    image_type    VARCHAR(16),
    sort          INT NOT NULL DEFAULT 0,
    caption       VARCHAR(255),
    original_name VARCHAR(255) NOT NULL,
    storage_path  VARCHAR(512) NOT NULL,
    mime_type     VARCHAR(128) NOT NULL,
    file_size     BIGINT NOT NULL DEFAULT 0,
    uploader_id   VARCHAR(64),
    status        VARCHAR(16) NOT NULL DEFAULT 'normal',
    remark        VARCHAR(500),
    create_time   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by     VARCHAR(64),
    create_id     VARCHAR(64),
    update_time   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_by     VARCHAR(64),
    update_id     VARCHAR(64),
    version       INT NOT NULL DEFAULT 0,
    delete_flag   INT NOT NULL DEFAULT 0,
    tenant_id     VARCHAR(50)
);
CREATE INDEX IF NOT EXISTS idx_auth_sys_attachment_cat ON auth_sys_attachment(category, biz_type, delete_flag);
CREATE INDEX IF NOT EXISTS idx_auth_sys_attachment_biz ON auth_sys_attachment(category, biz_type, biz_id, delete_flag);

-- ------------------------------------------------------------
-- 5. 通知中心：站内消息（支持站内信/邮件双通道，模板渲染后落库）
-- ------------------------------------------------------------
CREATE TABLE IF NOT EXISTS auth_sys_notification (
    id            VARCHAR(32) PRIMARY KEY,
    user_id       VARCHAR(32),
    notify_type   VARCHAR(32) NOT NULL,
    channel       VARCHAR(16) NOT NULL DEFAULT 'in_app',
    title         VARCHAR(255) NOT NULL,
    content       TEXT NOT NULL,
    template_code VARCHAR(64),
    ref_type      VARCHAR(32),
    ref_id        VARCHAR(64),
    read_flag     INT NOT NULL DEFAULT 0,
    read_at       TIMESTAMPTZ,
    send_status   VARCHAR(16) NOT NULL DEFAULT 'pending',
    send_error    TEXT,
    create_time   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by     VARCHAR(64)
);
CREATE INDEX IF NOT EXISTS idx_sys_notify_user ON auth_sys_notification(user_id, read_flag, create_time);
CREATE INDEX IF NOT EXISTS idx_sys_notify_type ON auth_sys_notification(notify_type, create_time);

-- ------------------------------------------------------------
-- 6. 通知模板：邮件 / 站内消息的标题与正文样式
--    正文中可用 ${varName} 占位，发送时由调用方传入变量 Map 渲染后落库 / 投递
-- ------------------------------------------------------------
CREATE TABLE IF NOT EXISTS auth_sys_notification_template (
    id               VARCHAR(64) PRIMARY KEY,
    template_code    VARCHAR(64) NOT NULL,
    template_name    VARCHAR(100) NOT NULL,
    notify_type      VARCHAR(32) NOT NULL DEFAULT 'system',
    channel          VARCHAR(16) NOT NULL DEFAULT 'in_app',
    title_template   VARCHAR(255),
    content_template TEXT NOT NULL,
    vars_hint        VARCHAR(500),
    status           INT NOT NULL DEFAULT 1,
    remark           VARCHAR(500),
    create_time      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by        VARCHAR(64),
    create_id        VARCHAR(64),
    update_time      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_by        VARCHAR(64),
    update_id        VARCHAR(64),
    version          INT NOT NULL DEFAULT 1,
    delete_flag      INT NOT NULL DEFAULT 0
);
CREATE UNIQUE INDEX IF NOT EXISTS uk_sys_notify_tpl ON auth_sys_notification_template(template_code, channel) WHERE delete_flag = 0;
CREATE INDEX IF NOT EXISTS idx_sys_notify_tpl_type ON auth_sys_notification_template(notify_type, status, delete_flag);

-- ------------------------------------------------------------
-- 7. 导出中心：通用异步导出任务台账（提交 → 后台生成 → 可下载）
-- ------------------------------------------------------------
CREATE TABLE IF NOT EXISTS auth_sys_export_task (
    id           VARCHAR(32) PRIMARY KEY,
    task_no      VARCHAR(64) NOT NULL,
    task_type    VARCHAR(16) NOT NULL DEFAULT 'user',
    query        VARCHAR(2000),
    status       VARCHAR(16) NOT NULL DEFAULT 'pending',
    total_rows   INT NOT NULL DEFAULT 0,
    success_rows INT NOT NULL DEFAULT 0,
    error_rows   INT NOT NULL DEFAULT 0,
    error_msg    VARCHAR(1000),
    file_path    VARCHAR(500),
    file_name    VARCHAR(255),
    file_size    BIGINT,
    create_time  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    create_by    VARCHAR(64),
    create_id    VARCHAR(64),
    update_time  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    update_by    VARCHAR(64),
    update_id    VARCHAR(64),
    version      INT NOT NULL DEFAULT 0,
    delete_flag  INT NOT NULL DEFAULT 0,
    tenant_id    VARCHAR(50)
);
CREATE INDEX IF NOT EXISTS idx_auth_sys_export_task ON auth_sys_export_task(create_by, delete_flag, create_time DESC);

-- ------------------------------------------------------------
-- 8. 菜单与按钮权限：附件管理 / 通知中心 / 通知模板 / 客户端管理 / 导出中心
--    以及 7 个模块的列表导出按钮（与后端 #[sa_check_permission] 一一对应）
-- ------------------------------------------------------------
INSERT INTO auth_sys_menu (id, parent_id, menu_name, menu_type, path, component, icon, sort_order, permission, status, visible, create_time, update_time, version, delete_flag)
VALUES
    ('1876543210000000042', '1876543210000000011', '附件管理', 'menu', '/system/attachment', 'system/attachment/index', 'PaperClipOutlined', 10, 'attachment:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000043', '1876543210000000011', '通知中心', 'menu', '/system/notification', 'system/notification/index', 'BellOutlined', 11, 'notification:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000130', '1876543210000000011', '通知模板', 'menu', '/system/notification-template', 'system/notification-template/index', 'FileTextOutlined', 13, 'notification:template:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000150', '1876543210000000011', '客户端管理', 'menu', '/system/client', 'system/client/index', 'CloudServerOutlined', 14, 'client:list', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000155', '1876543210000000011', '导出中心', 'menu', '/system/export', 'system/export/index', 'DownloadOutlined', 99, 'system:export:list', 1, 1, NOW(), NOW(), 1, 0),
    -- 附件管理按钮
    ('1876543210000000045', '1876543210000000042', '附件上传', 'button', NULL, NULL, NULL, 1, 'attachment:upload', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000046', '1876543210000000042', '附件预览', 'button', NULL, NULL, NULL, 2, 'attachment:read', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000047', '1876543210000000042', '附件删除', 'button', NULL, NULL, NULL, 3, 'attachment:delete', 1, 1, NOW(), NOW(), 1, 0),
    -- 通知中心按钮
    ('1876543210000000048', '1876543210000000043', '通知发送', 'button', NULL, NULL, NULL, 1, 'notification:send', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000049', '1876543210000000043', '通知广播', 'button', NULL, NULL, NULL, 2, 'notification:broadcast', 1, 1, NOW(), NOW(), 1, 0),
    -- 通知模板按钮
    ('1876543210000000131', '1876543210000000130', '模板新增', 'button', NULL, NULL, NULL, 1, 'notification:template:add', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000132', '1876543210000000130', '模板编辑', 'button', NULL, NULL, NULL, 2, 'notification:template:edit', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000133', '1876543210000000130', '模板删除', 'button', NULL, NULL, NULL, 3, 'notification:template:delete', 1, 1, NOW(), NOW(), 1, 0),
    -- 客户端管理按钮
    ('1876543210000000151', '1876543210000000150', '客户端新增', 'button', NULL, NULL, NULL, 1, 'client:add', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000152', '1876543210000000150', '客户端编辑', 'button', NULL, NULL, NULL, 2, 'client:edit', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000153', '1876543210000000150', '客户端删除', 'button', NULL, NULL, NULL, 3, 'client:delete', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000154', '1876543210000000150', '客户端导出', 'button', NULL, NULL, NULL, 4, 'client:export', 1, 1, NOW(), NOW(), 1, 0),
    -- 导出中心按钮
    ('1876543210000000090', '1876543210000000155', '提交导出', 'button', NULL, NULL, NULL, 1, 'system:export:create', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000091', '1876543210000000155', '下载导出文件', 'button', NULL, NULL, NULL, 2, 'system:export:download', 1, 1, NOW(), NOW(), 1, 0),
    -- 列表导出按钮（后端 GET /api/system/{users,roles,menus,depts,tenants,dicts,types,configs}/export）
    ('1876543210000000120', '1876543210000000012', '用户导出', 'button', NULL, NULL, NULL, 4, 'user:export', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000121', '1876543210000000013', '角色导出', 'button', NULL, NULL, NULL, 4, 'role:export', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000122', '1876543210000000014', '菜单导出', 'button', NULL, NULL, NULL, 4, 'menu:export', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000123', '1876543210000000015', '部门导出', 'button', NULL, NULL, NULL, 4, 'dept:export', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000124', '1876543210000000016', '租户导出', 'button', NULL, NULL, NULL, 4, 'tenant:export', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000125', '1876543210000000017', '字典导出', 'button', NULL, NULL, NULL, 4, 'dict:export', 1, 1, NOW(), NOW(), 1, 0),
    ('1876543210000000126', '1876543210000000018', '配置导出', 'button', NULL, NULL, NULL, 4, 'config:export', 1, 1, NOW(), NOW(), 1, 0)
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

-- 管理员角色（1876543210000000003）绑定新增菜单与按钮权限
INSERT INTO auth_sys_role_menu (id, role_id, menu_id, create_time, version, delete_flag) VALUES
    ('1876543210000000082', '1876543210000000003', '1876543210000000042', NOW(), 1, 0),
    ('1876543210000000083', '1876543210000000003', '1876543210000000043', NOW(), 1, 0),
    ('1876543210000000085', '1876543210000000003', '1876543210000000045', NOW(), 1, 0),
    ('1876543210000000086', '1876543210000000003', '1876543210000000046', NOW(), 1, 0),
    ('1876543210000000087', '1876543210000000003', '1876543210000000047', NOW(), 1, 0),
    ('1876543210000000088', '1876543210000000003', '1876543210000000048', NOW(), 1, 0),
    ('1876543210000000089', '1876543210000000003', '1876543210000000049', NOW(), 1, 0),
    ('1876543210000000630', '1876543210000000003', '1876543210000000120', NOW(), 1, 0),
    ('1876543210000000631', '1876543210000000003', '1876543210000000121', NOW(), 1, 0),
    ('1876543210000000632', '1876543210000000003', '1876543210000000122', NOW(), 1, 0),
    ('1876543210000000633', '1876543210000000003', '1876543210000000123', NOW(), 1, 0),
    ('1876543210000000634', '1876543210000000003', '1876543210000000124', NOW(), 1, 0),
    ('1876543210000000635', '1876543210000000003', '1876543210000000125', NOW(), 1, 0),
    ('1876543210000000636', '1876543210000000003', '1876543210000000126', NOW(), 1, 0),
    ('1876543210000000640', '1876543210000000003', '1876543210000000130', NOW(), 1, 0),
    ('1876543210000000641', '1876543210000000003', '1876543210000000131', NOW(), 1, 0),
    ('1876543210000000642', '1876543210000000003', '1876543210000000132', NOW(), 1, 0),
    ('1876543210000000643', '1876543210000000003', '1876543210000000133', NOW(), 1, 0),
    ('1876543210000000700', '1876543210000000003', '1876543210000000150', NOW(), 1, 0),
    ('1876543210000000701', '1876543210000000003', '1876543210000000151', NOW(), 1, 0),
    ('1876543210000000702', '1876543210000000003', '1876543210000000152', NOW(), 1, 0),
    ('1876543210000000703', '1876543210000000003', '1876543210000000153', NOW(), 1, 0),
    ('1876543210000000704', '1876543210000000003', '1876543210000000154', NOW(), 1, 0),
    ('1876543210000000092', '1876543210000000003', '1876543210000000090', NOW(), 1, 0),
    ('1876543210000000093', '1876543210000000003', '1876543210000000091', NOW(), 1, 0),
    ('1876543210000000705', '1876543210000000003', '1876543210000000155', NOW(), 1, 0)
ON CONFLICT (id) DO NOTHING;

-- ------------------------------------------------------------
-- 9. 注册身份字典（register_role）
--
-- 注册弹窗 / 完善资料页的「身份」下拉取自此字典：字典项的 dict_value 必须等于
-- 某条 auth_sys_role.role_code（且该角色属于默认客户端），后端按此做内连接过滤，
-- 配错的项不会出现在下拉里。「新增身份」必须同时加角色 + 字典项。
--
-- 模板默认只开放「普通用户」：超级管理员不开放自助注册。
-- ------------------------------------------------------------
INSERT INTO auth_sys_dict_type (id, dict_name, dict_type, status, create_time, update_time, version, delete_flag)
VALUES ('1876543210000000107', '注册身份', 'register_role', 1, NOW(), NOW(), 1, 0)
ON CONFLICT (id) DO UPDATE SET
    dict_name = EXCLUDED.dict_name,
    dict_type = EXCLUDED.dict_type,
    status = EXCLUDED.status,
    update_time = NOW();

INSERT INTO auth_sys_dict_item (id, dict_type_id, dict_label, dict_value, sort_order, status, create_time, update_time, version, delete_flag)
VALUES ('1876543210000000117', '1876543210000000107', '普通用户', 'user', 1, 1, NOW(), NOW(), 1, 0)
ON CONFLICT (id) DO UPDATE SET
    dict_type_id = EXCLUDED.dict_type_id,
    dict_label = EXCLUDED.dict_label,
    dict_value = EXCLUDED.dict_value,
    sort_order = EXCLUDED.sort_order,
    status = EXCLUDED.status,
    update_time = NOW();

-- ------------------------------------------------------------
-- 10. 通知模板初始数据（邮件验证码）
-- ------------------------------------------------------------
INSERT INTO auth_sys_notification_template (id, template_code, template_name, notify_type, channel, title_template, content_template, vars_hint, status, remark, create_time, update_time, version, delete_flag)
VALUES
    ('1876543210000000900', 'email_code_register', '注册验证码', 'system', 'email', '【Template Admin】注册验证码', E'您好：\n\n您正在注册账号，本次验证码为：${code}\n验证码 10 分钟内有效，请尽快填写。\n若并非您本人操作，请忽略本邮件。\n\n（本邮件由系统自动发送，请勿直接回复）', 'code', 1, '注册邮箱验证码', NOW(), NOW(), 1, 0),
    ('1876543210000000901', 'email_code_reset', '重置密码验证码', 'system', 'email', '【Template Admin】重置密码验证码', E'您好：\n\n您正在重置密码，本次验证码为：${code}\n验证码 10 分钟内有效，请尽快填写。\n若并非您本人操作，请忽略本邮件。\n\n（本邮件由系统自动发送，请勿直接回复）', 'code', 1, '找回密码邮箱验证码', NOW(), NOW(), 1, 0),
    ('1876543210000000902', 'email_code_login', '登录验证码', 'system', 'email', '【Template Admin】登录验证码', E'您好：\n\n您正在登录账号，本次验证码为：${code}\n验证码 10 分钟内有效，请尽快填写。\n若并非您本人操作，请忽略本邮件。\n\n（本邮件由系统自动发送，请勿直接回复）', 'code', 1, '邮箱验证码登录', NOW(), NOW(), 1, 0),
    ('1876543210000000911', 'email_code_change_password', '修改密码验证码', 'system', 'email', '【Template Admin】修改密码验证码', E'您好：\n\n您正在修改账号密码，本次验证码为：${code}\n验证码 10 分钟内有效，请尽快填写。\n若并非您本人操作，请忽略本邮件。\n\n（本邮件由系统自动发送，请勿直接回复）', 'code', 1, '账户管理-修改密码（验证码发到当前绑定邮箱）', NOW(), NOW(), 1, 0),
    ('1876543210000000912', 'email_code_change_email', '更换邮箱验证码', 'system', 'email', '【Template Admin】更换邮箱验证码', E'您好：\n\n您正在更换账号绑定的邮箱，本次验证码为：${code}\n验证码 10 分钟内有效，请尽快填写。\n若并非您本人操作，请忽略本邮件。\n\n（本邮件由系统自动发送，请勿直接回复）', 'code', 1, '账户管理-更换绑定邮箱（验证码发到新邮箱）', NOW(), NOW(), 1, 0)
ON CONFLICT (id) DO NOTHING;

-- ------------------------------------------------------------
-- 11. 种子数据补租户归属（表隔离模式必需）
--
-- 背景：`summer-sea-orm-ext-tenant.mode = "table"` 下，租户域实体（带 `#[sea_orm_ext(TENANT)]`）
-- 的查询会自动追加 `WHERE tenant_id = <当前租户>`；而登录后租户上下文来自 token 的
-- `tenantId`（取自 `auth_sys_user.tenant_id`）。
--
-- 种子数据的 tenant_id 为空会同时踩两个坑：
--   1. 登录 token 拿不到 tenantId → 所有租户域查询过滤成 SQL NULL → 列表全空；
--   2. 即便 token 有租户，`tenant_id IS NULL` 的行也匹配不上，同样不可见。
--
-- 因此这里把种子数据统一归属到默认租户（与 `default_tenant_id` 一致）。
-- 幂等：仅在 tenant_id 为空时回填，重复执行安全。
-- ------------------------------------------------------------
UPDATE auth_sys_user SET tenant_id = '1876543210000000001' WHERE tenant_id IS NULL;
UPDATE auth_sys_role SET tenant_id = '1876543210000000001' WHERE tenant_id IS NULL;
UPDATE auth_sys_menu SET tenant_id = '1876543210000000001' WHERE tenant_id IS NULL;
UPDATE auth_sys_user_role SET tenant_id = '1876543210000000001' WHERE tenant_id IS NULL;
UPDATE auth_sys_role_menu SET tenant_id = '1876543210000000001' WHERE tenant_id IS NULL;
UPDATE auth_sys_dept SET tenant_id = '1876543210000000001' WHERE tenant_id IS NULL;
UPDATE auth_sys_dict_type SET tenant_id = '1876543210000000001' WHERE tenant_id IS NULL;
UPDATE auth_sys_dict_item SET tenant_id = '1876543210000000001' WHERE tenant_id IS NULL;
UPDATE auth_sys_config SET tenant_id = '1876543210000000001' WHERE tenant_id IS NULL;
UPDATE auth_sys_tenant_user SET tenant_id = '1876543210000000001' WHERE tenant_id IS NULL;
UPDATE auth_sys_attachment SET tenant_id = '1876543210000000001' WHERE tenant_id IS NULL;
UPDATE auth_sys_export_task SET tenant_id = '1876543210000000001' WHERE tenant_id IS NULL;
UPDATE auth_sys_user_identity SET tenant_id = '1876543210000000001' WHERE tenant_id IS NULL;