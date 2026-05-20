use serde::{Deserialize, Serialize};
use crate::menu::dto::MenuVo;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginDto {
    pub user_name: Option<String>,
    pub pass_word: String,
    pub email: Option<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenVo {
    pub token: String,
    pub token_name: String,
    pub token_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserInfoVo {
    pub id: i64,
    pub user_name: String,
    pub nick_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub avatar: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub menus: Vec<MenuVo>,
}