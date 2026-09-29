use serde::{Deserialize, Serialize};
use crate::menu::dto::MenuVo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginDto {
    pub user_name: Option<String>,
    /// 密码：仅 loginType = "password" / "email" 时必填；
    /// 第三方登录（wechat / wechat_mp / dingtalk）不带该字段，故必须可选，否则反序列化直接失败
    pub pass_word: Option<String>,
    pub email: Option<String>,
    pub login_type: Option<String>,
    pub remember_me: Option<bool>,
    pub tenant_code: Option<String>,
    /// 客户端标识（OAuth2 的 client_id）：携带时按客户端加载菜单与功能权限，
    /// 并限制该账号只能登录其拥有角色的客户端；不携带时保持历史行为（不按客户端限制）
    #[serde(default)]
    pub client_code: Option<String>,
    /// 客户端密钥：客户端配置了密钥时必填并校验
    #[serde(default)]
    pub client_secret: Option<String>,
    pub code: Option<String>,
    pub state: Option<String>,
    /// Web 扫码登录的 CSRF state（与 `state` 分开：`state` 已被 tenantCode 复用）。
    /// 由 `GET /api/auth/wechat/qr-url` 生成，Redis 一次性消费。
    #[serde(default)]
    pub qr_state: Option<String>,
    /// 两步验证动态码（TOTP，6 位；启用 2FA 的账号登录必填）
    #[serde(default)]
    pub totp_code: Option<String>,
    /// 邮箱验证码（loginType = "email_code" 时必填，由官网发送，一次性校验）
    #[serde(default)]
    pub email_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocateTenantDto {
    pub user_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocateTenantVo {
    pub tenant_name: String,
    pub tenant_code: String,
    pub tenant_logo: Option<String>,
    pub user_name: String,
    pub login_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterDto {
    pub user_name: String,
    pub pass_word: String,
    pub nick_name: Option<String>,
    /// 邮箱（必填）：注册前必须通过邮箱验证码校验，账号一诞生就带着已验证的邮箱
    pub email: String,
    /// 邮箱验证码（必填，一次性消费；与发码端共享 `mail:code:{email}:register`）
    pub email_code: String,
    pub phone: Option<String>,
    pub tenant_code: Option<String>,
    /// 注册身份（必填）：必须是 `register_role` 字典的**启用**项，且能对上默认客户端下的角色编码。
    /// 后端绝不接受前端传 role_id，只接受 role_code。
    pub role_code: String,
}

/// 完善账户信息（微信自动注册后的惰性账号首次进入系统前必填）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteProfileDto {
    pub user_name: String,
    pub pass_word: String,
    pub phone: String,
    pub nick_name: Option<String>,
    /// 身份（必填）：取 `register_role` 字典的启用项
    pub role_code: String,
    /// 邮箱（必填）：与注册同口径，必须通过邮箱验证码校验
    pub email: String,
    /// 邮箱验证码（必填，一次性消费）
    pub email_code: String,
}

/// 修改个人信息（昵称 / 手机号）。
///
/// 用户名是登录凭据（全局唯一）不可改；邮箱与密码各有独立接口，均需邮箱验证码认证。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfileDto {
    /// 昵称（可选，最长 30 字符；空串表示清空）
    pub nick_name: Option<String>,
    /// 手机号（可选；填写须为合法手机号，空串表示清空）
    pub phone: Option<String>,
}

/// 修改密码（认证：验证码发到账号**当前**绑定邮箱，scene=change_password）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordDto {
    /// 邮箱验证码（必填，一次性消费）
    pub email_code: String,
    /// 新密码（8~64 位，需同时含字母与数字）
    pub new_password: String,
}

/// 更换绑定邮箱（认证：验证码发到**新**邮箱，scene=change_email）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangeEmailDto {
    /// 新邮箱（需未被其它账号占用）
    pub new_email: String,
    /// 新邮箱收到的验证码（必填，一次性消费）
    pub email_code: String,
}

/// 注册身份选项（免登录公开接口返回，仅「标签 + 角色码」）。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterRoleVo {
    pub label: String,
    pub value: String,
}

/// Web 微信扫码登录：二维码地址 + 一次性 CSRF state。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WechatQrVo {
    pub url: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenVo {
    pub token: String,
    pub token_name: String,
    pub token_prefix: String,
    pub refresh_token: Option<String>,
    pub expire_time: Option<i64>,
    pub refresh_expire_time: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshTokenDto {
    pub refresh_token: String,
}

/// 发送邮箱验证码。
///
/// scene 取值：`register`（注册/完善资料）/ `reset`（找回密码）/ `login`（邮箱验证码登录）
/// / `change_password`（改密码）/ `change_email`（换邮箱）。
/// 发码前必须携带图形验证码（captchaId + captchaCode），防止接口被脚本刷。
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendCodeDto {
    pub email: String,
    pub scene: String,
    pub captcha_id: String,
    pub captcha_code: String,
}

/// 图形验证码响应：captchaId 供提交回传；image 为 PNG data URI（Web <img> 直显）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptchaVo {
    pub captcha_id: String,
    pub image: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoVo {
    pub id: String,
    pub user_name: String,
    pub nick_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub menus: Vec<MenuVo>,
    pub tenant_id: Option<String>,
    pub tenant_code: Option<String>,
    pub tenant_name: Option<String>,
    /// 账号信息是否已完善（`auth_sys_user.pass_word` 非空即视为已完善）。
    /// 自动注册的惰性账号为 false，前端据此强制跳「完善账户信息」页。
    pub profile_completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThirdPartyCallbackVo {
    pub need_bind: bool,
    pub openid: String,
    pub user_name: Option<String>,
    pub tenant_code: Option<String>,
}
