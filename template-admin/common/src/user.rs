use sa_token_core::SaTokenContext;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    SaTokenContext::try_current().and_then(|ctx| ctx.login_id)
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
    SaTokenContext::try_current().and_then(|ctx| ctx.login_id)
}

pub async fn get_current_user_async() -> Option<CurrentUser> {
    get_current_user()
}