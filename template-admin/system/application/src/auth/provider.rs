use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BindDto {
    pub provider: String,
    pub code: String,
    pub state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderVo {
    pub name: String,
    pub display_name: String,
    pub icon: Option<String>,
    pub authorize_url: String,
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