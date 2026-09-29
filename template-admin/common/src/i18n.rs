//! 轻量国际化：按请求语言（Accept-Language / x-locale）翻译面向用户的业务消息
//!
//! 消息规约：`@<key>` 或 `@<key>:<arg1>|<arg2>`
//! - 不以 `@` 开头的消息 = 普通文本，原样透传（兼容存量消息，未迁移不受影响）；
//! - 以 `@` 开头 = 翻译键，查双语消息表，模板中的 `{0}`/`{1}` 依次替换参数；
//! - 未知 key 原样透传（渐进迁移时保持可运行）。
//! 翻译在响应改写中间件里按请求头语言执行，不依赖框架上下文。

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Zh,
    En,
}

impl Language {
    /// 从 Accept-Language 头解析语言；不识别/缺失一律按中文处理
    pub fn from_accept_language(header: Option<&str>) -> Language {
        let h = header.unwrap_or("").to_lowercase();
        if h.starts_with("en") || h.starts_with("en-") || h.contains("en-us") || h.contains("en-gb") {
            Language::En
        } else {
            Language::Zh
        }
    }
}

/// 双语消息表：key → (中文, 英文)
/// 迁移存量消息时在此补条目，模板参数用 {0}/{1} 占位
fn lookup(key: &str) -> Option<(&'static str, &'static str)> {
    let pair = match key {
        // ---- 通用 ----
        "unauthorized" => ("未登录或登录已过期", "Not logged in or session expired"),
        "forbidden" => ("无权限访问", "Access denied"),
        "bad_request" => ("请求参数错误", "Bad request"),
        "not_found" => ("资源不存在", "Resource not found"),
        "internal_error" => ("服务器内部错误", "Internal server error"),
        "deleted_ok" => ("删除成功", "Deleted successfully"),
        "created_ok" => ("创建成功", "Created successfully"),
        "updated_ok" => ("更新成功", "Updated successfully"),
        "saved_ok" => ("保存成功", "Saved successfully"),
        "operated_ok" => ("操作成功", "Operation succeeded"),
        "operate_failed" => ("操作失败，请重试", "Operation failed, please retry"),
        "load_failed" => ("加载失败", "Failed to load"),
        "record_not_found" => ("记录不存在", "Record not found"),
        "status_updated_ok" => ("状态更新成功", "Status updated successfully"),
        "state_updated_ok" => ("状态已更新", "Status updated"),
        "assigned_ok" => ("分配成功", "Assigned successfully"),
        "sent_ok" => ("发送成功", "Sent successfully"),
        "sent_count_ok" => ("已发送 {0} 条", "{0} sent"),
        "read_ok" => ("已读", "Read"),
        "all_read_ok" => ("全部已读", "All read"),
        "broadcast_ok" => ("广播成功", "Broadcasted successfully"),
        "enabled_ok" => ("已启用", "Enabled"),
        "disabled_ok" => ("已禁用", "Disabled"),
        "handled_ok" => ("已处置", "Handled"),
        "audit_done_ok" => ("审核完成", "Review completed"),
        "processed_ok" => ("处理完成", "Processed"),
        "code_required" => ("授权码不能为空", "Authorization code is required"),
        "not_logged_in" => ("未登录: {0}", "Not logged in: {0}"),

        // ---- 系统：用户 / 租户 / 部门 / 配置 / 字典 ----
        "user_not_found" => ("用户不存在", "User not found"),
        "username_taken" => ("用户名已存在", "Username already exists"),
        "tenant_not_found" => ("租户不存在", "Tenant not found"),
        "tenant_expired" => ("租户已过期", "Tenant has expired"),
        "tenant_code_exists" => ("租户编码已存在", "Tenant code already exists"),
        "tenant_code_required" => ("租户编码不能为空", "Tenant code is required"),
        "dept_not_found" => ("部门不存在", "Department not found"),
        "config_not_found" => ("配置不存在", "Config not found"),
        "config_key_exists" => ("配置键已存在", "Config key already exists"),
        "dict_type_not_found" => ("字典类型不存在", "Dictionary type not found"),
        "dict_type_exists" => ("字典类型已存在", "Dictionary type already exists"),
        "dict_type_not_found_code" => ("字典类型 {0} 不存在", "Dictionary type {0} not found"),
        "dict_item_not_found" => ("字典项不存在", "Dictionary item not found"),
        "default_tenant_not_configured" => ("未配置默认租户 ID", "Default tenant ID is not configured"),
        "default_tenant_not_found" => ("默认租户不存在", "Default tenant not found"),

        // ---- 系统：数据库 / 租户建库 ----
        "db_connect_failed" => ("连接数据库服务器失败: {0}", "Failed to connect to database server: {0}"),
        "db_connect_test_failed" => ("连接测试失败: {0}", "Connection test failed: {0}"),
        "connect_failed" => ("连接失败: {0}", "Connection failed: {0}"),
        "sqlite_no_create_db" => ("SQLite 不支持创建数据库", "SQLite does not support creating databases"),
        "db_create_failed" => ("创建数据库失败: {0}", "Failed to create database: {0}"),
        "tenant_db_connect_failed" => ("连接租户数据库失败: {0}", "Failed to connect to tenant database: {0}"),
        "tenant_schema_init_failed" => ("初始化Schema失败: {0}", "Failed to initialize schema: {0}"),
        "tenant_sql_init_failed" => ("执行初始化SQL失败: {0}", "Failed to execute initialization SQL: {0}"),

        // ---- 系统：附件 ----
        "attachment_not_found" => ("附件不存在", "Attachment not found"),
        "file_type_unsupported" => ("不支持的文件类型", "Unsupported file type"),
        "file_too_large_50mb" => ("文件大小超过50MB限制", "File size exceeds the 50MB limit"),
        "file_empty" => ("文件内容为空", "File content is empty"),
        "attachment_biz_type_mismatch" => ("附件与业务类型不匹配，无权访问", "Attachment does not match the business type, access denied"),
        "attachment_biz_id_mismatch" => ("附件与业务对象不匹配，无权访问", "Attachment does not match the business object, access denied"),

        // ---- 系统：登录 / 认证 ----
        "bad_credentials" => ("用户名或密码错误", "Incorrect username or password"),
        "username_required" => ("用户名不能为空", "Username is required"),
        "email_required" => ("邮箱不能为空", "Email is required"),
        "email_or_code_required" => ("邮箱或验证码不能为空", "Email or verification code is required"),
        "email_code_required" => ("邮箱验证码不能为空", "Email verification code is required"),
        "email_code_invalid" => ("邮箱验证码错误或已过期", "Email verification code is invalid or expired"),
        "bad_email_credentials" => ("邮箱或密码错误", "Incorrect email or password"),
        "wrong_password" => ("密码错误", "Incorrect password"),
        "account_disabled" => ("账号已被禁用", "Account has been disabled"),
        "account_not_found" => ("账号不存在", "Account does not exist"),
        "account_not_bound" => ("该账号未绑定", "This account is not bound"),
        "wechat_bound_other" => ("该微信已绑定其他账号", "This WeChat account is already bound to another account"),
        "login_failed" => ("登录失败: {0}", "Login failed: {0}"),
        "token_info_failed" => ("获取令牌信息失败: {0}", "Failed to get token info: {0}"),
        "refresh_token_failed" => ("生成刷新令牌失败: {0}", "Failed to generate refresh token: {0}"),
        "password_hash_failed" => ("密码处理失败: {0}", "Password processing failed: {0}"),
        "two_factor_required" => ("需要两步验证", "Two-factor verification required"),
        "wxapp_not_configured" => ("微信小程序配置未设置", "WeChat Mini Program is not configured"),
        "wxapp_api_failed" => ("微信小程序接口调用失败: {0}", "WeChat Mini Program API call failed: {0}"),
        "wxapp_parse_failed" => ("解析微信小程序响应失败: {0}", "Failed to parse WeChat Mini Program response: {0}"),
        "wxapp_login_failed" => ("微信小程序登录失败: {0}", "WeChat Mini Program login failed: {0}"),
        "openid_failed" => ("获取openid失败", "Failed to get openid"),
        "wechat_autoregister_failed" => ("微信自动注册失败，请稍后重试", "WeChat auto-registration failed, please retry later"),
        "account_profile_incomplete" => ("该账号尚未完善信息，请使用微信登录后完善", "This account has not completed its profile, please sign in with WeChat to complete it"),
        "profile_already_completed" => ("账号信息已完善，无需重复提交", "Profile already completed"),
        "username_invalid" => ("用户名需为 3-50 位字母、数字或下划线", "Username must be 3-50 characters of letters, digits or underscores"),
        "password_invalid" => ("密码长度需为 8-64 位", "Password length must be 8-64 characters"),
        "phone_invalid" => ("手机号格式不正确", "Invalid phone number format"),
        "nick_name_too_long" => ("昵称长度不能超过 30 个字符", "Nickname must not exceed 30 characters"),
        "role_not_selectable" => ("所选身份不可用，请重新选择", "The selected role is not available, please choose again"),
        "qr_state_invalid" => ("二维码已失效，请重新扫码", "QR code has expired, please scan again"),
        "wechat_open_not_configured" => ("微信开放平台配置未设置，暂不支持扫码登录", "WeChat Open Platform is not configured, QR login unavailable"),
        "wechat_already_bound" => ("当前账号已绑定微信，请先解绑", "This account already has a WeChat bound, please unbind it first"),
        "not_bound" => ("该账号未绑定该第三方账号", "This third-party account is not bound"),
        "email_not_bound" => ("账号未绑定邮箱，无法通过邮箱验证码认证", "No email is bound to this account"),
        "email_unchanged" => ("新邮箱与当前邮箱相同，无需更换", "The new email is the same as the current one"),
        "password_changed" => ("密码修改成功", "Password changed successfully"),

        // ---- 系统：验证码 / 邮箱 ----
        "captcha_service_unavailable" => ("验证码服务暂不可用，请稍后重试", "Captcha service unavailable, please retry later"),
        "captcha_service_down" => ("验证码服务暂不可用", "Captcha service is temporarily unavailable"),
        "captcha_too_frequent" => ("获取图形验证码过于频繁，请稍后再试", "Too many captcha requests, please retry later"),
        "captcha_daily_limit" => ("今日图形验证码获取次数已达上限", "Daily captcha request limit reached"),
        "captcha_required" => ("请先完成图形验证码", "Please complete the captcha first"),
        "captcha_invalid" => ("图形验证码错误或已过期", "Captcha is incorrect or expired"),
        "captcha_not_found" => ("图形验证码不存在或已过期", "Captcha does not exist or has expired"),
        "captcha_scene_invalid" => ("验证码场景非法", "Invalid captcha scene"),
        "email_invalid" => ("邮箱格式不正确", "Invalid email format"),
        "send_too_frequent" => ("发送过于频繁，请 60 秒后重试", "Too many requests, please retry after 60 seconds"),
        "email_code_daily_limit" => ("今日验证码发送次数已达上限", "Daily verification code limit reached"),
        "mail_from_invalid" => ("发件邮箱格式错误: {0}", "Invalid sender email: {0}"),
        "mail_to_invalid" => ("收件邮箱格式错误: {0}", "Invalid recipient email: {0}"),
        "mail_build_failed" => ("构造邮件失败: {0}", "Failed to build email: {0}"),
        "mail_send_failed" => ("邮件发送失败: {0}", "Failed to send email: {0}"),

        // ---- 系统：通知 / 支付 ----
        "notif_recipient_required" => ("按模板发送时必须指定接收用户", "Recipients are required when sending by template"),
        "notif_content_required" => ("标题与正文不能为空", "Title and content are required"),
        "pay_provider_unsupported" => ("仅支持 wechat/alipay", "Only wechat/alipay are supported"),
        "prepay_failed" => ("预下单失败: {0}", "Prepay failed: {0}"),
        "order_not_found" => ("订单不存在", "Order not found"),
        "amount_positive" => ("金额必须大于0", "Amount must be greater than 0"),
        "processor_not_initialized" => ("通知发送器尚未初始化", "Notification sender is not initialized"),

        // ---- 注册 ----
        "register_closed" => ("平台暂未开放注册", "Registration is not open yet"),
        "username_format_invalid" => ("用户名需为 3~30 位字母、数字、下划线或中划线", "Username must be 3-30 characters of letters, digits, underscore or hyphen"),
        "password_format_invalid" => ("密码需为 8~64 位且同时包含字母与数字", "Password must be 8-64 characters and contain both letters and digits"),
        "email_taken" => ("邮箱已被注册", "Email is already registered"),
        "email_not_registered" => ("该邮箱未注册", "This email is not registered"),
        "invite_code_invalid" => ("邀请码无效", "Invalid invite code"),
        "invite_code_conflict" => ("邀请码生成冲突，请重试", "Invite code generation conflict, please retry"),
        "captcha_sent_ok" => ("验证码已发送", "Verification code sent"),
        "password_reset_ok" => ("密码已重置，请重新登录", "Password reset, please log in again"),

        // ---- relay：渠道 / 模型 ----
        "channel_not_found" => ("渠道不存在", "Channel not found"),
        "channel_name_url_key_required" => ("名称、上游地址、密钥均不能为空", "Name, upstream URL and key are all required"),
        "channel_key_encrypt_failed" => ("渠道密钥加密失败: {0}", "Failed to encrypt channel key: {0}"),
        "channel_key_decrypt_failed" => ("渠道密钥解密失败: {0}", "Failed to decrypt channel key: {0}"),
        "channel_model_not_found" => ("渠道模型不存在", "Channel model not found"),
        "channel_model_exists" => ("该渠道已存在此模型", "This model already exists on the channel"),
        "channel_model_hijack" => ("存在不属于该渠道的模型记录", "Some models do not belong to this channel"),
        "model_message_required" => ("模型与消息内容不能为空", "Model and message content are required"),
        "upstream_subscribe_failed" => ("请求上游订阅信息失败: {0}", "Failed to request upstream subscription: {0}"),
        "subscribe_parse_failed" => ("解析订阅响应失败: {0}", "Failed to parse subscription response: {0}"),
        "upstream_usage_failed" => ("请求上游用量失败: {0}", "Failed to request upstream usage: {0}"),
        "balance_convert_failed" => ("余额数值转换失败", "Failed to convert balance value"),
        "upstream_balance_failed" => ("请求 DeepSeek 余额失败: {0}", "Failed to request DeepSeek balance: {0}"),
        "http_client_build_failed" => ("HTTP 客户端构建失败: {0}", "Failed to build HTTP client: {0}"),
        "ratio_negative" => ("group.ratio 不能为负", "group.ratio cannot be negative"),

        // ---- relay：API Key ----
        "key_name_required" => ("Key 名称不能为空", "Key name is required"),
        "key_max_reached" => ("每个用户最多创建 {0} 个 Key", "Each user can create at most {0} keys"),
        "key_name_taken" => ("同名 Key 已存在", "A key with the same name already exists"),
        "key_gen_conflict" => ("Key 生成冲突，请重试", "Key generation conflict, please retry"),
        "key_team_billing_forbidden" => ("仅团队成员可将 Key 绑定该团队计费", "Only team members can bind a key to team billing"),
        "key_not_found" => ("Key 不存在", "Key not found"),
        "expire_format_invalid" => ("过期时间格式应为 YYYY-MM-DD HH:mm:ss 或 YYYY-MM-DD", "Expiry time format should be YYYY-MM-DD HH:mm:ss or YYYY-MM-DD"),

        // ---- relay：实名认证 ----
        "kyc_name_format_invalid" => ("姓名与身份证号格式不正确", "Invalid name or ID number format"),
        "kyc_pending_exists" => ("已有待审核的实名申请", "A KYC application is pending review"),
        "kyc_already_done" => ("已完成实名认证，无需重复提交", "Already verified, no need to resubmit"),
        "kyc_id_encrypt_failed" => ("证件号加密失败: {0}", "Failed to encrypt ID number: {0}"),
        "kyc_not_found" => ("认证记录不存在", "Verification record not found"),
        "kyc_already_processed" => ("该申请已处理", "This application has already been processed"),

        // ---- relay：媒体缓冲 ----
        "media_mkdir_failed" => ("创建临时目录失败: {0}", "Failed to create temp directory: {0}"),
        "media_client_failed" => ("创建HTTP客户端失败: {0}", "Failed to create HTTP client: {0}"),
        "media_download_failed" => ("下载请求失败: {0}", "Download request failed: {0}"),
        "media_upstream_status" => ("上游返回 {0}", "Upstream returned {0}"),
        "media_chunk_failed" => ("下载数据块失败: {0}", "Failed to download data chunk: {0}"),
        "media_write_failed" => ("写入分片失败: {0}", "Failed to write shard: {0}"),
        "media_merge_file_failed" => ("创建合并文件失败: {0}", "Failed to create merged file: {0}"),
        "media_read_failed" => ("读取分片失败: {0}", "Failed to read shard: {0}"),
        "media_merge_write_failed" => ("合并写入失败: {0}", "Failed to write merged data: {0}"),

        // ---- 采购：工厂 / 权限 ----
        "factory_not_found" => ("工厂不存在", "Factory not found"),
        "factory_name_required" => ("工厂名称不能为空", "Factory name is required"),
        "factory_status_invalid" => ("状态仅允许 enabled/disabled", "Status must be enabled or disabled"),
        "user_id_required" => ("用户 id 不能为空", "User id is required"),
        "user_binding_exists" => ("该用户已绑定此工厂", "This user is already bound to this factory"),

        // ---- 采购：选品 / 工作流 ----
        "product_name_required" => ("品名不能为空", "Product name is required"),
        "factory_required" => ("工厂不能为空", "Factory is required"),
        "product_edit_forbidden" => ("当前状态「{0}」不可编辑（仅品类池/确认选品状态可编辑）", "Current status \"{0}\" cannot be edited (only pool/confirmed can be edited)"),
        "product_delete_forbidden" => ("当前状态「{0}」不可删除（仅品类池可删除，流转中的选品请走「放弃」）", "Current status \"{0}\" cannot be deleted (only pool items can be deleted; use \"abandon\" for in-progress items)"),
        "illegal_transition" => ("非法状态迁移: {0} -> {1}", "Illegal status transition: {0} -> {1}"),
        "abandon_reason_required" => ("放弃选品必须填写原因", "Reason is required to abandon an item"),
        "scope_forbidden" => ("无权访问该工厂数据（数据范围限制）", "No access to this factory's data (data scope restriction)"),
        "product_not_found" => ("选品不存在", "Product not found"),
        "product_status_forbidden" => ("当前状态「{0}」不可执行该操作", "Current status \"{0}\" does not allow this operation"),

        // ---- 采购：物流 / 包装 / 出货 ----
        "logistics_not_found" => ("物流记录不存在", "Logistics record not found"),
        "logistics_status_invalid" => ("状态仅允许 {0}", "Status must be one of: {0}"),
        "logistics_add_forbidden" => ("当前状态「{0}」不可录入物流（仅在确认打样/包装大货阶段可录入打样物流）", "Logistics cannot be added in status \"{0}\" (only in sampling/packaged stage)"),
        "shipment_not_found" => ("出货单不存在", "Shipment not found"),
        "shipment_create_forbidden" => ("当前状态「{0}」不可确认出货（仅打样通过后的包装/大货状态可出货）", "Shipment cannot be confirmed in status \"{0}\" (only packaged items can ship)"),
        "packaging_not_found" => ("包装规格不存在", "Packaging spec not found"),
        "packaging_status_forbidden" => ("当前状态「{0}」不可录入包装规格（仅打样通过后的包装/大货状态可录入）", "Packaging cannot be added in status \"{0}\" (only packaged items can have packaging)"),
        "packaging_confirmed_frozen" => ("已确认的包装规格不可修改，请新增新版本", "Confirmed packaging specs cannot be modified; create a new version"),
        "packaging_change_product_forbidden" => ("包装规格不允许更换选品，请新增新版本", "Packaging spec cannot switch products; create a new version"),
        "packaging_confirmed_undeletable" => ("已确认的包装规格不可删除", "Confirmed packaging specs cannot be deleted"),

        // ---- 采购：导入 ----
        "excel_missing_column" => ("Excel 缺少必填列「{0}」", "Excel is missing required column \"{0}\""),
        "excel_parse_failed" => ("Excel 解析失败: {0}", "Failed to parse Excel: {0}"),
        "excel_no_sheet" => ("Excel 文件无工作表", "Excel file has no worksheet"),
        "excel_read_failed" => ("读取工作表失败: {0}", "Failed to read worksheet: {0}"),
        "excel_missing_header" => ("Excel 缺少表头行", "Excel is missing the header row"),
        "excel_generate_failed" => ("Excel 生成失败: {0}", "Failed to generate Excel: {0}"),
        "zip_extract_failed" => ("zip 解包失败: {0}", "Failed to extract zip: {0}"),
        "zip_read_failed" => ("读取 zip 条目失败: {0}", "Failed to read zip entry: {0}"),
        "zip_image_failed" => ("读取 zip 图片失败: {0}", "Failed to read zip image: {0}"),
        "upload_read_failed" => ("读取上传内容失败: {0}", "Failed to read upload content: {0}"),
        "upload_too_large" => ("上传内容超过大小上限", "Upload exceeds the size limit"),

        // ---- 补充：认证 / 存储 / 任务 / 安全 ----
        "refresh_token_invalid" => ("刷新令牌无效或已过期: {0}", "Refresh token is invalid or expired: {0}"),
        "logout_failed" => ("登出失败: {0}", "Logout failed: {0}"),
        "mail_template_not_found" => ("邮件模板 {0} 不存在或已停用", "Email template {0} does not exist or is disabled"),
        "secret_encrypt_failed" => ("密钥加密失败: {0}", "Failed to encrypt secret: {0}"),
        "secret_decrypt_failed" => ("密钥解密失败: {0}", "Failed to decrypt secret: {0}"),
        "storage_mkdir_failed" => ("创建上传目录失败: {0}", "Failed to create upload directory: {0}"),
        "storage_write_failed" => ("文件写入失败: {0}", "Failed to write file: {0}"),
        "storage_delete_failed" => ("本地文件删除失败(key={0}): {1}", "Failed to delete local file (key={0}): {1}"),
        "storage_list_failed" => ("列举本地文件失败: {0}", "Failed to list local files: {0}"),
        "storage_read_dir_failed" => ("读取目录项失败: {0}", "Failed to read directory entry: {0}"),
        "media_flush_failed" => ("刷新文件失败: {0}", "Failed to flush file: {0}"),
        "media_read_merged_failed" => ("读取合并文件失败: {0}", "Failed to read merged file: {0}"),
        "model_price_not_configured" => ("模型 {0} 未配置定价，无法使用任务功能", "Model {0} has no pricing configured, task feature unavailable"),
        "no_available_channel" => ("没有可用的 {0} 渠道", "No available {0} channels"),
        "upstream_request_failed" => ("请求上游失败: {0}", "Failed to request upstream: {0}"),
        "date_format_invalid" => ("日期格式错误: {0}", "Invalid date format: {0}"),
        "security_rule_exists" => ("已存在启用的 {0} 规则，请直接编辑", "An enabled {0} rule already exists, edit it directly"),
        "captcha_generate_failed" => ("图形验证码生成失败: {0}", "Failed to generate captcha: {0}"),
        "order_not_found_no" => ("订单不存在: {0}", "Order not found: {0}"),
        "team_order_missing_team" => ("团队订单缺少 team_id: {0}", "Team order is missing team_id: {0}"),
        "binding_exists" => ("该用户已绑定此工厂", "This user is already bound to this factory"),
        "menu_not_found" => ("菜单不存在", "Menu not found"),
        "role_not_found" => ("角色不存在", "Role not found"),
        "role_code_exists" => ("角色编码已存在", "Role code already exists"),
        "role_has_children" => ("该角色下存在子角色，无法删除", "This role has child roles and cannot be deleted"),
        // ---- 系统：客户端（权限体系顶级维度）----
        "client_not_found" => ("客户端不存在", "Client not found"),
        "client_disabled" => ("客户端已停用", "Client has been disabled"),
        "client_required" => ("请先选择所属客户端", "Please choose a client first"),
        "client_id_required" => ("客户端ID不能为空", "Client ID is required"),
        "client_code_required" => ("客户端标识不能为空", "Client code is required"),
        "client_code_invalid" => ("客户端标识只能包含字母、数字、- 和 _，且不超过 64 个字符", "Client code may only contain letters, digits, '-' and '_' within 64 characters"),
        "client_code_exists" => ("客户端标识已存在", "Client code already exists"),
        "client_code_immutable" => ("客户端标识创建后不可修改", "Client code cannot be changed after creation"),
        "client_secret_required" => ("客户端密钥不能为空", "Client secret is required"),
        "client_secret_invalid" => ("客户端密钥错误", "Invalid client secret"),
        "client_immutable" => ("不允许跨客户端迁移", "Moving across clients is not allowed"),
        "client_login_denied" => ("该账号无权登录此客户端", "This account is not allowed to sign in to this client"),
        "menu_client_mismatch" => ("存在不属于该客户端的菜单", "Some menus do not belong to this client"),
        "role_id_required" => ("角色ID不能为空", "Role ID is required"),
        "third_party_login_unsupported" => ("不支持的第三方登录", "Unsupported third-party login"),
        "wechat_not_configured" => ("微信配置未设置", "WeChat is not configured"),
        "dingtalk_not_implemented" => ("钉钉登录尚未实现", "DingTalk login is not implemented yet"),
        "tenant_database_mode_register" => ("该租户为 database 模式，请走 database 注册流程", "This tenant uses database mode; please register via the database flow"),
        "tenant_table_register_need_default" => ("未配置默认租户 ID，无法完成 table 模式注册", "Default tenant ID is not configured; table-mode registration is unavailable"),
        "template_code_format_invalid" => ("模板编码需为 1~64 个字符", "Template code must be 1-64 characters"),
        "template_content_required" => ("正文模板不能为空", "Template content is required"),
        "template_not_found" => ("模板不存在", "Template not found"),
        "template_code_exists" => ("模板编码 {0}（{1}）已存在", "Template code {0} ({1}) already exists"),
        "wallet_credit_positive" => ("入账金额必须为正数", "Credit amount must be positive"),
        "wallet_debit_positive" => ("扣款金额必须为正数", "Debit amount must be positive"),
        "wallet_update_conflict" => ("钱包余额更新冲突（重试 3 次仍失败）", "Wallet balance update conflict (still failing after 3 retries)"),
        "wallet_adjust_remark_required" => ("调账必须填写备注", "Remark is required for balance adjustment"),
        "wallet_adjust_amount_nonzero" => ("调账金额不能为零", "Adjustment amount cannot be zero"),
        "file_not_found" => ("文件不存在", "File not found"),
        "upload_mode_read_failed" => ("读取 mode 失败: {0}", "Failed to read mode: {0}"),
        "rustfs_upload_failed" => ("RustFS 上传失败: {0}", "RustFS upload failed: {0}"),
        "rustfs_read_failed" => ("RustFS 读取失败: {0}", "RustFS read failed: {0}"),
        "rustfs_delete_failed" => ("RustFS 删除失败(key={0}): {1}", "RustFS delete failed (key={0}): {1}"),
        "rustfs_list_failed" => ("RustFS 列举对象失败(prefix={0}): {1}", "RustFS list objects failed (prefix={0}): {1}"),
        "storage_dir_list_failed" => ("读取目录({0})失败: {1}", "Failed to read directory ({0}): {1}"),
        "security_obj_type_invalid" => ("对象类型仅支持 key/user/ip", "Object type only supports key, user or ip"),
        "security_rule_type_invalid" => ("规则类型仅支持 concurrency / qps / daily_quota / default_pricing", "Rule type only supports concurrency, qps, daily_quota or default_pricing"),
        "security_name_required" => ("规则名称不能为空", "Rule name is required"),
        "security_default_pricing_required" => ("默认定价须提供输入/输出单价", "Default pricing requires input/output unit prices"),
        "security_limit_required" => ("限制值不能为空", "Limit value is required"),
        "security_limit_positive" => ("限制值必须大于 0", "Limit value must be greater than 0"),
        "security_rule_not_found" => ("规则不存在", "Rule not found"),
        "model_not_found" => ("模型不存在", "Model not found"),
        "model_name_taken" => ("模型名已存在", "Model name already exists"),
        "model_has_usage_undeletable" => ("该模型已产生用量记录，禁止删除（可单独改为下架）", "This model has usage records and cannot be deleted; retire it instead"),
        "model_invalid_id" => ("存在无效的模型 ID", "Some model IDs are invalid"),
        "model_price_negative" => ("单价不能为负数", "Unit price cannot be negative"),
        "month_format_invalid" => ("月份格式应为 YYYY-MM", "Month format should be YYYY-MM"),
        "logout_ok" => ("登出成功", "Logged out successfully"),
        "bind_ok" => ("绑定成功", "Bound successfully"),
        "batch_delete_ok" => ("批量删除成功", "Batch deleted successfully"),
        "quota_reset_ok" => ("额度已重置", "Quota has been reset"),
        "group_updated" => ("分组已调整为 {0}", "Group adjusted to {0}"),
        "blacklisted_ok" => ("已加入黑名单（缓存已失效，立即生效）", "Added to blacklist (cache invalidated, effective immediately)"),
        "removed_ok" => ("已移除", "Removed"),
        "rule_created_ok" => ("规则已创建（缓存已失效，立即生效）", "Rule created (cache invalidated, effective immediately)"),
        "rule_updated_ok" => ("规则已更新（缓存已失效，立即生效）", "Rule updated (cache invalidated, effective immediately)"),
        "transfer_submitted_ok" => ("转账申请已提交，请等待财务审核", "Transfer request submitted, awaiting finance review"),
        "invoice_submitted_ok" => ("开票申请已提交", "Invoice request submitted"),
        "batch_delete_models_ok" => ("批量删除 {0} 个模型成功", "{0} models deleted successfully"),
        "batch_delete_keys_ok" => ("批量删除 {0} 条成功", "{0} items deleted successfully"),
        "kyc_submitted_ok" => ("申请已提交，请等待审核", "Application submitted, awaiting review"),
        "twofa_enabled_ok" => ("两步验证已开启", "Two-factor verification enabled"),
        "twofa_disabled_ok" => ("两步验证已关闭", "Two-factor verification disabled"),
        "role_changed_ok" => ("角色已变更", "Role updated"),
        "member_removed_ok" => ("成员已移除", "Member removed"),
        "redeem_code_required" => ("兑换码不能为空", "Redeem code is required"),
        "redeem_ok" => ("兑换成功，到账 {0} 元", "Redeemed successfully, {0} CNY credited"),
        "ship_via_shipment" => ("确认出货请通过出货单接口操作", "Confirm shipment via the shipment endpoint"),
        "twofa_not_configured" => ("尚未配置两步验证", "Two-factor verification is not configured"),
        "twofa_secret_not_generated" => ("尚未生成验证器密钥，请先绑定", "Authenticator secret not generated yet, please bind first"),
        "twofa_secret_invalid" => ("密钥格式异常", "Invalid secret format"),
        "otp_invalid" => ("动态验证码错误或已过期", "Verification code is incorrect or expired"),
        "otp_wrong" => ("动态验证码错误", "Incorrect verification code"),
        "transfer_amount_positive" => ("转账金额必须大于零", "Transfer amount must be greater than zero"),
        "transfer_pending_exists" => ("已有待审核的转账申请", "A transfer request is pending review"),
        "transfer_not_found" => ("转账申请不存在", "Transfer request not found"),
        "request_already_processed" => ("该申请已被处理", "This request has already been processed"),
        "invoice_amount_positive" => ("开票金额必须大于零", "Invoice amount must be greater than zero"),
        "invoice_type_invalid" => ("发票类型不正确", "Invalid invoice type"),
        "invoice_title_required" => ("发票抬头不能为空", "Invoice title is required"),
        "invoice_tax_required" => ("增值税专票必须填写税号", "Tax ID is required for VAT special invoices"),
        "invoice_not_found" => ("发票记录不存在", "Invoice record not found"),
        "invoice_record_not_found" => ("开票记录不存在", "Invoice request record not found"),
        "template_code_chars_invalid" => ("模板编码仅支持字母、数字、下划线、中划线、点", "Template code only supports letters, digits, underscore, hyphen and dot"),
        "template_channel_invalid" => ("渠道仅支持 in_app（站内信）或 email（邮件）", "Channel only supports in_app or email"),
        "twofa_code_required" => ("该账号已开启两步验证，请输入动态验证码", "Two-factor verification is enabled; please enter the verification code"),
        "email_recipient_required" => ("邮件渠道必须指定接收用户", "Email channel requires recipient users"),
        "recipient_not_found" => ("接收用户不存在", "Recipient user not found"),
        "recipient_no_email" => ("接收用户未配置邮箱", "Recipient user has no email configured"),
        "notification_not_found" => ("通知不存在", "Notification not found"),
        "operation_forbidden" => ("无权操作", "Operation not allowed"),
        "file_signature_invalid" => ("签名无效或已过期", "Invalid or expired signature"),
        "file_owner_only" => ("只能删除本人上传的文件", "Only your own uploaded files can be deleted"),
        "reconciliation_empty" => ("对账查询无结果", "No reconciliation result"),
        "platform_model_not_found" => ("平台模型不存在，请先创建模型", "Platform model does not exist; create it first"),
        "wechat_pay_disabled" => ("微信支付未启用", "WeChat Pay is not enabled"),
        "wechat_callback_verify_failed" => ("微信回调验签失败", "WeChat callback signature verification failed"),
        "alipay_pay_disabled" => ("支付宝支付未启用", "Alipay is not enabled"),
        "alipay_callback_verify_failed" => ("支付宝回调验签失败", "Alipay callback signature verification failed"),
        "cipher_text_invalid" => ("密文长度非法", "Invalid ciphertext length"),
        "topup_amount_positive" => ("充值金额必须大于 0", "Top-up amount must be greater than 0"),
        "pay_channel_unsupported" => ("支付渠道仅支持 wechat / alipay", "Payment channel only supports wechat or alipay"),
        "redeem_count_range" => ("生成数量需在 1~1000 之间", "Generate count must be between 1 and 1000"),
        "redeem_denomination_positive" => ("面额必须大于 0", "Denomination must be greater than 0"),
        "redeem_code_not_found" => ("兑换码不存在", "Redeem code not found"),
        "redeem_used_not_operable" => ("已使用的兑换码不可操作", "A used redeem code cannot be operated"),
        "redeem_daily_try_limit" => ("今日兑换尝试次数已达上限", "Daily redeem attempt limit reached"),
        "redeem_code_used" => ("兑换码已被使用", "Redeem code has already been used"),
        "redeem_code_disabled" => ("兑换码已被禁用", "Redeem code has been disabled"),
        "redeem_code_expired" => ("兑换码已过期", "Redeem code has expired"),
        "user_profile_not_found" => ("用户档案不存在", "User profile not found"),
        "team_not_found" => ("团队不存在", "Team not found"),
        "team_disabled" => ("团队已被禁用", "Team has been disabled"),
        "not_team_member" => ("你不是该团队成员", "You are not a member of this team"),
        "team_owner_or_admin_only" => ("仅团队所有者或管理员可执行此操作", "Only the team owner or admins can perform this action"),
        "team_owner_only" => ("仅团队所有者可执行此操作", "Only the team owner can perform this action"),
        "team_balance_conflict" => ("团队余额更新冲突（重试 3 次仍失败）", "Team balance update conflict (still failing after 3 retries)"),
        "team_name_length" => ("团队名称需为 1~64 个字符", "Team name must be 1-64 characters"),
        "team_role_invalid" => ("角色仅支持 admin / member", "Role only supports admin or member"),
        "user_missing_or_disabled" => ("用户不存在或已被禁用", "User does not exist or is disabled"),
        "user_already_team_member" => ("该用户已是团队成员", "This user is already a team member"),
        "team_member_not_found" => ("成员不存在", "Team member not found"),
        "team_owner_role_frozen" => ("不能变更团队所有者的角色", "The team owner's role cannot be changed"),
        "team_owner_removal_forbidden" => ("不能移除团队所有者", "The team owner cannot be removed"),

        _ => return None,
    };
    Some(pair)
}

