use serde::{Deserialize, Serialize};
use crate::menu::dto::MenuVo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginDto {
    pub user_name: Option<String>,
    pub pass_word: String,
    pub email: Option<String>,
    pub login_type: Option<String>,
    pub remember_me: Option<bool>,
    pub tenant_code: Option<String>,
    pub code: Option<String>,
    pub state: Option<String>,
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
    pub email: Option<String>,
    pub phone: Option<String>,
    pub register_type: Option<String>,
    pub tenant_code: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThirdPartyCallbackVo {
    pub need_bind: bool,
    pub openid: String,
    pub user_name: Option<String>,
    pub tenant_code: Option<String>,
}