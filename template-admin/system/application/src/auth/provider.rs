use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindDto {
    pub provider: String,
    /// 授权码（bind 必填；unbind 不传，serde default 空）
    #[serde(default)]
    pub code: String,
    #[serde(default)]
    pub state: Option<String>,
    /// Web 扫码绑定的 CSRF state（provider=wechat 时必填，Redis 一次性消费）
    #[serde(default)]
    pub qr_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderVo {
    pub name: String,
    pub display_name: String,
    pub icon: Option<String>,
    pub authorize_url: String,
}

/// 当前账号已绑定的第三方身份（供前端展示绑定状态 / 提供解绑入口）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderBindingVo {
    /// 身份类型（如 wechat_mp）
    pub provider: String,
}

impl ProviderVo {
    pub fn new(name: &str, display_name: &str, icon: Option<&str>, authorize_url: &str) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
            icon: icon.map(|s| s.to_string()),
            authorize_url: authorize_url.to_string(),
        }
    }
}