/// 解析并翻译 `@key[:arg1|arg2]`；非 `@` 消息或未知 key 原样返回
pub fn resolve(msg: &str, lang: Language) -> String {
    let trimmed = msg.trim();
    if !trimmed.starts_with('@') {
        return msg.to_string();
    }
    let body = &trimmed[1..];
    let (key, args) = match body.split_once(':') {
        Some((k, rest)) => (
            k,
            rest.split('|').map(|s| s.to_string()).collect::<Vec<_>>(),
        ),
        None => (body, Vec::new()),
    };
    let Some((zh, en)) = lookup(key) else {
        return msg.to_string();
    };
    let template = match lang {
        Language::Zh => zh,
        Language::En => en,
    };
    if args.is_empty() {
        return template.to_string();
    }
    let mut out = template.to_string();
    for (i, arg) in args.iter().enumerate() {
        out = out.replace(&format!("{{{}}}", i), arg);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_plain_text_passthrough() {
        assert_eq!(resolve("选品不存在", Language::En), "选品不存在");
    }

    #[test]
    fn resolve_unknown_key_passthrough() {
        assert_eq!(resolve("@no_such_key", Language::En), "@no_such_key");
    }

    #[test]
    fn resolve_key_zh_en() {
        assert_eq!(resolve("@product_not_found", Language::Zh), "选品不存在");
        assert_eq!(resolve("@product_not_found", Language::En), "Product not found");
    }

    #[test]
    fn resolve_key_with_args() {
        assert_eq!(
            resolve("@product_status_forbidden:pool", Language::En),
            "Current status \"pool\" does not allow this operation"
        );
        assert_eq!(
            resolve("@product_status_forbidden:已出货", Language::Zh),
            "当前状态「已出货」不可执行该操作"
        );
    }

    #[test]
    fn from_language_header() {
        assert_eq!(Language::from_accept_language(None), Language::Zh);
        assert_eq!(Language::from_accept_language(Some("zh-CN")), Language::Zh);
        assert_eq!(Language::from_accept_language(Some("en-US")), Language::En);
        assert_eq!(Language::from_accept_language(Some("EN,en;q=0.9")), Language::En);
        assert_eq!(Language::from_accept_language(Some("fr-FR")), Language::Zh);
    }
}
