use sa_token_core::SaTokenContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// 客户端会话登录 ID 分隔符。
///
/// 同一账号在不同客户端登录时使用 `用户ID#客户端编码` 作为 sa-token 的 login_id，
/// 使角色/权限快照按客户端隔离（否则后登录的客户端会覆盖前一个客户端的权限集）。
/// 未携带客户端编码的历史登录方式沿用纯用户 ID。
pub const CLIENT_LOGIN_SEP: char = '#';

/// 构造按客户端隔离后的 sa-token login_id。
pub fn build_scoped_login_id(user_id: &str, client_code: Option<&str>) -> String {
    match client_code.map(str::trim).filter(|v| !v.is_empty()) {
        Some(code) => format!("{user_id}{CLIENT_LOGIN_SEP}{code}"),
        None => user_id.to_string(),
    }
}

/// 从 sa-token login_id 还原真实用户 ID（去掉客户端后缀）。
pub fn base_user_id_from_login_id(login_id: &str) -> String {
    login_id
        .split_once(CLIENT_LOGIN_SEP)
        .map(|(user_id, _)| user_id)
        .unwrap_or(login_id)
        .to_string()
}

/// 从 sa-token login_id 解析客户端编码。
pub fn client_code_from_login_id(login_id: &str) -> Option<String> {
    login_id
        .split_once(CLIENT_LOGIN_SEP)
        .map(|(_, code)| code.trim().to_string())
        .filter(|code| !code.is_empty())
}

/// 第三方自动注册时的占位用户名：`wx_` + `sha256(identity_value)` 前 8 字节的 16 位 hex。
///
/// 取名确定性（同一第三方标识必得同名）是并发首登的关键：两个请求同时建号时，
/// 双方算出同一个占位名，输的一方因唯一键失败后可按第三方标识重查恢复。
pub fn placeholder_user_name(identity_value: &str) -> String {
    use sha2::{Digest, Sha256};
    let digest = Sha256::digest(identity_value.as_bytes());
    let mut hex = String::with_capacity(16);
    for byte in digest.iter().take(8) {
        hex.push_str(&format!("{byte:02x}"));
    }
    format!("wx_{hex}")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurrentUser {
    pub id: String,
    pub user_name: String,
    pub nick_name: Option<String>,
    pub tenant_id: Option<String>,
    pub tenant_code: Option<String>,
    pub tenant_mode: Option<String>,
}

fn get_extra_data() -> Option<Value> {
    SaTokenContext::try_current()
        .and_then(|ctx| ctx.token_info)
        .and_then(|info| info.extra_data.clone())
}

pub fn get_current_user_id() -> Option<String> {
    SaTokenContext::try_current()
        .and_then(|ctx| ctx.login_id)
        .map(|login_id| base_user_id_from_login_id(&login_id))
}

/// 当前请求完整的 sa-token login_id（可能带客户端后缀 `用户ID#客户端编码`）。
pub fn get_current_login_id() -> Option<String> {
    SaTokenContext::try_current().and_then(|ctx| ctx.login_id)
}

/// 当前登录客户端编码：优先取 token extra（clientCode），
/// 兼容 refresh 后 extra 缺失的场景，回退解析 login_id 后缀。
pub fn get_current_client_code() -> Option<String> {
    if let Some(data) = get_extra_data() {
        if let Some(code) = data
            .get("clientCode")
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|v| !v.is_empty())
        {
            return Some(code.to_string());
        }
    }
    get_current_login_id().and_then(|login_id| client_code_from_login_id(&login_id))
}

/// 当前登录客户端主键 ID（token extra 的 clientId）。
pub fn get_current_client_id() -> Option<String> {
    get_extra_data()?
        .get("clientId")?
        .as_str()
        .map(|s| s.to_string())
}

pub fn get_current_user_name() -> Option<String> {
    let data = get_extra_data()?;
    data.get("userName")?.as_str().map(|s: &str| s.to_string())
}

pub fn get_current_user_nick_name() -> Option<String> {
    let data = get_extra_data()?;
    data.get("nickName")?.as_str().map(|s: &str| s.to_string())
}

pub fn get_current_tenant_id() -> Option<String> {
    let data = get_extra_data()?;
    data.get("tenantId")?.as_str().map(|s: &str| s.to_string())
}

pub fn get_current_tenant_code() -> Option<String> {
    let data = get_extra_data()?;
    data.get("tenantCode")?.as_str().map(|s: &str| s.to_string())
}

pub fn get_current_tenant_mode() -> Option<String> {
    let data = get_extra_data()?;
    data.get("tenantMode")?.as_str().map(|s: &str| s.to_string())
}

pub fn get_current_user() -> Option<CurrentUser> {
    let id = get_current_user_id()?;
    let user_name = get_current_user_name().unwrap_or_default();
    let nick_name = get_current_user_nick_name();
    let tenant_id = get_current_tenant_id();
    let tenant_code = get_current_tenant_code();
    let tenant_mode = get_current_tenant_mode();
    Some(CurrentUser {
        id,
        user_name,
        nick_name,
        tenant_id,
        tenant_code,
        tenant_mode,
    })
}

pub async fn get_current_user_id_async() -> Option<String> {
    get_current_user_id()
}

pub async fn get_current_user_async() -> Option<CurrentUser> {
    get_current_user()
}
